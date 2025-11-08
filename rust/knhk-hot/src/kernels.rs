// knhk-hot: Kernel dispatch FFI bindings
// Safe wrappers around C kernel dispatch table

#![allow(non_camel_case_types)]

use std::os::raw::c_int;

/// Kernel types (subset of operations for hot path)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KernelType {
    AskSp = 0,      // ASK(S,P) - hot path
    CountSpGe = 1,  // COUNT(S,P) >= k
    AskSpo = 2,     // ASK(S,P,O) - exact match
    ValidateSp = 3, // VALIDATE(S,P) - datatype validation
    UniqueSp = 4,   // UNIQUE(S,P) - single value check
    CompareO = 5,   // COMPARE O with value
}

/// Kernel dispatch table entry
#[repr(C)]
pub struct KernelDispatch {
    pub kernel_type: KernelType,
    pub execute: KernelFn,
}

/// Kernel function pointer type
/// Returns: CPU cycles consumed
pub type KernelFn = extern "C" fn(
    s_lane: *const u64,
    p_lane: *const u64,
    o_lane: *const u64,
    n_rows: usize,
    out_mask: *mut u64,
) -> u64;

#[link(name = "knhk")]
extern "C" {
    /// Get kernel dispatch table
    pub fn knhk_get_kernel_dispatch_table() -> *const KernelDispatch;

    /// Kernel implementations
    pub fn knhk_kernel_ask_sp_impl(
        s_lane: *const u64,
        p_lane: *const u64,
        o_lane: *const u64,
        n_rows: usize,
        out_mask: *mut u64,
    ) -> u64;

    pub fn knhk_kernel_count_sp_ge_impl(
        s_lane: *const u64,
        p_lane: *const u64,
        o_lane: *const u64,
        n_rows: usize,
        out_mask: *mut u64,
    ) -> u64;

    pub fn knhk_kernel_ask_spo_impl(
        s_lane: *const u64,
        p_lane: *const u64,
        o_lane: *const u64,
        n_rows: usize,
        out_mask: *mut u64,
    ) -> u64;

    pub fn knhk_kernel_validate_sp_impl(
        s_lane: *const u64,
        p_lane: *const u64,
        o_lane: *const u64,
        n_rows: usize,
        out_mask: *mut u64,
    ) -> u64;

    pub fn knhk_kernel_unique_sp_impl(
        s_lane: *const u64,
        p_lane: *const u64,
        o_lane: *const u64,
        n_rows: usize,
        out_mask: *mut u64,
    ) -> u64;

    pub fn knhk_kernel_compare_o_impl(
        s_lane: *const u64,
        p_lane: *const u64,
        o_lane: *const u64,
        n_rows: usize,
        out_mask: *mut u64,
    ) -> u64;

    /// Branchless kernel selection
    pub fn knhk_select_kernel(kernel_type: c_int) -> KernelFn;
}

/// Safe wrapper for kernel execution
pub struct KernelExecutor;

impl KernelExecutor {
    /// Execute kernel with automatic dispatch
    ///
    /// # Safety
    /// Caller must ensure:
    /// - Arrays are aligned to 64 bytes
    /// - n_rows ≤ 8
    /// - Arrays have at least n_rows elements
    ///
    /// # Returns
    /// - Ok((cycles, mask)): Cycles consumed and output mask
    /// - Err(msg): Execution error
    pub fn execute(
        kernel_type: KernelType,
        s_lane: &[u64],
        p_lane: &[u64],
        o_lane: &[u64],
        n_rows: usize,
    ) -> Result<(u64, u64), String> {
        if n_rows > 8 {
            return Err("n_rows exceeds maximum of 8".to_string());
        }

        if s_lane.len() < n_rows || p_lane.len() < n_rows || o_lane.len() < n_rows {
            return Err("Array lengths insufficient for n_rows".to_string());
        }

        let mut out_mask: u64 = 0;

        let cycles = unsafe {
            match kernel_type {
                KernelType::AskSp => knhk_kernel_ask_sp_impl(
                    s_lane.as_ptr(),
                    p_lane.as_ptr(),
                    o_lane.as_ptr(),
                    n_rows,
                    &mut out_mask,
                ),
                KernelType::CountSpGe => knhk_kernel_count_sp_ge_impl(
                    s_lane.as_ptr(),
                    p_lane.as_ptr(),
                    o_lane.as_ptr(),
                    n_rows,
                    &mut out_mask,
                ),
                KernelType::AskSpo => knhk_kernel_ask_spo_impl(
                    s_lane.as_ptr(),
                    p_lane.as_ptr(),
                    o_lane.as_ptr(),
                    n_rows,
                    &mut out_mask,
                ),
                KernelType::ValidateSp => knhk_kernel_validate_sp_impl(
                    s_lane.as_ptr(),
                    p_lane.as_ptr(),
                    o_lane.as_ptr(),
                    n_rows,
                    &mut out_mask,
                ),
                KernelType::UniqueSp => knhk_kernel_unique_sp_impl(
                    s_lane.as_ptr(),
                    p_lane.as_ptr(),
                    o_lane.as_ptr(),
                    n_rows,
                    &mut out_mask,
                ),
                KernelType::CompareO => knhk_kernel_compare_o_impl(
                    s_lane.as_ptr(),
                    p_lane.as_ptr(),
                    o_lane.as_ptr(),
                    n_rows,
                    &mut out_mask,
                ),
            }
        };

        Ok((cycles, out_mask))
    }

    /// Execute kernel via branchless dispatch
    ///
    /// Uses C dispatch table for zero branch mispredicts
    pub fn execute_dispatch(
        kernel_type: KernelType,
        s_lane: &[u64],
        p_lane: &[u64],
        o_lane: &[u64],
        n_rows: usize,
    ) -> Result<(u64, u64), String> {
        if n_rows > 8 {
            return Err("n_rows exceeds maximum of 8".to_string());
        }

        if s_lane.len() < n_rows || p_lane.len() < n_rows || o_lane.len() < n_rows {
            return Err("Array lengths insufficient for n_rows".to_string());
        }

        let mut out_mask: u64 = 0;

        let kernel_fn = unsafe { knhk_select_kernel(kernel_type as c_int) };

        let cycles = kernel_fn(
            s_lane.as_ptr(),
            p_lane.as_ptr(),
            o_lane.as_ptr(),
            n_rows,
            &mut out_mask,
        );

        Ok((cycles, out_mask))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_type_values() {
        assert_eq!(KernelType::AskSp as i32, 0);
        assert_eq!(KernelType::CountSpGe as i32, 1);
        assert_eq!(KernelType::AskSpo as i32, 2);
        assert_eq!(KernelType::ValidateSp as i32, 3);
        assert_eq!(KernelType::UniqueSp as i32, 4);
        assert_eq!(KernelType::CompareO as i32, 5);
    }

    #[test]
    fn test_kernel_executor_bounds_check() {
        let s = [1u64, 2, 3, 4, 5, 6, 7, 8];
        let p = [10u64, 20, 30, 40, 50, 60, 70, 80];
        let o = [100u64, 200, 300, 400, 500, 600, 700, 800];

        // Test max allowed size
        let result = KernelExecutor::execute(KernelType::AskSp, &s, &p, &o, 8);
        assert!(result.is_ok());

        // Test exceeds bounds
        let result = KernelExecutor::execute(KernelType::AskSp, &s, &p, &o, 9);
        assert!(result.is_err());
    }

    #[test]
    fn test_kernel_executor_array_length_check() {
        let s = [1u64, 2];
        let p = [10u64, 20];
        let o = [100u64, 200];

        // Arrays too short for requested n_rows
        let result = KernelExecutor::execute(KernelType::AskSp, &s, &p, &o, 3);
        assert!(result.is_err());
    }
}
