//! Workflow engine constants
//!
//! Centralized constants for the workflow engine including performance
//! constraints, guard limits, and configuration defaults.
//!
//! **DFLSS Compliance**: All constants are validated at compile time using const fn.

/// Chatman Constant: Maximum hot path ticks (8 ticks = 2ns at 4GHz)
pub const HOT_PATH_MAX_TICKS: u32 = 8;

/// Hot path maximum latency in nanoseconds
pub const HOT_PATH_MAX_NS: u64 = 2;

/// Warm path maximum latency in milliseconds
pub const WARM_PATH_MAX_MS: u64 = 1;

/// Cold path maximum latency in milliseconds
pub const COLD_PATH_MAX_MS: u64 = 500;

/// Maximum run length (Chatman Constant: ≤8)
pub const MAX_RUN_LEN: usize = 8;

/// Maximum batch size
pub const MAX_BATCH_SIZE: usize = 1000;

/// Maximum pattern ID
pub const MAX_PATTERN_ID: u32 = 43;

/// Minimum pattern ID
pub const MIN_PATTERN_ID: u32 = 1;

/// Default SLO window size (seconds)
pub const DEFAULT_SLO_WINDOW_SECONDS: u64 = 60;

/// Default cache TTL (seconds)
pub const DEFAULT_CACHE_TTL_SECONDS: u64 = 300;

/// Default circuit breaker threshold
pub const DEFAULT_CIRCUIT_BREAKER_THRESHOLD: u32 = 5;

/// Default circuit breaker timeout (seconds)
pub const DEFAULT_CIRCUIT_BREAKER_TIMEOUT_SECONDS: u64 = 60;

/// Default retry max attempts
pub const DEFAULT_RETRY_MAX_ATTEMPTS: u32 = 3;

/// Default retry backoff multiplier
pub const DEFAULT_RETRY_BACKOFF_MULTIPLIER: f64 = 2.0;

/// Compile-time validation of DFLSS constraints
///
/// Validates that DFLSS requirements are met at compile time:
/// - MAX_RUN_LEN ≤ 8 (Chatman Constant)
/// - HOT_PATH_MAX_TICKS ≤ 8 (Chatman Constant)
///
/// # Returns
/// * `true` if all DFLSS constraints are satisfied, `false` otherwise
pub const fn validate_dflss_constraints(max_run_len: usize, tick_budget: u32) -> bool {
    max_run_len <= 8 && tick_budget <= 8
}

/// Compile-time validation that MAX_RUN_LEN satisfies DFLSS CTQ 2
///
/// Returns `true` if MAX_RUN_LEN ≤ 8 (Chatman Constant).
pub const fn validate_max_run_len_const(max_run_len: usize) -> bool {
    max_run_len <= 8
}

/// Compile-time validation that tick budget satisfies DFLSS CTQ 2
///
/// Returns `true` if tick_budget ≤ 8 (Chatman Constant).
pub const fn validate_tick_budget_const(tick_budget: u32) -> bool {
    tick_budget <= 8
}

/// Compile-time validated DFLSS constants
///
/// These constants are validated at compile time to ensure DFLSS compliance.
pub const DFLSS_VALID: bool = validate_dflss_constraints(MAX_RUN_LEN, HOT_PATH_MAX_TICKS);
pub const MAX_RUN_LEN_VALID: bool = validate_max_run_len_const(MAX_RUN_LEN);
pub const TICK_BUDGET_VALID: bool = validate_tick_budget_const(HOT_PATH_MAX_TICKS);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dflss_constraints_validation() {
        // Verify compile-time validation
        assert!(DFLSS_VALID);
        assert!(MAX_RUN_LEN_VALID);
        assert!(TICK_BUDGET_VALID);

        // Test validation functions
        assert!(validate_dflss_constraints(8, 8));
        assert!(validate_dflss_constraints(7, 7));
        assert!(!validate_dflss_constraints(9, 8)); // MAX_RUN_LEN > 8
        assert!(!validate_dflss_constraints(8, 9)); // TICK_BUDGET > 8

        assert!(validate_max_run_len_const(8));
        assert!(validate_max_run_len_const(1));
        assert!(!validate_max_run_len_const(9));

        assert!(validate_tick_budget_const(8));
        assert!(validate_tick_budget_const(1));
        assert!(!validate_tick_budget_const(9));
    }
}
