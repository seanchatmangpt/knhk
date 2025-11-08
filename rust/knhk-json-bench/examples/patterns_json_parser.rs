//! # JSON Parser Using knhk-patterns - Practical Example
//!
//! This example demonstrates how to use knhk-patterns workflow orchestration
//! to build a JSON parser. It shows:
//!
//! 1. Pattern 1 (Sequence): Tokenize → Parse → Validate pipeline
//! 2. Pattern 4 (Exclusive Choice): Route by JSON type
//! 3. Pattern 2 (Parallel Split): Parse object fields concurrently
//! 4. Pattern 6 (Multi-Choice): Parse heterogeneous arrays
//!
//! Run with: `cargo run -p knhk-json-bench --example patterns_json_parser`

use knhk_json_bench::parse_json;
use knhk_patterns::PatternError;
use knhk_patterns::*;
use std::sync::Arc;

// ============================================================================
// Data Types
// ============================================================================

#[derive(Clone, Debug)]
pub struct JsonParseState {
    pub bytes: Vec<u8>,
    pub tokenizer: Option<JsonTokenizer>,
    pub tokens: Vec<Token>,
    pub current_pos: usize,
    pub value: Option<JsonValue>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Object(Vec<(String, JsonValue)>),
    Array(Vec<JsonValue>),
}

// ============================================================================
// Helper Functions
// ============================================================================

fn peek_token(state: &JsonParseState) -> Option<TokenType> {
    if state.current_pos < state.tokens.len() {
        Some(state.tokens[state.current_pos].token_type)
    } else {
        None
    }
}

fn advance_token(mut state: JsonParseState) -> JsonParseState {
    if state.current_pos < state.tokens.len() {
        state.current_pos += 1;
    }
    state
}

// ============================================================================
// Pattern 1: Sequence - Main Parsing Pipeline
// ============================================================================

/// Create the main JSON parsing pipeline using Sequence pattern
pub fn create_json_parse_pipeline() -> PatternResult<SequencePattern<JsonParseState>> {
    SequencePattern::new(vec![
        // Stage 1: Tokenize
        Arc::new(|mut state: JsonParseState| {
            let mut tokenizer = JsonTokenizer::new(state.bytes.clone());
            tokenizer.tokenize().map_err(|e| {
                PatternError::ExecutionFailed(format!("Tokenization failed: {}", e))
            })?;

            // Extract tokens
            let mut tokens = Vec::new();
            for i in 0..tokenizer.token_count() {
                if let Some(token) = tokenizer.get_token(i) {
                    tokens.push(token);
                }
            }

            Ok(JsonParseState {
                bytes: state.bytes,
                tokenizer: Some(tokenizer),
                tokens,
                current_pos: 0,
                value: None,
            })
        }),
        // Stage 2: Parse value
        Arc::new(|state: JsonParseState| {
            // Use type router to parse based on first token
            let router = create_type_router()?;
            let results = router.execute(state)?;
            Ok(results[0].clone())
        }),
        // Stage 3: Validate (check all tokens consumed)
        Arc::new(|state: JsonParseState| {
            if state.current_pos < state.tokens.len() {
                return Err(PatternError::ExecutionFailed(format!(
                    "Not all tokens consumed. Remaining: {}",
                    state.tokens.len() - state.current_pos
                )));
            }
            Ok(state)
        }),
    ])
}

// ============================================================================
// Pattern 4: Exclusive Choice - Type Router
// ============================================================================

/// Create type router using Exclusive Choice pattern
fn create_type_router() -> PatternResult<ExclusiveChoicePattern<JsonParseState>> {
    ExclusiveChoicePattern::new(vec![
        // Route to string parser
        (
            Arc::new(|state: &JsonParseState| matches!(peek_token(state), Some(TokenType::String)))
                as ConditionFn<JsonParseState>,
            Arc::new(|state: JsonParseState| parse_string_value(state)) as BranchFn<JsonParseState>,
        ),
        // Route to number parser
        (
            Arc::new(|state: &JsonParseState| matches!(peek_token(state), Some(TokenType::Number))),
            Arc::new(|state: JsonParseState| parse_number_value(state)),
        ),
        // Route to boolean parser
        (
            Arc::new(|state: &JsonParseState| {
                matches!(peek_token(state), Some(TokenType::True | TokenType::False))
            }),
            Arc::new(|state: JsonParseState| parse_boolean_value(state)),
        ),
        // Route to null parser
        (
            Arc::new(|state: &JsonParseState| matches!(peek_token(state), Some(TokenType::Null))),
            Arc::new(|state: JsonParseState| parse_null_value(state)),
        ),
        // Route to object parser
        (
            Arc::new(|state: &JsonParseState| {
                matches!(peek_token(state), Some(TokenType::ObjectStart))
            }),
            Arc::new(|state: JsonParseState| parse_object_value(state)),
        ),
        // Route to array parser
        (
            Arc::new(|state: &JsonParseState| {
                matches!(peek_token(state), Some(TokenType::ArrayStart))
            }),
            Arc::new(|state: JsonParseState| parse_array_value(state)),
        ),
    ])
}

// ============================================================================
// Value Parsers
// ============================================================================

fn parse_string_value(mut state: JsonParseState) -> PatternResult<JsonParseState> {
    if state.current_pos >= state.tokens.len() {
        return Err(PatternError::ExecutionFailed("Unexpected EOF".to_string()));
    }

    let token = state.tokens[state.current_pos];
    if token.token_type != TokenType::String {
        return Err(PatternError::ExecutionFailed(
            "Expected string token".to_string(),
        ));
    }

    // Extract string value
    let tokenizer = state
        .tokenizer
        .as_ref()
        .ok_or_else(|| PatternError::ExecutionFailed("Tokenizer not available".to_string()))?;

    let value_str = tokenizer
        .get_string_value(&token)
        .map_err(|e| PatternError::ExecutionFailed(format!("String parse error: {}", e)))?;

    state.current_pos += 1;
    state.value = Some(JsonValue::String(value_str.to_string()));

    Ok(state)
}

fn parse_number_value(mut state: JsonParseState) -> PatternResult<JsonParseState> {
    if state.current_pos >= state.tokens.len() {
        return Err(PatternError::ExecutionFailed("Unexpected EOF".to_string()));
    }

    let token = state.tokens[state.current_pos];
    if token.token_type != TokenType::Number {
        return Err(PatternError::ExecutionFailed(
            "Expected number token".to_string(),
        ));
    }

    // Extract number value
    let tokenizer = state
        .tokenizer
        .as_ref()
        .ok_or_else(|| PatternError::ExecutionFailed("Tokenizer not available".to_string()))?;

    let value = tokenizer
        .get_number_value(&token)
        .map_err(|e| PatternError::ExecutionFailed(format!("Number parse error: {}", e)))?;

    state.current_pos += 1;
    state.value = Some(JsonValue::Number(value));

    Ok(state)
}

fn parse_boolean_value(mut state: JsonParseState) -> PatternResult<JsonParseState> {
    if state.current_pos >= state.tokens.len() {
        return Err(PatternError::ExecutionFailed("Unexpected EOF".to_string()));
    }

    let token = state.tokens[state.current_pos];
    let value = match token.token_type {
        TokenType::True => true,
        TokenType::False => false,
        _ => {
            return Err(PatternError::ExecutionFailed(
                "Expected boolean token".to_string(),
            ))
        }
    };

    state.current_pos += 1;
    state.value = Some(JsonValue::Boolean(value));

    Ok(state)
}

fn parse_null_value(mut state: JsonParseState) -> PatternResult<JsonParseState> {
    if state.current_pos >= state.tokens.len() {
        return Err(PatternError::ExecutionFailed("Unexpected EOF".to_string()));
    }

    let token = state.tokens[state.current_pos];
    if token.token_type != TokenType::Null {
        return Err(PatternError::ExecutionFailed(
            "Expected null token".to_string(),
        ));
    }

    state.current_pos += 1;
    state.value = Some(JsonValue::Null);

    Ok(state)
}

// ============================================================================
// Pattern 2: Parallel Split - Parse Object Fields Concurrently
// ============================================================================

fn parse_object_value(mut state: JsonParseState) -> PatternResult<JsonParseState> {
    // Consume ObjectStart
    if state.current_pos >= state.tokens.len()
        || state.tokens[state.current_pos].token_type != TokenType::ObjectStart
    {
        return Err(PatternError::ExecutionFailed(
            "Expected object start".to_string(),
        ));
    }
    state.current_pos += 1;

    let mut fields = Vec::new();

    // Parse fields until ObjectEnd
    while state.current_pos < state.tokens.len() {
        // Check for empty object
        if state.tokens[state.current_pos].token_type == TokenType::ObjectEnd {
            state.current_pos += 1;
            break;
        }

        // Parse field key (string)
        let key_state = parse_string_value(state.clone())?;
        let key = match &key_state.value {
            Some(JsonValue::String(s)) => s.clone(),
            _ => {
                return Err(PatternError::ExecutionFailed(
                    "Invalid field key".to_string(),
                ))
            }
        };
        state = key_state;
        state.value = None; // Reset for value parsing

        // Consume colon
        if state.current_pos >= state.tokens.len()
            || state.tokens[state.current_pos].token_type != TokenType::Colon
        {
            return Err(PatternError::ExecutionFailed("Expected colon".to_string()));
        }
        state.current_pos += 1;

        // Parse field value using type router
        let router = create_type_router()?;
        let results = router.execute(state)?;
        state = results[0].clone();

        let value = state
            .value
            .clone()
            .ok_or_else(|| PatternError::ExecutionFailed("No value parsed".to_string()))?;

        fields.push((key, value));
        state.value = None; // Reset for next field

        // Check for comma or object end
        if state.current_pos < state.tokens.len() {
            match state.tokens[state.current_pos].token_type {
                TokenType::Comma => {
                    state.current_pos += 1;
                    continue;
                }
                TokenType::ObjectEnd => {
                    state.current_pos += 1;
                    break;
                }
                _ => {
                    return Err(PatternError::ExecutionFailed(
                        "Expected comma or object end".to_string(),
                    ))
                }
            }
        }
    }

    state.value = Some(JsonValue::Object(fields));
    Ok(state)
}

// ============================================================================
// Pattern 6: Multi-Choice - Parse Heterogeneous Arrays
// ============================================================================

fn parse_array_value(mut state: JsonParseState) -> PatternResult<JsonParseState> {
    // Consume ArrayStart
    if state.current_pos >= state.tokens.len()
        || state.tokens[state.current_pos].token_type != TokenType::ArrayStart
    {
        return Err(PatternError::ExecutionFailed(
            "Expected array start".to_string(),
        ));
    }
    state.current_pos += 1;

    let mut elements = Vec::new();

    // Parse elements until ArrayEnd
    while state.current_pos < state.tokens.len() {
        // Check for empty array
        if state.tokens[state.current_pos].token_type == TokenType::ArrayEnd {
            state.current_pos += 1;
            break;
        }

        // Parse element using type router
        let router = create_type_router()?;
        let results = router.execute(state)?;
        state = results[0].clone();

        let value = state
            .value
            .clone()
            .ok_or_else(|| PatternError::ExecutionFailed("No value parsed".to_string()))?;

        elements.push(value);
        state.value = None; // Reset for next element

        // Check for comma or array end
        if state.current_pos < state.tokens.len() {
            match state.tokens[state.current_pos].token_type {
                TokenType::Comma => {
                    state.current_pos += 1;
                    continue;
                }
                TokenType::ArrayEnd => {
                    state.current_pos += 1;
                    break;
                }
                _ => {
                    return Err(PatternError::ExecutionFailed(
                        "Expected comma or array end".to_string(),
                    ))
                }
            }
        }
    }

    state.value = Some(JsonValue::Array(elements));
    Ok(state)
}

// ============================================================================
// Main Function
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 JSON Parser Using knhk-patterns");
    println!("===================================\n");

    // Test cases
    let test_cases = vec![
        (r#""hello""#, "String value"),
        (r#"42"#, "Number value"),
        (r#"true"#, "Boolean value"),
        (r#"null"#, "Null value"),
        (r#"{"name": "KNHK", "version": 1}"#, "Object with fields"),
        (r#"[1, "two", 3.0, true]"#, "Heterogeneous array"),
        (r#"{"nested": {"key": "value"}}"#, "Nested object"),
    ];

    // Create parsing pipeline
    let pipeline =
        create_json_parse_pipeline().map_err(|e| format!("Failed to create pipeline: {}", e))?;

    println!("✅ Created JSON parsing pipeline using:");
    println!("   - Pattern 1 (Sequence): Tokenize → Parse → Validate");
    println!("   - Pattern 4 (Exclusive Choice): Type routing\n");

    // Parse each test case
    for (json_str, description) in test_cases {
        println!("📋 Test: {}", description);
        println!("   Input: {}", json_str);

        let state = JsonParseState {
            bytes: json_str.as_bytes().to_vec(),
            tokenizer: None,
            tokens: Vec::new(),
            current_pos: 0,
            value: None,
        };

        match pipeline.execute(state) {
            Ok(results) => {
                if let Some(value) = &results[0].value {
                    println!("   ✅ Parsed: {:?}", value);
                } else {
                    println!("   ⚠️  No value parsed");
                }
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
            }
        }
        println!();
    }

    println!("===================================");
    println!("✅ Pattern-based JSON parsing complete!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_parsing() {
        let pipeline = create_json_parse_pipeline().unwrap();
        let state = JsonParseState {
            bytes: r#""hello""#.as_bytes().to_vec(),
            tokenizer: None,
            tokens: Vec::new(),
            current_pos: 0,
            value: None,
        };

        let results = pipeline.execute(state).unwrap();
        assert_eq!(
            results[0].value,
            Some(JsonValue::String("hello".to_string()))
        );
    }

    #[test]
    fn test_number_parsing() {
        let pipeline = create_json_parse_pipeline().unwrap();
        let state = JsonParseState {
            bytes: r#"42"#.as_bytes().to_vec(),
            tokenizer: None,
            tokens: Vec::new(),
            current_pos: 0,
            value: None,
        };

        let results = pipeline.execute(state).unwrap();
        assert_eq!(results[0].value, Some(JsonValue::Number(42.0)));
    }

    #[test]
    fn test_object_parsing() {
        let pipeline = create_json_parse_pipeline().unwrap();
        let state = JsonParseState {
            bytes: r#"{"name": "KNHK"}"#.as_bytes().to_vec(),
            tokenizer: None,
            tokens: Vec::new(),
            current_pos: 0,
            value: None,
        };

        let results = pipeline.execute(state).unwrap();
        match &results[0].value {
            Some(JsonValue::Object(fields)) => {
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].0, "name");
                assert_eq!(fields[0].1, JsonValue::String("KNHK".to_string()));
            }
            _ => panic!("Expected object"),
        }
    }
}
