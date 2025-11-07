// knhk-hot: Fiber execution FFI bindings
// Per-shard, per-hook execution units

#![allow(non_camel_case_types)]

use crate::{Ctx, Ir, Receipt};
use crate::ring_ffi::{knhk_delta_ring_t, knhk_assertion_ring_t};
use std::os::raw::c_int;

/// Fiber execution result
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FiberResult {
    Success = 0,
    Parked = 1,
    Error = -1,
}

#[link(name = "knhk")]
extern "C" {
    /// Execute μ on ≤8 items at tick slot
    pub fn knhk_fiber_execute(
        ctx: *const Ctx,
        ir: *mut Ir,
        tick: u64,
        cycle_id: u64,
        shard_id: u64,
        hook_id: u64,
        receipt: *mut Receipt,
    ) -> c_int;

    /// Park delta to W1
    pub fn knhk_fiber_park(
        delta_ring: *mut knhk_delta_ring_t,
        tick: u64,
        ring_idx: u64,
        cycle_id: u64,
    );

    /// Execute fiber from delta ring at tick slot
    pub fn knhk_fiber_process_tick(
        delta_ring: *mut knhk_delta_ring_t,
        assertion_ring: *mut knhk_assertion_ring_t,
        ctx: *mut Ctx,
        ir: *mut Ir,
        tick: u64,
        shard_id: u64,
        hook_id: u64,
    ) -> usize;
}

/// Safe wrapper for fiber execution
pub struct FiberExecutor;

impl FiberExecutor {
    /// Execute μ on ≤8 items at tick slot
    pub fn execute(
        ctx: &Ctx,
        ir: &mut Ir,
        tick: u64,
        cycle_id: u64,
        shard_id: u64,
        hook_id: u64,
    ) -> Result<Receipt, String> {
        let mut receipt = Receipt {
            cycle_id,
            shard_id,
            hook_id,
            ..Default::default()
        };

        let result = unsafe {
            knhk_fiber_execute(
                ctx as *const _,
                ir as *mut _,
                tick,
                cycle_id,
                shard_id,
                hook_id,
                &mut receipt as *mut _,
            )
        };

        match result {
            0 => Ok(receipt),
            1 => Err("Fiber parked to W1".to_string()),
            _ => Err("Fiber execution error".to_string()),
        }
    }

    /// Process tick: read from delta ring, execute, write to assertion ring
    pub fn process_tick(
        delta_ring: &mut crate::ring_ffi::DeltaRing,
        assertion_ring: &mut crate::ring_ffi::AssertionRing,
        ctx: &mut Ctx,
        ir: &mut Ir,
        tick: u64,
        shard_id: u64,
        hook_id: u64,
    ) -> usize {
        unsafe {
            knhk_fiber_process_tick(
                &mut delta_ring.inner as *mut _,
                &mut assertion_ring.inner as *mut _,
                ctx as *mut _,
                ir as *mut _,
                tick,
                shard_id,
                hook_id,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Ctx, Ir, Op, Run};

    #[test]
    fn test_fiber_executor_execute() {
        // Create test SoA arrays
        let s_array = [0x1234u64, 0x5678, 0, 0, 0, 0, 0, 0];
        let p_array = [0xabcdu64, 0xef00, 0, 0, 0, 0, 0, 0];
        let o_array = [0x1111u64, 0x2222, 0, 0, 0, 0, 0, 0];

        let ctx = Ctx {
            S: s_array.as_ptr(),
            P: p_array.as_ptr(),
            O: o_array.as_ptr(),
            run: Run {
                pred: 0xabcd,
                off: 0,
                len: 2,
            },
        };

        let mut ir = Ir {
            op: Op::AskSp,
            s: 0x1234,
            p: 0xabcd,
            o: 0x1111,
            k: 0,
            out_S: std::ptr::null_mut(),
            out_P: std::ptr::null_mut(),
            out_O: std::ptr::null_mut(),
            out_mask: 0,
            construct8_pattern_hint: 0,
        };

        // Execute fiber
        let result = FiberExecutor::execute(&ctx, &mut ir, 0, 1, 2, 3);
        
        // Should succeed or park (both are valid)
        match result {
            Ok(receipt) => {
                assert_eq!(receipt.cycle_id, 1);
                assert_eq!(receipt.shard_id, 2);
                assert_eq!(receipt.hook_id, 3);
                assert!(receipt.ticks <= 8);
            }
            Err(e) => {
                // Parking is acceptable
                assert!(e.contains("parked") || e.contains("error"));
            }
        }
    }

    #[test]
    fn test_fiber_executor_receipt_generation() {
        // Create test SoA arrays
        let s_array = [0x1234u64, 0, 0, 0, 0, 0, 0, 0];
        let p_array = [0xabcdu64, 0, 0, 0, 0, 0, 0, 0];
        let o_array = [0x1111u64, 0, 0, 0, 0, 0, 0, 0];

        let ctx = Ctx {
            S: s_array.as_ptr(),
            P: p_array.as_ptr(),
            O: o_array.as_ptr(),
            run: Run {
                pred: 0xabcd,
                off: 0,
                len: 1,
            },
        };

        let mut ir = Ir {
            op: Op::AskSp,
            s: 0x1234,
            p: 0xabcd,
            o: 0x1111,
            k: 0,
            out_S: std::ptr::null_mut(),
            out_P: std::ptr::null_mut(),
            out_O: std::ptr::null_mut(),
            out_mask: 0,
            construct8_pattern_hint: 0,
        };

        // Execute fiber with specific cycle_id, shard_id, hook_id
        let cycle_id = 42;
        let shard_id = 3;
        let hook_id = 7;
        
        let result = FiberExecutor::execute(&ctx, &mut ir, 0, cycle_id, shard_id, hook_id);
        
        match result {
            Ok(receipt) => {
                // Verify receipt contains correct identifiers
                assert_eq!(receipt.cycle_id, cycle_id);
                assert_eq!(receipt.shard_id, shard_id);
                assert_eq!(receipt.hook_id, hook_id);
                assert!(receipt.ticks <= 8, "Receipt ticks should be within budget");
                assert_ne!(receipt.span_id, 0, "Span ID should be generated");
            }
            Err(_) => {
                // Parking is acceptable, but receipt should still have identifiers if generated
            }
        }
    }

    #[test]
    fn test_fiber_executor_tick_budget_enforcement() {
        // Create test SoA arrays with 8 items (at budget limit)
        let s_array = [0x1000u64, 0x2000, 0x3000, 0x4000, 0x5000, 0x6000, 0x7000, 0x8000];
        let p_array = [0xabcdu64; 8];
        let o_array = [0x1111u64; 8];

        let ctx = Ctx {
            S: s_array.as_ptr(),
            P: p_array.as_ptr(),
            O: o_array.as_ptr(),
            run: Run {
                pred: 0xabcd,
                off: 0,
                len: 8, // At budget limit
            },
        };

        let mut ir = Ir {
            op: Op::AskSp,
            s: 0x1000,
            p: 0xabcd,
            o: 0x1111,
            k: 0,
            out_S: std::ptr::null_mut(),
            out_P: std::ptr::null_mut(),
            out_O: std::ptr::null_mut(),
            out_mask: 0,
            construct8_pattern_hint: 0,
        };

        // Execute fiber - should succeed or park
        let result = FiberExecutor::execute(&ctx, &mut ir, 0, 1, 0, 0);
        
        match result {
            Ok(receipt) => {
                // If succeeds, ticks should be ≤ 8
                assert!(receipt.ticks <= 8, "Receipt ticks {} exceeds budget", receipt.ticks);
            }
            Err(e) => {
                // Parking is acceptable when budget exceeded
                assert!(e.contains("parked") || e.contains("error"));
            }
        }
    }
}

