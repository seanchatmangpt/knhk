// knhk-aot: Code specialization
// Generates specialized code paths for common patterns

#![no_std]
extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;

/// Specialization pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecializationPattern {
    /// All non-zero pattern (skip mask generation)
    AllNonZero,
    /// All zero pattern (early return)
    AllZero,
    /// Sparse pattern (optimized sparse computation)
    Sparse,
    /// Dense pattern (standard computation)
    Dense,
}

/// Specialization result
#[derive(Debug, Clone)]
pub struct SpecializationResult {
    /// Detected pattern
    pub pattern: SpecializationPattern,
    /// Function name for specialized variant
    pub function_name: String,
    /// Optimization hints
    pub hints: Vec<String>,
}

/// Detect input pattern for specialization
pub fn detect_pattern(subjects: &[u64; 8], len: usize) -> SpecializationPattern {
    if len == 0 {
        return SpecializationPattern::AllZero;
    }
    
    let mut zero_count = 0;
    let mut non_zero_count = 0;
    
    for i in 0..len {
        if subjects[i] == 0 {
            zero_count += 1;
        } else {
            non_zero_count += 1;
        }
    }
    
    if zero_count == len {
        SpecializationPattern::AllZero
    } else if non_zero_count == len {
        SpecializationPattern::AllNonZero
    } else if zero_count > len / 2 {
        SpecializationPattern::Sparse
    } else {
        SpecializationPattern::Dense
    }
}

/// Generate specialization result
pub fn specialize(pattern: SpecializationPattern, len: usize) -> SpecializationResult {
    let function_name = match pattern {
        SpecializationPattern::AllNonZero => format!("knhk_construct8_emit_8_all_nonzero_len{}", len),
        SpecializationPattern::AllZero => format!("knhk_construct8_emit_8_all_zero_len{}", len),
        SpecializationPattern::Sparse => format!("knhk_construct8_emit_8_sparse_len{}", len),
        SpecializationPattern::Dense => format!("knhk_construct8_emit_8_dense_len{}", len),
    };
    
    let hints = match pattern {
        SpecializationPattern::AllNonZero => {
            let mut v = Vec::new();
            v.push("Skip mask generation".to_string());
            v.push("Use all-ones mask constant".to_string());
            v
        },
        SpecializationPattern::AllZero => {
            let mut v = Vec::new();
            v.push("Early return".to_string());
            v.push("Set out_mask to 0".to_string());
            v
        },
        SpecializationPattern::Sparse => {
            let mut v = Vec::new();
            v.push("Use sparse computation path".to_string());
            v.push("Optimize for zero lanes".to_string());
            v
        },
        SpecializationPattern::Dense => {
            let mut v = Vec::new();
            v.push("Use standard computation".to_string());
            v
        },
    };
    
    SpecializationResult {
        pattern,
        function_name,
        hints,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_all_zero() {
        let subjects = [0u64; 8];
        let pattern = detect_pattern(&subjects, 8);
        assert_eq!(pattern, SpecializationPattern::AllZero);
    }

    #[test]
    fn test_detect_all_nonzero() {
        let subjects = [1u64, 2u64, 3u64, 4u64, 5u64, 6u64, 7u64, 8u64];
        let pattern = detect_pattern(&subjects, 8);
        assert_eq!(pattern, SpecializationPattern::AllNonZero);
    }
}

