// knhk-warm v0.5.0 — Rust warm path API (≤500ms budget)
// Warm path operations: CONSTRUCT8 and other emit operations
// Law: Warm path operations exceed 8-tick hot path budget but complete in <500ms

#![allow(non_camel_case_types)]

use std::time::{Duration, Instant};

pub const WARM_PATH_THRESHOLD_MS: u64 = 500;
pub const NROWS: usize = 8;
pub const ALIGN: usize = 64;

// Re-export types from knhk-hot for warm path operations
pub use knhk_hot::*;

pub mod warm_path {
    use super::*;
    use std::time::Instant;

    /// Warm path receipt (no ticks, only latency)
    #[derive(Debug, Clone, Copy, Default)]
    pub struct WarmReceipt {
        pub lanes: u32,
        pub span_id: u64,
        pub a_hash: u64,
    }

    /// Warm path execution result
    #[derive(Debug, Clone)]
    pub struct WarmPathResult {
        pub lanes_written: usize,
        pub receipt: WarmReceipt,
        pub latency_us: u64,
    }

    /// Execute CONSTRUCT8 in warm path
    /// Returns lanes written and latency in microseconds
    /// Note: ctx should be from an Engine instance (use engine.ctx())
    pub fn execute_construct8(
        ctx: &knhk_hot::Ctx,
        ir: &mut knhk_hot::Ir,
    ) -> Result<WarmPathResult, WarmPathError> {
        // Validate CONSTRUCT8 operation
        if ir.op != knhk_hot::Op::Construct8 {
            return Err(WarmPathError::InvalidOperation);
        }

        // Validate output buffers
        if ir.out_S.is_null() || ir.out_P.is_null() || ir.out_O.is_null() {
            return Err(WarmPathError::InvalidOutputBuffers);
        }

        // Measure execution time
        let start = Instant::now();
        
        // Execute CONSTRUCT8 via batch API (knhk_eval_batch8 calls the inline function)
        let mut irs = [*ir];
        let mut rcpts = [knhk_hot::Receipt::default()];
        
        let lanes_written = unsafe {
            knhk_hot::knhk_eval_batch8(
                ctx as *const knhk_hot::Ctx,
                irs.as_mut_ptr(),
                1,
                rcpts.as_mut_ptr(),
            )
        };
        
        let receipt = if lanes_written > 0 {
            // Update ir with any changes (like out_mask)
            *ir = irs[0];
            rcpts[0]
        } else {
            return Err(WarmPathError::ExecutionFailed);
        };

        let elapsed = start.elapsed();
        let latency_us = elapsed.as_micros() as u64;

        // Verify warm path timing constraint
        let latency_ms = latency_us / 1000;
        if latency_ms >= WARM_PATH_THRESHOLD_MS {
            return Err(WarmPathError::TimingExceeded(format!(
                "Warm path operation exceeded threshold: {} ms >= {} ms",
                latency_ms, WARM_PATH_THRESHOLD_MS
            )));
        }

        // Convert hot path receipt to warm path receipt
        let warm_receipt = WarmReceipt {
            lanes: receipt.lanes,
            span_id: receipt.span_id,
            a_hash: receipt.a_hash,
        };

        Ok(WarmPathResult {
            lanes_written: if receipt.lanes > 0 { receipt.lanes as usize } else { 0 },
            receipt: warm_receipt,
            latency_us,
        })
    }

    /// Warm path error types
    #[derive(Debug, Clone, PartialEq)]
    pub enum WarmPathError {
        InvalidInput,
        InvalidOperation,
        InvalidOutputBuffers,
        ExecutionFailed,
        TimingExceeded(String),
    }

    impl std::fmt::Display for WarmPathError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                WarmPathError::InvalidInput => write!(f, "Invalid input"),
                WarmPathError::InvalidOperation => write!(f, "Invalid operation (expected CONSTRUCT8)"),
                WarmPathError::InvalidOutputBuffers => write!(f, "Invalid output buffers"),
                WarmPathError::ExecutionFailed => write!(f, "Execution failed"),
                WarmPathError::TimingExceeded(msg) => write!(f, "Timing exceeded: {}", msg),
            }
        }
    }

    impl std::error::Error for WarmPathError {}
}

// Warm path metrics
#[derive(Debug, Clone, Default)]
pub struct WarmPathMetrics {
    pub operation_count: u64,
    pub total_latency_ms: u64,
    pub max_latency_ms: u64,
}

impl WarmPathMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_operation(&mut self, latency_ms: u64) {
        self.operation_count += 1;
        self.total_latency_ms += latency_ms;
        self.max_latency_ms = self.max_latency_ms.max(latency_ms);
    }

    pub fn avg_latency_ms(&self) -> f64 {
        if self.operation_count == 0 {
            0.0
        } else {
            self.total_latency_ms as f64 / self.operation_count as f64
        }
    }
}

// Helper: Check if an operation belongs to warm path
pub fn is_warm_path_op(op: Op) -> bool {
    matches!(op, Op::Construct8)
}

// Helper: Check if an operation belongs to hot path
pub fn is_hot_path_op(op: Op) -> bool {
    matches!(
        op,
        Op::AskSp
            | Op::CountSpGe
            | Op::AskSpo
            | Op::CountSpLe
            | Op::CountSpEq
            | Op::AskOp
            | Op::UniqueSp
            | Op::CountOpGe
            | Op::CountOpLe
            | Op::CountOpEq
            | Op::CompareOEQ
            | Op::CompareOGT
            | Op::CompareOLT
            | Op::CompareOGE
            | Op::CompareOLE
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_warm_path_op() {
        assert!(is_warm_path_op(Op::Construct8));
        assert!(!is_warm_path_op(Op::AskSp));
    }

    #[test]
    fn test_is_hot_path_op() {
        assert!(is_hot_path_op(Op::AskSp));
        assert!(!is_hot_path_op(Op::Construct8));
    }

    #[test]
    fn test_warm_path_metrics() {
        let mut metrics = WarmPathMetrics::new();
        metrics.record_operation(100);
        metrics.record_operation(200);
        metrics.record_operation(150);
        
        assert_eq!(metrics.operation_count, 3);
        assert_eq!(metrics.max_latency_ms, 200);
        assert!((metrics.avg_latency_ms() - 150.0).abs() < 0.1);
    }
}