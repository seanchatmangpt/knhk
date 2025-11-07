// knhk-aot: Pattern detection
// Detects input patterns for optimization hints

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;

/// Input pattern analysis
#[derive(Debug, Clone)]
pub struct PatternAnalysis {
    /// Zero-position bitmask (bit i = 1 if subject[i] == 0)
    pub zero_mask: u8,
    /// Non-zero count
    pub non_zero_count: usize,
    /// Zero count
    pub zero_count: usize,
    /// Pattern type
    pub pattern_type: PatternType,
}

/// Pattern type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    AllZero,
    AllNonZero,
    Sparse,
    Dense,
}

/// Analyze input pattern
pub fn analyze_pattern(subjects: &[u64; 8], len: usize) -> PatternAnalysis {
    let mut zero_mask = 0u8;
    let mut zero_count = 0;
    let mut non_zero_count = 0;

    for (i, item) in subjects.iter().enumerate().take(len) {
        if *item == 0 {
            zero_mask |= 1 << i;
            zero_count += 1;
        } else {
            non_zero_count += 1;
        }
    }
    
    let pattern_type = if zero_count == len {
        PatternType::AllZero
    } else if non_zero_count == len {
        PatternType::AllNonZero
    } else if zero_count > len / 2 {
        PatternType::Sparse
    } else {
        PatternType::Dense
    };
    
    PatternAnalysis {
        zero_mask,
        non_zero_count,
        zero_count,
        pattern_type,
    }
}

/// Generate optimization hints from pattern
pub fn generate_hints(analysis: &PatternAnalysis) -> Vec<String> {
    let mut hints = Vec::new();
    
    match analysis.pattern_type {
        PatternType::AllZero => {
            hints.push("Early return: all subjects are zero".to_string());
            hints.push("Set out_mask to 0".to_string());
        }
        PatternType::AllNonZero => {
            hints.push("Skip mask generation: all subjects are non-zero".to_string());
            hints.push("Use all-ones mask constant".to_string());
        }
        PatternType::Sparse => {
            hints.push("Use sparse computation path".to_string());
            hints.push(format!("Optimize for {} zero lanes", analysis.zero_count));
        }
        PatternType::Dense => {
            hints.push("Use standard computation".to_string());
        }
    }
    
    hints.push(format!("Zero mask: 0x{:02x}", analysis.zero_mask));
    
    hints
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_all_zero() {
        let subjects = [0u64; 8];
        let analysis = analyze_pattern(&subjects, 8);
        
        assert_eq!(analysis.zero_mask, 0xFF);
        assert_eq!(analysis.zero_count, 8);
        assert_eq!(analysis.pattern_type, PatternType::AllZero);
    }

    #[test]
    fn test_analyze_all_nonzero() {
        let subjects = [1u64, 2u64, 3u64, 4u64, 5u64, 6u64, 7u64, 8u64];
        let analysis = analyze_pattern(&subjects, 8);
        
        assert_eq!(analysis.zero_mask, 0x00);
        assert_eq!(analysis.non_zero_count, 8);
        assert_eq!(analysis.pattern_type, PatternType::AllNonZero);
    }
}

