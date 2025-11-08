//! Workflow engine constants
//!
//! Centralized constants for the workflow engine including performance
//! constraints, guard limits, and configuration defaults.

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
