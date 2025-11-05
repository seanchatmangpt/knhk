// knhk-warm: Warm path operations for KNHK
// Handles operations that exceed the 8-tick hot path budget but can complete within 500ms
// Production-ready implementation with proper error handling and observability

#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;

pub mod construct8;
pub mod error;

pub use construct8::WarmPathConstruct8;
pub use error::WarmPathError;

/// Warm path operation result
#[derive(Debug, Clone)]
pub struct WarmPathResult {
    pub success: bool,
    pub latency_ms: u64,
    pub lanes_written: usize,
    pub span_id: u64,
}

impl WarmPathResult {
    pub fn new(success: bool, latency_ms: u64, lanes_written: usize, span_id: u64) -> Self {
        Self {
            success,
            latency_ms,
            lanes_written,
            span_id,
        }
    }
}

#[cfg(test)]
mod tests {
    // Tests are in tests/warm_path_test.rs
}


