// knhk-warm/src/construct8.rs
// CONSTRUCT8 Warm Path Bridge
// Routes CONSTRUCT8 operations from hot path to warm path with ≤500ms budget

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use knhk_hot::{Engine, Ir, Receipt, Run};
use crate::WarmPathResult;

#[cfg(feature = "std")]
use std::time::Instant;

/// Execute CONSTRUCT8 operation in warm path
/// 
/// This function wraps the C hot path CONSTRUCT8 implementation but
/// measures timing externally (not in hot path). The operation itself
/// performs SIMD loads, blending, and stores which exceed the 8-tick
/// hot path budget, so it's routed to warm path.
/// 
/// # Arguments
/// * `engine` - Hot path engine instance
/// * `ir` - IR with CONSTRUCT8 operation
/// * `rcpt` - Receipt to fill with provenance information
/// 
/// # Returns
/// WarmPathResult with success status, latency, and lanes written
#[cfg(feature = "std")]
pub fn execute_construct8_warm(
    engine: &Engine,
    ir: &mut Ir,
    rcpt: &mut Receipt,
) -> WarmPathResult {
    // Measure timing externally (not in hot path)
    let start = Instant::now();
    
    // Execute CONSTRUCT8 via hot path engine (C implementation)
    // Note: The C implementation itself doesn't measure timing
    let lanes_written = engine.eval_construct8(ir, rcpt);
    
    let elapsed = start.elapsed();
    let latency_ms = elapsed.as_secs_f64() * 1000.0;
    
    // Verify warm path budget (≤500ms)
    let success = latency_ms <= crate::WARM_PATH_BUDGET_MS as f64;
    
    WarmPathResult {
        success,
        latency_ms,
        lanes_written,
    }
}

/// Execute CONSTRUCT8 operation in warm path (no_std version)
/// 
/// For no_std environments, timing measurement must be done externally
/// by the caller. This version just executes the operation.
#[cfg(not(feature = "std"))]
pub fn execute_construct8_warm(
    engine: &Engine,
    ir: &mut Ir,
    rcpt: &mut Receipt,
) -> WarmPathResult {
    // Execute CONSTRUCT8 via hot path engine
    let lanes_written = engine.eval_construct8(ir, rcpt);
    
    // For no_std, caller must measure timing externally
    WarmPathResult {
        success: true,
        latency_ms: 0.0, // Caller must set this
        lanes_written,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use knhk_hot::{Aligned, Op};

    #[test]
    #[cfg(feature = "std")]
    fn test_construct8_warm_path_execution() {
        // Create test data
        let s_array = Aligned([1u64, 2u64, 3u64, 0, 0, 0, 0, 0]);
        let p_array = Aligned([10u64, 10u64, 10u64, 0, 0, 0, 0, 0]);
        let o_array = Aligned([20u64, 21u64, 22u64, 0, 0, 0, 0, 0]);
        
        let mut engine = Engine::new(
            s_array.0.as_ptr(),
            p_array.0.as_ptr(),
            o_array.0.as_ptr(),
        );
        
        // Pin run
        let run = Run {
            pred: 10,
            off: 0,
            len: 3,
        };
        engine.pin_run(run).expect("Failed to pin run");
        
        // Create CONSTRUCT8 IR
        let mut out_s = Aligned([0u64; 8]);
        let mut out_p = Aligned([0u64; 8]);
        let mut out_o = Aligned([0u64; 8]);
        
        let mut ir = Ir {
            op: Op::Construct8,
            s: 0,
            p: 10,
            o: 100,
            k: 0,
            out_S: out_s.0.as_mut_ptr(),
            out_P: out_p.0.as_mut_ptr(),
            out_O: out_o.0.as_mut_ptr(),
            out_mask: 0,
        };
        
        let mut receipt = Receipt::default();
        
        // Execute in warm path
        let result = execute_construct8_warm(&engine, &mut ir, &mut receipt);
        
        // Verify warm path execution
        assert!(result.success, "CONSTRUCT8 should complete in warm path budget");
        assert!(result.latency_ms <= crate::WARM_PATH_BUDGET_MS as f64, 
                "Latency should be ≤500ms");
        assert_eq!(result.lanes_written, 3, "Should construct 3 triples");
        assert_eq!(receipt.lanes, 3, "Receipt should have 3 lanes");
        assert_ne!(receipt.span_id, 0, "Receipt should have span ID");
    }
}

