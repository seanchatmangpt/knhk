// Pattern extraction for DoD validator
// Extracts code patterns (unwrap, expect, TODO, etc.) and converts to SoA arrays for KNHK hot path

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Pattern type for code validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternType {
    Unwrap = 1,
    Expect = 2,
    Todo = 3,
    Placeholder = 4,
    Panic = 5,
    Result = 6,
}

impl PatternType {
    pub fn as_u32(&self) -> u32 {
        match self {
            PatternType::Unwrap => 1,
            PatternType::Expect => 2,
            PatternType::Todo => 3,
            PatternType::Placeholder => 4,
            PatternType::Panic => 5,
            PatternType::Result => 6,
        }
    }
}

/// Pattern match found in code
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_type: PatternType,
    pub pattern_hash: u64,
    pub line: u32,
    pub column: u32,
    pub context: String,
}

/// Pattern extraction result
#[derive(Debug, Clone)]
pub struct PatternExtractionResult {
    pub patterns: Vec<PatternMatch>,
    pub code_hash: u64,
    pub file_path: std::path::PathBuf,
}

/// Pattern extractor for code files
pub struct PatternExtractor;

impl PatternExtractor {
    /// Create a new pattern extractor
    pub fn new() -> Self {
        Self
    }

    /// Extract patterns from code file
    /// Returns patterns converted to SoA arrays for KNHK hot path
    pub fn extract_from_file<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<PatternExtractionResult, String> {
        let path = file_path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

        self.extract_from_content(&content, path)
    }

    /// Extract patterns from code content
    pub fn extract_from_content<P: AsRef<Path>>(
        &self,
        content: &str,
        file_path: P,
    ) -> Result<PatternExtractionResult, String> {
        let mut patterns = Vec::new();
        let code_hash = Self::hash_content(content);

        // Extract patterns line by line
        for (line_num, line) in content.lines().enumerate() {
            let line_num = (line_num + 1) as u32;

            // Check for unwrap() pattern
            if let Some(col) = Self::find_pattern(line, ".unwrap()") {
                patterns.push(PatternMatch {
                    pattern_type: PatternType::Unwrap,
                    pattern_hash: Self::hash_pattern(".unwrap()"),
                    line: line_num,
                    column: col as u32,
                    context: line.trim().to_string(),
                });
            }

            // Check for expect() pattern
            if let Some(col) = Self::find_pattern(line, ".expect(") {
                patterns.push(PatternMatch {
                    pattern_type: PatternType::Expect,
                    pattern_hash: Self::hash_pattern(".expect("),
                    line: line_num,
                    column: col as u32,
                    context: line.trim().to_string(),
                });
            }

            // Check for TODO comment
            if let Some(col) = Self::find_pattern_case_insensitive(line, "TODO") {
                patterns.push(PatternMatch {
                    pattern_type: PatternType::Todo,
                    pattern_hash: Self::hash_pattern("TODO"),
                    line: line_num,
                    column: col as u32,
                    context: line.trim().to_string(),
                });
            }

            // Check for placeholder comment
            if Self::find_pattern_case_insensitive(line, "placeholder").is_some() ||
               Self::find_pattern_case_insensitive(line, "In production").is_some() {
                patterns.push(PatternMatch {
                    pattern_type: PatternType::Placeholder,
                    pattern_hash: Self::hash_pattern("placeholder"),
                    line: line_num,
                    column: 0,
                    context: line.trim().to_string(),
                });
            }

            // Check for panic! pattern
            if let Some(col) = Self::find_pattern(line, "panic!") {
                patterns.push(PatternMatch {
                    pattern_type: PatternType::Panic,
                    pattern_hash: Self::hash_pattern("panic!"),
                    line: line_num,
                    column: col as u32,
                    context: line.trim().to_string(),
                });
            }

            // Check for Result<T, E> pattern (positive validation)
            if Self::find_pattern(line, "Result<").is_some() {
                patterns.push(PatternMatch {
                    pattern_type: PatternType::Result,
                    pattern_hash: Self::hash_pattern("Result<"),
                    line: line_num,
                    column: 0,
                    context: line.trim().to_string(),
                });
            }
        }

        Ok(PatternExtractionResult {
            patterns,
            code_hash,
            file_path: file_path.as_ref().to_path_buf(),
        })
    }

    /// Convert patterns to SoA arrays for KNHK hot path
    /// Returns (S array, P array, O array) with up to 8 patterns
    pub fn to_soa_arrays(
        &self,
        extraction: &PatternExtractionResult,
        pattern_type: PatternType,
    ) -> Result<([u64; 8], [u64; 8], [u64; 8]), String> {
        // Filter patterns by type
        let relevant_patterns: Vec<_> = extraction
            .patterns
            .iter()
            .filter(|p| p.pattern_type == pattern_type)
            .take(8) // Guard: max_run_len ≤ 8
            .collect();

        if relevant_patterns.len() > 8 {
            return Err(format!(
                "Pattern count {} exceeds max_run_len 8",
                relevant_patterns.len()
            ));
        }

        let mut s_array = [0u64; 8];
        let mut p_array = [0u64; 8];
        let mut o_array = [0u64; 8];

        for (i, pattern) in relevant_patterns.iter().enumerate() {
            s_array[i] = pattern.pattern_hash;
            p_array[i] = pattern_type.as_u32() as u64;
            o_array[i] = extraction.code_hash;
        }

        Ok((s_array, p_array, o_array))
    }

    /// Find pattern in line (case-sensitive)
    fn find_pattern(line: &str, pattern: &str) -> Option<usize> {
        line.find(pattern)
    }

    /// Find pattern in line (case-insensitive)
    fn find_pattern_case_insensitive(line: &str, pattern: &str) -> Option<usize> {
        let line_lower = line.to_lowercase();
        let pattern_lower = pattern.to_lowercase();
        line_lower.find(&pattern_lower)
    }

    /// Hash content using FNV-1a (consistent with KNHK)
    fn hash_content(content: &str) -> u64 {
        const FNV_OFFSET_BASIS: u64 = 1469598103934665603u64;
        const FNV_PRIME: u64 = 1099511628211u64;

        let mut hash = FNV_OFFSET_BASIS;
        for byte in content.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }

    /// Hash pattern string using FNV-1a
    fn hash_pattern(pattern: &str) -> u64 {
        Self::hash_content(pattern)
    }
}

impl Default for PatternExtractor {
    fn default() -> Self {
        Self::new()
    }
}

