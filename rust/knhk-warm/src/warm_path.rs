// Warm path operations
// CONSTRUCT8 execution moved from hot path (exceeds 8-tick budget)

use crate::ffi::{Op, HotContext, HotHookIr, HotReceipt};

/// Warm path execution result
#[derive(Debug, Clone)]
pub struct WarmPathResult {
    /// Number of triples written
    pub lanes_written: i32,
    /// Receipt from execution
    pub receipt: WarmReceipt,
    /// Execution latency in microseconds
    pub latency_us: u64,
}

/// Warm path receipt (similar to hot path receipt)
#[derive(Debug, Clone, Default)]
pub struct WarmReceipt {
    /// Number of lanes written
    pub lanes: u32,
    /// Span ID for OTEL correlation
    pub span_id: u64,
    /// Provenance hash
    pub a_hash: u64,
}

/// Execute CONSTRUCT8 in warm path (≤500µs budget, ≤1ms SLO)
/// 
/// This function routes CONSTRUCT8 operations from hot path to warm path
/// since CONSTRUCT8 performs emit work (SIMD loads, blending, stores) which
/// exceeds the 8-tick hot path budget.
pub fn execute_construct8(
    ctx: &HotContext,
    ir: &mut HotHookIr,
) -> Result<WarmPathResult, WarmPathError> {
    // Validate CONSTRUCT8 operation
    if !matches!(ir.op, Op::Construct8) {
        return Err(WarmPathError::InvalidOperation);
    }

    // Validate output buffers
    if ir.out_S.is_null() || ir.out_P.is_null() || ir.out_O.is_null() {
        return Err(WarmPathError::InvalidOutputBuffers);
    }

    // Execute CONSTRUCT8 via hot path FFI (same implementation, different budget)
    // Note: This still uses the C hot path implementation, but we're routing
    // it through warm path for timing purposes
    let mut receipt = HotReceipt::default();
    let lanes_written = unsafe {
        crate::ffi::knhk_hot_eval_construct8(
            ctx as *const HotContext,
            ir as *mut HotHookIr,
            &mut receipt,
        )
    };

    // Convert hot path receipt to warm path receipt
    let warm_receipt = WarmReceipt {
        lanes: receipt.lanes,
        span_id: receipt.span_id,
        a_hash: receipt.a_hash,
    };

    // Record execution time (warm path has ≤500µs budget, ≤1ms SLO)
    // Note: Actual timing measurement happens at Rust level, not in C hot path
    let latency_us = 0; // Will be measured externally

    Ok(WarmPathResult {
        lanes_written,
        receipt: warm_receipt,
        latency_us,
    })
}

/// Warm path error types
#[derive(Debug, Clone, PartialEq)]
pub enum WarmPathError {
    InvalidInput(String),
    InvalidOperation,
    InvalidOutputBuffers,
    ExecutionFailed,
    QueryParseError(String),
    QueryExecutionError(String),
    PathSelectionError(String),
    CacheError(String),
}

impl core::fmt::Display for WarmPathError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WarmPathError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            WarmPathError::InvalidOperation => write!(f, "Invalid operation for warm path"),
            WarmPathError::InvalidOutputBuffers => write!(f, "Invalid output buffers"),
            WarmPathError::ExecutionFailed => write!(f, "Warm path execution failed"),
            WarmPathError::QueryParseError(msg) => write!(f, "Query parse error: {}", msg),
            WarmPathError::QueryExecutionError(msg) => write!(f, "Query execution error: {}", msg),
            WarmPathError::PathSelectionError(msg) => write!(f, "Path selection error: {}", msg),
            WarmPathError::CacheError(msg) => write!(f, "Cache error: {}", msg),
        }
    }
}

