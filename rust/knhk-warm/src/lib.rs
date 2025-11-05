// knhk-warm v0.1.0 - Warm Path Operations
// Warm path: ≤500ms budget for emit operations (CONSTRUCT8)
// Hot path: ≤8 ticks for query operations (ASK, COUNT, etc.)
//
// Separation: Hot path handles queries, warm path handles emit operations
// that exceed the 8-tick budget due to SIMD loads, blending, and stores.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::time::Instant;

pub mod construct8;

pub use construct8::execute_construct8_warm;

/// Warm path budget: ≤500ms (p95)
pub const WARM_PATH_BUDGET_MS: u64 = 500;

/// Warm path metrics
#[cfg(feature = "std")]
#[derive(Debug, Clone)]
pub struct WarmPathMetrics {
    pub operation_count: u64,
    pub latency_p50_ms: f64,
    pub latency_p95_ms: f64,
    pub latency_p99_ms: f64,
}

#[cfg(feature = "std")]
impl Default for WarmPathMetrics {
    fn default() -> Self {
        Self {
            operation_count: 0,
            latency_p50_ms: 0.0,
            latency_p95_ms: 0.0,
            latency_p99_ms: 0.0,
        }
    }
}

/// Warm path execution result
#[derive(Debug, Clone)]
pub struct WarmPathResult {
    pub success: bool,
    pub latency_ms: f64,
    pub lanes_written: usize,
}

