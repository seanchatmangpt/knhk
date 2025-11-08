// rust/knhk-aot/src/lib.rs
// Ahead-Of-Time (AOT) Compilation Guard
// Validates IR before execution to enforce Chatman Constant (≤8 ticks)
// Includes template analysis, prebinding, and MPHF generation

// CRITICAL: Enforce proper error handling - no unwrap/expect in production code
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

// Use jemalloc for better allocation performance in AOT scenarios
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

// Explicit imports for cdylib/staticlib compatibility
use std::string::String;

#[cfg(feature = "validation")]
use knhk_validation::policy_engine::{PolicyEngine, PolicyViolation};

// Module declarations - all modules included
pub mod mphf;
pub mod pattern;
pub mod prebinding;
pub mod specialization;
pub mod template;
pub mod template_analyzer;

// Re-exports for convenience
pub use mphf::{Mphf, MphfCache};
pub use prebinding::PreboundIr;
pub use template::ConstructTemplate;

/// Hook IR validation result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    Valid,
    ExceedsTickBudget,
    InvalidOperation,
    InvalidRunLength,
}

/// AOT guard for validating hooks before execution
pub struct AotGuard;

impl AotGuard {
    /// Validate hook IR before execution
    /// Returns Ok(()) if valid, Err(ValidationResult) if invalid
    pub fn validate_ir(op: u32, run_len: u64, k: u64) -> Result<(), ValidationResult> {
        #[cfg(feature = "validation")]
        {
            // Use policy engine for guard constraint validation
            let policy_engine = PolicyEngine::new();
            if let Err(violation) = policy_engine.validate_guard_constraint(run_len) {
                return Err(ValidationResult::InvalidRunLength);
            }
        }
        #[cfg(not(feature = "validation"))]
        {
            // Check run length ≤ 8 (Chatman Constant constraint)
            if run_len > 8 {
                return Err(ValidationResult::InvalidRunLength);
            }
        }

        // Validate operation is in hot path set
        if !Self::is_hot_path_op(op) {
            return Err(ValidationResult::InvalidOperation);
        }

        // Check operation-specific constraints
        match op {
            // ASK operations - always valid if run_len ≤ 8
            1 | 3 | 7 => Ok(()),
            // COUNT operations - check k threshold
            2 | 5 | 6 | 9 | 10 | 11 => {
                if k > run_len {
                    return Err(ValidationResult::ExceedsTickBudget);
                }
                Ok(())
            }
            // UNIQUE - run_len must be ≤ 1
            8 => {
                if run_len > 1 {
                    return Err(ValidationResult::ExceedsTickBudget);
                }
                Ok(())
            }
            // COMPARE operations - always valid
            12..=16 => Ok(()),
            // CONSTRUCT8 - check template size ≤ 8
            32 => {
                if run_len > 8 {
                    return Err(ValidationResult::ExceedsTickBudget);
                }
                Ok(())
            }
            _ => Err(ValidationResult::InvalidOperation),
        }
    }

    /// Check if operation is in hot path set
    fn is_hot_path_op(op: u32) -> bool {
        matches!(
            op,
            1 | 2 | 3 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 32
        )
    }

    /// Get validation error message
    pub fn error_message(result: &ValidationResult) -> String {
        match result {
            ValidationResult::Valid => String::from("Valid"),
            ValidationResult::ExceedsTickBudget => String::from("Operation exceeds 8-tick budget"),
            ValidationResult::InvalidOperation => String::from("Operation not in hot path set"),
            ValidationResult::InvalidRunLength => String::from("Run length must be ≤ 8"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_ask_sp() {
        assert!(AotGuard::validate_ir(1, 8, 0).is_ok());
    }

    #[test]
    fn test_validate_invalid_run_length() {
        assert_eq!(
            AotGuard::validate_ir(1, 9, 0),
            Err(ValidationResult::InvalidRunLength)
        );
    }

    #[test]
    fn test_validate_unique_sp() {
        assert!(AotGuard::validate_ir(8, 1, 0).is_ok());
        assert_eq!(
            AotGuard::validate_ir(8, 2, 0),
            Err(ValidationResult::ExceedsTickBudget)
        );
    }
}
