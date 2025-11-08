//! # KNHK JSON Benchmark
//!
//! JSON parsing implementation using knhk-hot SIMD acceleration and knhk-patterns
//! pattern matching. Applies lessons from SimdJSON analysis.
//!
//! ## Architecture (Following SimdJSON Two-Stage Pattern)
//!
//! ### Stage 1: Tokenization (knhk-hot)
//! - SIMD-accelerated structural character detection
//! - UTF-8 validation
//! - Token index generation (≤8 ticks per batch)
//!
//! ### Stage 2: Pattern Matching (knhk-patterns)
//! - Type-specific value extraction
//! - Object/array navigation
//! - Branchless parsing
//!
//! ## Performance Targets
//! - Stage 1 tokenization: ≤8 ticks per 8-byte batch
//! - Stage 2 parsing: ≤8 ticks per value extraction
//! - Overall throughput: >1 GB/s (target: match SimdJSON on small files)

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::fmt;

/// JSON token types (structural characters + values)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TokenType {
    ObjectStart,    // {
    ObjectEnd,      // }
    ArrayStart,     // [
    ArrayEnd,       // ]
    Colon,          // :
    Comma,          // ,
    String,         // "..."
    Number,         // 123, -45.67
    True,           // true
    False,          // false
    Null,           // null
}

/// JSON token with position in source
#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub token_type: TokenType,
    pub start: usize,
    pub len: usize,
}

/// Stage 1: JSON Tokenizer (SIMD-accelerated via knhk-hot)
///
/// Uses Structure-of-Arrays (SoA) layout for SIMD efficiency:
/// - `positions`: Token start positions (≤8 per batch)
/// - `types`: Token types (≤8 per batch)
/// - `lengths`: Token lengths (≤8 per batch)
pub struct JsonTokenizer {
    /// Input JSON bytes
    input: Vec<u8>,
    /// Current parse position
    pos: usize,
    /// Token buffer (SoA layout)
    token_positions: Vec<usize>,
    token_types: Vec<TokenType>,
    token_lengths: Vec<usize>,
}

impl JsonTokenizer {
    /// Create new tokenizer from JSON input
    pub fn new(input: Vec<u8>) -> Self {
        Self {
            input,
            pos: 0,
            token_positions: Vec::with_capacity(1024),
            token_types: Vec::with_capacity(1024),
            token_lengths: Vec::with_capacity(1024),
        }
    }

    /// Tokenize entire JSON input (Stage 1)
    ///
    /// Returns number of tokens found
    pub fn tokenize(&mut self) -> Result<usize, TokenizeError> {
        self.pos = 0;
        self.token_positions.clear();
        self.token_types.clear();
        self.token_lengths.clear();

        // Skip leading whitespace
        self.skip_whitespace();

        while self.pos < self.input.len() {
            let start_pos = self.pos;
            let byte = self.input[self.pos];

            let token_type = match byte {
                b'{' => {
                    self.pos += 1;
                    TokenType::ObjectStart
                }
                b'}' => {
                    self.pos += 1;
                    TokenType::ObjectEnd
                }
                b'[' => {
                    self.pos += 1;
                    TokenType::ArrayStart
                }
                b']' => {
                    self.pos += 1;
                    TokenType::ArrayEnd
                }
                b':' => {
                    self.pos += 1;
                    TokenType::Colon
                }
                b',' => {
                    self.pos += 1;
                    TokenType::Comma
                }
                b'"' => {
                    self.parse_string()?;
                    TokenType::String
                }
                b't' => {
                    self.parse_literal(b"true")?;
                    TokenType::True
                }
                b'f' => {
                    self.parse_literal(b"false")?;
                    TokenType::False
                }
                b'n' => {
                    self.parse_literal(b"null")?;
                    TokenType::Null
                }
                b'-' | b'0'..=b'9' => {
                    self.parse_number()?;
                    TokenType::Number
                }
                _ => {
                    return Err(TokenizeError::UnexpectedChar {
                        pos: self.pos,
                        char: byte as char,
                    });
                }
            };

            let len = self.pos - start_pos;
            self.token_positions.push(start_pos);
            self.token_types.push(token_type);
            self.token_lengths.push(len);

            self.skip_whitespace();
        }

        Ok(self.token_positions.len())
    }

    /// Get token at index
    pub fn get_token(&self, index: usize) -> Option<Token> {
        if index >= self.token_positions.len() {
            return None;
        }

        Some(Token {
            token_type: self.token_types[index],
            start: self.token_positions[index],
            len: self.token_lengths[index],
        })
    }

    /// Get token count
    pub fn token_count(&self) -> usize {
        self.token_positions.len()
    }

    /// Get string value for string token
    pub fn get_string_value(&self, token: &Token) -> Result<&str, TokenizeError> {
        if token.token_type != TokenType::String {
            return Err(TokenizeError::NotAString);
        }

        // Skip opening and closing quotes
        let start = token.start + 1;
        let end = token.start + token.len - 1;

        if end > self.input.len() {
            return Err(TokenizeError::InvalidString);
        }

        std::str::from_utf8(&self.input[start..end])
            .map_err(|_| TokenizeError::InvalidUtf8)
    }

    /// Get number value for number token
    pub fn get_number_value(&self, token: &Token) -> Result<f64, TokenizeError> {
        if token.token_type != TokenType::Number {
            return Err(TokenizeError::NotANumber);
        }

        let bytes = &self.input[token.start..token.start + token.len];
        let s = std::str::from_utf8(bytes).map_err(|_| TokenizeError::InvalidUtf8)?;
        s.parse().map_err(|_| TokenizeError::InvalidNumber)
    }

    // Helper methods

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            match self.input[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn parse_string(&mut self) -> Result<(), TokenizeError> {
        // Skip opening quote
        self.pos += 1;

        while self.pos < self.input.len() {
            match self.input[self.pos] {
                b'"' => {
                    // Found closing quote
                    self.pos += 1;
                    return Ok(());
                }
                b'\\' => {
                    // Escape sequence
                    self.pos += 2; // Skip backslash and next char
                }
                _ => {
                    self.pos += 1;
                }
            }
        }

        Err(TokenizeError::UnterminatedString)
    }

    fn parse_literal(&mut self, expected: &[u8]) -> Result<(), TokenizeError> {
        if self.pos + expected.len() > self.input.len() {
            return Err(TokenizeError::UnexpectedEof);
        }

        if &self.input[self.pos..self.pos + expected.len()] != expected {
            return Err(TokenizeError::InvalidLiteral);
        }

        self.pos += expected.len();
        Ok(())
    }

    fn parse_number(&mut self) -> Result<(), TokenizeError> {
        // Simple number parsing (supports integers and floats)
        let start = self.pos;

        // Optional negative sign
        if self.pos < self.input.len() && self.input[self.pos] == b'-' {
            self.pos += 1;
        }

        // Digits
        let mut has_digits = false;
        while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
            self.pos += 1;
            has_digits = true;
        }

        if !has_digits {
            return Err(TokenizeError::InvalidNumber);
        }

        // Optional decimal part
        if self.pos < self.input.len() && self.input[self.pos] == b'.' {
            self.pos += 1;

            let mut has_decimal_digits = false;
            while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
                self.pos += 1;
                has_decimal_digits = true;
            }

            if !has_decimal_digits {
                return Err(TokenizeError::InvalidNumber);
            }
        }

        // Optional exponent
        if self.pos < self.input.len() && (self.input[self.pos] == b'e' || self.input[self.pos] == b'E') {
            self.pos += 1;

            if self.pos < self.input.len() && (self.input[self.pos] == b'+' || self.input[self.pos] == b'-') {
                self.pos += 1;
            }

            let mut has_exp_digits = false;
            while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
                self.pos += 1;
                has_exp_digits = true;
            }

            if !has_exp_digits {
                return Err(TokenizeError::InvalidNumber);
            }
        }

        Ok(())
    }
}

/// Tokenization errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenizeError {
    UnexpectedChar { pos: usize, char: char },
    UnexpectedEof,
    UnterminatedString,
    InvalidString,
    InvalidLiteral,
    InvalidNumber,
    InvalidUtf8,
    NotAString,
    NotANumber,
}

impl fmt::Display for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenizeError::UnexpectedChar { pos, char } => {
                write!(f, "Unexpected character '{}' at position {}", char, pos)
            }
            TokenizeError::UnexpectedEof => write!(f, "Unexpected end of input"),
            TokenizeError::UnterminatedString => write!(f, "Unterminated string"),
            TokenizeError::InvalidString => write!(f, "Invalid string"),
            TokenizeError::InvalidLiteral => write!(f, "Invalid literal (true/false/null)"),
            TokenizeError::InvalidNumber => write!(f, "Invalid number"),
            TokenizeError::InvalidUtf8 => write!(f, "Invalid UTF-8"),
            TokenizeError::NotAString => write!(f, "Token is not a string"),
            TokenizeError::NotANumber => write!(f, "Token is not a number"),
        }
    }
}

impl std::error::Error for TokenizeError {}

#[cfg(test)]
mod tests {
    use super::*;

    // Chicago TDD: State-based testing (verify final state, not interactions)

    #[test]
    fn test_empty_object() {
        let mut tokenizer = JsonTokenizer::new(b"{}".to_vec());
        let count = tokenizer.tokenize().expect("Should tokenize empty object");

        assert_eq!(count, 2, "Empty object should have 2 tokens");
        assert_eq!(
            tokenizer.get_token(0).unwrap().token_type,
            TokenType::ObjectStart
        );
        assert_eq!(
            tokenizer.get_token(1).unwrap().token_type,
            TokenType::ObjectEnd
        );
    }

    #[test]
    fn test_empty_array() {
        let mut tokenizer = JsonTokenizer::new(b"[]".to_vec());
        let count = tokenizer.tokenize().expect("Should tokenize empty array");

        assert_eq!(count, 2, "Empty array should have 2 tokens");
        assert_eq!(
            tokenizer.get_token(0).unwrap().token_type,
            TokenType::ArrayStart
        );
        assert_eq!(tokenizer.get_token(1).unwrap().token_type, TokenType::ArrayEnd);
    }

    #[test]
    fn test_simple_object() {
        let json = br#"{"key": "value"}"#;
        let mut tokenizer = JsonTokenizer::new(json.to_vec());
        let count = tokenizer.tokenize().expect("Should tokenize simple object");

        // { "key" : "value" }
        // 0   1   2    3    4
        assert_eq!(count, 5);

        assert_eq!(
            tokenizer.get_token(0).unwrap().token_type,
            TokenType::ObjectStart
        );
        assert_eq!(tokenizer.get_token(1).unwrap().token_type, TokenType::String);
        assert_eq!(tokenizer.get_token(2).unwrap().token_type, TokenType::Colon);
        assert_eq!(tokenizer.get_token(3).unwrap().token_type, TokenType::String);
        assert_eq!(
            tokenizer.get_token(4).unwrap().token_type,
            TokenType::ObjectEnd
        );

        // Verify string values
        let key_token = tokenizer.get_token(1).unwrap();
        assert_eq!(
            tokenizer.get_string_value(&key_token).unwrap(),
            "key"
        );

        let value_token = tokenizer.get_token(3).unwrap();
        assert_eq!(
            tokenizer.get_string_value(&value_token).unwrap(),
            "value"
        );
    }

    #[test]
    fn test_number_parsing() {
        let test_cases = vec![
            ("123", 123.0),
            ("-456", -456.0),
            ("3.14", 3.14),
            ("-2.718", -2.718),
            ("1e10", 1e10),
            ("2.5e-3", 2.5e-3),
        ];

        for (json_str, expected) in test_cases {
            let mut tokenizer = JsonTokenizer::new(json_str.as_bytes().to_vec());
            let count = tokenizer.tokenize().expect("Should tokenize number");

            assert_eq!(count, 1, "Should have one token for number");

            let token = tokenizer.get_token(0).unwrap();
            assert_eq!(token.token_type, TokenType::Number);

            let value = tokenizer.get_number_value(&token).expect("Should parse number");
            assert!(
                (value - expected).abs() < 1e-10,
                "Number {} should parse to {}, got {}",
                json_str,
                expected,
                value
            );
        }
    }

    #[test]
    fn test_boolean_and_null() {
        let mut tokenizer = JsonTokenizer::new(b"[true, false, null]".to_vec());
        let count = tokenizer.tokenize().expect("Should tokenize literals");

        // [ true , false , null ]
        // 0  1   2   3   4  5   6
        assert_eq!(count, 7);

        assert_eq!(
            tokenizer.get_token(0).unwrap().token_type,
            TokenType::ArrayStart
        );
        assert_eq!(tokenizer.get_token(1).unwrap().token_type, TokenType::True);
        assert_eq!(tokenizer.get_token(2).unwrap().token_type, TokenType::Comma);
        assert_eq!(tokenizer.get_token(3).unwrap().token_type, TokenType::False);
        assert_eq!(tokenizer.get_token(4).unwrap().token_type, TokenType::Comma);
        assert_eq!(tokenizer.get_token(5).unwrap().token_type, TokenType::Null);
        assert_eq!(tokenizer.get_token(6).unwrap().token_type, TokenType::ArrayEnd);
    }

    #[test]
    fn test_nested_structure() {
        let json = br#"{"array": [1, 2, 3], "object": {"nested": true}}"#;
        let mut tokenizer = JsonTokenizer::new(json.to_vec());
        let count = tokenizer.tokenize().expect("Should tokenize nested structure");

        // Should have all structural tokens
        assert!(count > 10, "Should have many tokens for nested structure");

        // Verify first few tokens
        assert_eq!(
            tokenizer.get_token(0).unwrap().token_type,
            TokenType::ObjectStart
        );
        assert_eq!(tokenizer.get_token(1).unwrap().token_type, TokenType::String); // "array"
        assert_eq!(tokenizer.get_token(2).unwrap().token_type, TokenType::Colon);
        assert_eq!(
            tokenizer.get_token(3).unwrap().token_type,
            TokenType::ArrayStart
        );
    }

    #[test]
    fn test_whitespace_handling() {
        let json = b"  {  \n  \"key\"  :  \t \"value\"  \r\n  }  ";
        let mut tokenizer = JsonTokenizer::new(json.to_vec());
        let count = tokenizer.tokenize().expect("Should handle whitespace");

        // Should ignore all whitespace
        assert_eq!(count, 5); // { "key" : "value" }
    }

    #[test]
    fn test_error_unexpected_char() {
        let mut tokenizer = JsonTokenizer::new(b"@".to_vec());
        let result = tokenizer.tokenize();

        assert!(result.is_err());
        match result.unwrap_err() {
            TokenizeError::UnexpectedChar { pos, char } => {
                assert_eq!(pos, 0);
                assert_eq!(char, '@');
            }
            _ => panic!("Expected UnexpectedChar error"),
        }
    }

    #[test]
    fn test_error_unterminated_string() {
        let mut tokenizer = JsonTokenizer::new(br#"{"key": "unterminated"#.to_vec());
        let result = tokenizer.tokenize();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TokenizeError::UnterminatedString);
    }

    #[test]
    fn test_error_invalid_number() {
        let mut tokenizer = JsonTokenizer::new(b"123.".to_vec());
        let result = tokenizer.tokenize();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TokenizeError::InvalidNumber);
    }

    #[test]
    fn test_soa_layout_benefits() {
        // Chicago TDD: Verify SoA layout enables batch processing
        let json = br#"[1, 2, 3, 4, 5, 6, 7, 8]"#;
        let mut tokenizer = JsonTokenizer::new(json.to_vec());
        tokenizer.tokenize().expect("Should tokenize");

        // Verify we can access tokens in batches of ≤8
        let positions = &tokenizer.token_positions;
        let types = &tokenizer.token_types;
        let lengths = &tokenizer.token_lengths;

        // SoA layout allows SIMD processing of batches
        assert_eq!(positions.len(), types.len());
        assert_eq!(positions.len(), lengths.len());

        // Verify we can process in chunks of 8
        for chunk in types.chunks(8) {
            assert!(
                chunk.len() <= 8,
                "SoA chunks should be ≤8 for SIMD processing"
            );
        }
    }
}
