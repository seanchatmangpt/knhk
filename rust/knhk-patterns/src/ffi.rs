// rust/knhk-patterns/src/ffi.rs
// FFI bindings to C workflow patterns (knhk-hot)

use std::ffi::CStr;
use std::os::raw::{c_char, c_uint, c_void};

// ============================================================================
// C Types
// ============================================================================

#[repr(C)]
pub struct PatternContext {
    pub data: *mut u64,
    pub len: u32,
    pub metadata: u64,
}

#[repr(C)]
pub struct PatternResult {
    pub success: bool,
    pub branches: u32,
    pub result: u64,
    pub error: *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PatternType {
    Sequence = 1,
    ParallelSplit = 2,
    Synchronization = 3,
    ExclusiveChoice = 4,
    SimpleMerge = 5,
    MultiChoice = 6,
    Discriminator = 9, // Pattern 9: First-wins (race condition)
    ArbitraryCycles = 10,
    ImplicitTermination = 11, // Pattern 11: Workflow completion
    DeferredChoice = 16,
    Timeout = 20,      // Production: Timeout pattern
    Cancellation = 21, // Production: Cancellation pattern
}

pub type BranchFn = unsafe extern "C" fn(*mut PatternContext) -> bool;
pub type ConditionFn = unsafe extern "C" fn(*const PatternContext) -> bool;

// ============================================================================
// FFI Function Declarations
// ============================================================================

extern "C" {
    // Pattern 1: Sequence
    pub fn knhk_pattern_sequence(
        ctx: *mut PatternContext,
        branches: *const BranchFn,
        num_branches: c_uint,
    ) -> PatternResult;

    // Pattern 2: Parallel Split
    pub fn knhk_pattern_parallel_split(
        ctx: *mut PatternContext,
        branches: *const BranchFn,
        num_branches: c_uint,
    ) -> PatternResult;

    pub fn knhk_pattern_parallel_split_simd(
        ctx: *mut PatternContext,
        branches: *const BranchFn,
        num_branches: c_uint,
    ) -> PatternResult;

    // Pattern 3: Synchronization
    pub fn knhk_pattern_synchronization(
        ctx: *mut PatternContext,
        branch_results: *const u64,
        num_branches: c_uint,
    ) -> PatternResult;

    pub fn knhk_pattern_synchronization_simd(
        ctx: *mut PatternContext,
        branch_results: *const u64,
        num_branches: c_uint,
    ) -> PatternResult;

    // Pattern 4: Exclusive Choice
    pub fn knhk_pattern_exclusive_choice(
        ctx: *mut PatternContext,
        conditions: *const ConditionFn,
        branches: *const BranchFn,
        num_branches: c_uint,
    ) -> PatternResult;

    // Pattern 5: Simple Merge
    pub fn knhk_pattern_simple_merge(ctx: *mut PatternContext, branch_result: u64)
        -> PatternResult;

    // Pattern 6: Multi-Choice
    pub fn knhk_pattern_multi_choice(
        ctx: *mut PatternContext,
        conditions: *const ConditionFn,
        branches: *const BranchFn,
        num_branches: c_uint,
    ) -> PatternResult;

    pub fn knhk_pattern_multi_choice_simd(
        ctx: *mut PatternContext,
        conditions: *const ConditionFn,
        branches: *const BranchFn,
        num_branches: c_uint,
    ) -> PatternResult;

    // Pattern 10: Arbitrary Cycles
    pub fn knhk_pattern_arbitrary_cycles(
        ctx: *mut PatternContext,
        branch: BranchFn,
        should_continue: ConditionFn,
        max_iterations: c_uint,
    ) -> PatternResult;

    // Pattern 16: Deferred Choice
    pub fn knhk_pattern_deferred_choice(
        ctx: *mut PatternContext,
        conditions: *const ConditionFn,
        branches: *const BranchFn,
        num_branches: c_uint,
        timeout_ticks: u64,
    ) -> PatternResult;

    // Branchless dispatch
    pub fn knhk_dispatch_pattern(
        pattern_type: PatternType,
        ctx: *mut PatternContext,
        pattern_data: *mut c_void,
        data_size: c_uint,
    ) -> PatternResult;

    // Helper functions
    pub fn knhk_pattern_context_create(capacity: c_uint) -> *mut PatternContext;
    pub fn knhk_pattern_context_destroy(ctx: *mut PatternContext);
    pub fn knhk_pattern_context_add(ctx: *mut PatternContext, data: u64) -> bool;
    pub fn knhk_pattern_name(pattern_type: PatternType) -> *const c_char;
    pub fn knhk_pattern_tick_budget(pattern_type: PatternType) -> c_uint;
    pub fn knhk_pattern_validate_ingress(
        pattern_type: PatternType,
        num_branches: c_uint,
        error_msg: *mut *const c_char,
    ) -> bool;
}

// ============================================================================
// Safe Rust Wrappers
// ============================================================================

impl PatternResult {
    /// Convert C pattern result to Rust Result
    pub fn into_result(self) -> Result<u64, String> {
        if self.success {
            Ok(self.result)
        } else {
            let error_msg = if self.error.is_null() {
                "Unknown error".to_string()
            } else {
                unsafe { CStr::from_ptr(self.error).to_string_lossy().into_owned() }
            };
            Err(error_msg)
        }
    }
}

impl PatternType {
    /// Get pattern name for telemetry
    pub fn name(&self) -> &'static str {
        unsafe {
            let c_str = knhk_pattern_name(*self);
            CStr::from_ptr(c_str).to_str().unwrap_or("Unknown")
        }
    }

    /// Get tick budget for ingress validation
    pub fn tick_budget(&self) -> u32 {
        // Hardcoded budgets for new patterns not in C yet
        match self {
            PatternType::Sequence => 1,
            PatternType::ParallelSplit => 2,
            PatternType::Synchronization => 3,
            PatternType::ExclusiveChoice => 2,
            PatternType::SimpleMerge => 1,
            PatternType::MultiChoice => 3,
            PatternType::Discriminator => 3, // First-wins with atomic coordination
            PatternType::ArbitraryCycles => 2,
            PatternType::ImplicitTermination => 2, // Track active branches
            PatternType::DeferredChoice => 3,
            PatternType::Timeout => 2,      // Check timeout + execute
            PatternType::Cancellation => 1, // Atomic cancel check
        }
    }

    /// Validate pattern at ingress (guards enforce constraints ONCE)
    pub fn validate_ingress(&self, num_branches: u32) -> Result<(), String> {
        // For new patterns not yet in C, validate in Rust
        match self {
            PatternType::Discriminator
            | PatternType::ImplicitTermination
            | PatternType::Timeout
            | PatternType::Cancellation => {
                // Rust-side validation for new patterns
                const MAX_BRANCHES: u32 = 1024;

                match self {
                    PatternType::Timeout | PatternType::Cancellation => {
                        if num_branches != 1 {
                            return Err(format!(
                                "{} requires exactly 1 branch",
                                match self {
                                    PatternType::Timeout => "Timeout",
                                    PatternType::Cancellation => "Cancellation",
                                    _ => unreachable!(),
                                }
                            ));
                        }
                    }
                    PatternType::Discriminator | PatternType::ImplicitTermination => {
                        if num_branches == 0 {
                            return Err(format!(
                                "{} requires at least 1 branch",
                                match self {
                                    PatternType::Discriminator => "Discriminator",
                                    PatternType::ImplicitTermination => "ImplicitTermination",
                                    _ => unreachable!(),
                                }
                            ));
                        }
                        if num_branches > MAX_BRANCHES {
                            return Err(format!(
                                "Too many branches: {} (max {})",
                                num_branches, MAX_BRANCHES
                            ));
                        }
                    }
                    _ => {}
                }
                Ok(())
            }
            // For existing patterns, use C validation
            _ => unsafe {
                let mut error_ptr: *const c_char = std::ptr::null();
                let valid = knhk_pattern_validate_ingress(*self, num_branches, &mut error_ptr);

                if valid {
                    Ok(())
                } else {
                    let error_msg = if error_ptr.is_null() {
                        "Validation failed".to_string()
                    } else {
                        CStr::from_ptr(error_ptr).to_string_lossy().into_owned()
                    };
                    Err(error_msg)
                }
            },
        }
    }

    /// Check if pattern supports SIMD optimization
    pub fn is_simd_capable(&self) -> bool {
        matches!(
            self,
            PatternType::ParallelSplit
                | PatternType::Synchronization
                | PatternType::MultiChoice
                | PatternType::Discriminator // SIMD for parallel race
        )
    }
}
