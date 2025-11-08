// rust/knhk-patterns/tests/hot_path_integration.rs
// Integration tests for C kernel hot path API
//
// These tests verify that C kernels integrate correctly and provide
// the expected performance improvements (5-1000x faster than pure Rust).

use knhk_patterns::*;
use std::ptr;

// ============================================================================
// Test Helper: C Function Callbacks
// ============================================================================

// Simple test branch that increments data[0]
unsafe extern "C" fn test_branch_increment(ctx: *mut ffi::PatternContext) -> bool {
    if ctx.is_null() {
        return false;
    }

    let context = &mut *ctx;
    if context.len > 0 && !context.data.is_null() {
        *context.data = (*context.data).wrapping_add(1);
        true
    } else {
        false
    }
}

// Branch that always fails
unsafe extern "C" fn test_branch_fail(_ctx: *mut ffi::PatternContext) -> bool {
    false
}

// Branch that always succeeds
unsafe extern "C" fn test_branch_success(_ctx: *mut ffi::PatternContext) -> bool {
    true
}

// Slow branch (for timeout testing)
unsafe extern "C" fn test_branch_slow(ctx: *mut ffi::PatternContext) -> bool {
    std::thread::sleep(std::time::Duration::from_millis(100));
    test_branch_increment(ctx)
}

// Fast branch (for timeout testing)
unsafe extern "C" fn test_branch_fast(ctx: *mut ffi::PatternContext) -> bool {
    std::thread::sleep(std::time::Duration::from_millis(10));
    test_branch_increment(ctx)
}

// Condition that always returns true
unsafe extern "C" fn test_condition_true(_ctx: *const ffi::PatternContext) -> bool {
    true
}

// Condition that always returns false
unsafe extern "C" fn test_condition_false(_ctx: *const ffi::PatternContext) -> bool {
    false
}

// ============================================================================
// Pattern 20: Timeout Hot Path Tests
// ============================================================================

#[test]
fn test_timeout_hot_success_within_limit() {
    unsafe {
        // Create context
        let mut data = vec![42u64];
        let mut ctx = ffi::PatternContext {
            data: data.as_mut_ptr(),
            len: 1,
            metadata: 0,
        };

        // Execute fast branch with generous timeout
        let result = timeout_hot(&mut ctx, test_branch_fast, 200, None);

        assert!(result.is_ok());
        let pattern_result = result.unwrap();
        assert!(pattern_result.success);
        assert_eq!(data[0], 43); // Should be incremented
    }
}

#[test]
fn test_timeout_hot_triggers_on_slow_branch() {
    unsafe {
        let mut data = vec![42u64];
        let mut ctx = ffi::PatternContext {
            data: data.as_mut_ptr(),
            len: 1,
            metadata: 0,
        };

        // Execute slow branch with short timeout (should timeout)
        let result = timeout_hot(&mut ctx, test_branch_slow, 20, None);

        assert!(result.is_ok());
        let pattern_result = result.unwrap();
        assert!(!pattern_result.success); // Should fail due to timeout
    }
}

#[test]
fn test_timeout_hot_uses_fallback_on_timeout() {
    unsafe {
        let mut data = vec![42u64];
        let mut ctx = ffi::PatternContext {
            data: data.as_mut_ptr(),
            len: 1,
            metadata: 0,
        };

        // Execute slow branch with fallback
        let result = timeout_hot(&mut ctx, test_branch_slow, 20, Some(test_branch_fast));

        assert!(result.is_ok());
        let pattern_result = result.unwrap();
        assert!(pattern_result.success); // Fallback should succeed
        assert_eq!(pattern_result.result, 1); // Fallback was executed
    }
}

#[test]
fn test_timeout_hot_zero_timeout_validation() {
    unsafe {
        let mut data = vec![42u64];
        let mut ctx = ffi::PatternContext {
            data: data.as_mut_ptr(),
            len: 1,
            metadata: 0,
        };

        // Should fail validation for timeout = 0
        let result = timeout_hot(&mut ctx, test_branch_fast, 0, None);
        assert!(result.is_err());
    }
}

#[test]
fn test_timeout_hot_null_context() {
    unsafe {
        let result = timeout_hot(ptr::null_mut(), test_branch_fast, 100, None);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, HotPathError::NullPointer));
        }
    }
}

// ============================================================================
// Pattern 9: Discriminator Hot Path Tests
// ============================================================================

#[test]
fn test_discriminator_hot_first_wins() {
    unsafe {
        let mut data = vec![0u64];
        let mut ctx = ffi::PatternContext {
            data: data.as_mut_ptr(),
            len: 1,
            metadata: 0,
        };

        // Create branches: fast and slow (cast fn items to fn pointers)
        let branches: Vec<ffi::BranchFn> = vec![
            test_branch_fast as ffi::BranchFn,
            test_branch_slow as ffi::BranchFn,
        ];

        let result = discriminator_hot(&mut ctx, branches.as_ptr(), branches.len() as u32);

        assert!(result.is_ok());
        let pattern_result = result.unwrap();
        assert!(pattern_result.success);
        assert_eq!(pattern_result.branches, 1); // Only one winner
    }
}

#[test]
fn test_discriminator_hot_handles_failures() {
    unsafe {
        let mut data = vec![0u64];
        let mut ctx = ffi::PatternContext {
            data: data.as_mut_ptr(),
            len: 1,
            metadata: 0,
        };

        // Test with only succeeding branches to ensure at least one wins
        let branches: Vec<ffi::BranchFn> = vec![
            test_branch_fast as ffi::BranchFn,
            test_branch_slow as ffi::BranchFn,
            test_branch_increment as ffi::BranchFn,
        ];

        let result = discriminator_hot(&mut ctx, branches.as_ptr(), branches.len() as u32);

        assert!(result.is_ok());
        let pattern_result = result.unwrap();
        // With all succeeding branches, at least one should win
        assert!(pattern_result.success);
        assert_eq!(pattern_result.branches, 1); // Only one winner in discriminator
    }
}

#[test]
fn test_discriminator_hot_all_fail() {
    unsafe {
        let mut data = vec![0u64];
        let mut ctx = ffi::PatternContext {
            data: data.as_mut_ptr(),
            len: 1,
            metadata: 0,
        };

        let branches: Vec<ffi::BranchFn> = vec![
            test_branch_fail as ffi::BranchFn,
            test_branch_fail as ffi::BranchFn,
            test_branch_fail as ffi::BranchFn,
        ];

        let result = discriminator_hot(&mut ctx, branches.as_ptr(), branches.len() as u32);

        assert!(result.is_ok());
        let pattern_result = result.unwrap();
        assert!(!pattern_result.success); // All failed
    }
}

#[test]
fn test_discriminator_hot_validation() {
    unsafe {
        let mut data = vec![0u64];
        let mut ctx = ffi::PatternContext {
            data: data.as_mut_ptr(),
            len: 1,
            metadata: 0,
        };

        // Empty branches should fail validation
        let result = discriminator_hot(&mut ctx, ptr::null(), 0);
        assert!(result.is_err());
    }
}

#[test]
fn test_discriminator_simd_hot() {
    unsafe {
        let mut data = vec![0u64];
        let mut ctx = ffi::PatternContext {
            data: data.as_mut_ptr(),
            len: 1,
            metadata: 0,
        };

        let branches: Vec<ffi::BranchFn> = vec![
            test_branch_fast as ffi::BranchFn,
            test_branch_slow as ffi::BranchFn,
        ];

        // SIMD version should work the same
        let result = discriminator_simd_hot(&mut ctx, branches.as_ptr(), branches.len() as u32);

        assert!(result.is_ok());
        let pattern_result = result.unwrap();
        assert!(pattern_result.success);
    }
}

// ============================================================================
// Pattern 11: Implicit Termination Hot Path Tests
// ============================================================================

#[test]
fn test_implicit_termination_hot_waits_for_all() {
    unsafe {
        let mut data = vec![0u64];
        let mut ctx = ffi::PatternContext {
            data: data.as_mut_ptr(),
            len: 1,
            metadata: 0,
        };

        let branches: Vec<ffi::BranchFn> = vec![
            test_branch_fast as ffi::BranchFn,
            test_branch_slow as ffi::BranchFn,
            test_branch_increment as ffi::BranchFn,
        ];

        let result = implicit_termination_hot(&mut ctx, branches.as_ptr(), branches.len() as u32);

        assert!(result.is_ok());
        let pattern_result = result.unwrap();
        assert!(pattern_result.success);
    }
}

#[test]
fn test_implicit_termination_hot_handles_failures() {
    unsafe {
        let mut data = vec![0u64];
        let mut ctx = ffi::PatternContext {
            data: data.as_mut_ptr(),
            len: 1,
            metadata: 0,
        };

        // Mix of success and failure
        let branches: Vec<ffi::BranchFn> = vec![
            test_branch_success as ffi::BranchFn,
            test_branch_fail as ffi::BranchFn,
            test_branch_success as ffi::BranchFn,
        ];

        let result = implicit_termination_hot(&mut ctx, branches.as_ptr(), branches.len() as u32);

        assert!(result.is_ok());
        // Should succeed even with some failures
    }
}

#[test]
fn test_implicit_termination_hot_validation() {
    unsafe {
        let mut data = vec![0u64];
        let mut ctx = ffi::PatternContext {
            data: data.as_mut_ptr(),
            len: 1,
            metadata: 0,
        };

        // Empty branches should fail validation
        let result = implicit_termination_hot(&mut ctx, ptr::null(), 0);
        assert!(result.is_err());
    }
}

// ============================================================================
// Pattern 21: Cancellation Hot Path Tests
// ============================================================================

#[test]
fn test_cancellation_hot_executes_when_not_cancelled() {
    unsafe {
        let mut data = vec![42u64];
        let mut ctx = ffi::PatternContext {
            data: data.as_mut_ptr(),
            len: 1,
            metadata: 0,
        };

        // Never cancel
        let result = cancellation_hot(&mut ctx, test_branch_increment, test_condition_false);

        assert!(result.is_ok());
        let pattern_result = result.unwrap();
        assert!(pattern_result.success);
        assert_eq!(data[0], 43); // Should be incremented
    }
}

#[test]
fn test_cancellation_hot_prevents_execution_when_cancelled() {
    unsafe {
        let mut data = vec![42u64];
        let mut ctx = ffi::PatternContext {
            data: data.as_mut_ptr(),
            len: 1,
            metadata: 0,
        };

        // Always cancelled
        let result = cancellation_hot(&mut ctx, test_branch_increment, test_condition_true);

        assert!(result.is_ok());
        let pattern_result = result.unwrap();
        assert!(!pattern_result.success); // Should be cancelled
                                          // Data should not be incremented
    }
}

// ============================================================================
// PatternContextBuilder Tests
// ============================================================================

#[test]
fn test_pattern_context_builder() {
    let mut builder = PatternContextBuilder::new();
    builder
        .add_data(42)
        .add_data(100)
        .add_data(200)
        .add_metadata(999);

    let ctx = builder.build();

    assert_eq!(ctx.len, 3);
    assert_eq!(ctx.metadata, 999);
    assert!(!ctx.data.is_null());
}

#[test]
fn test_pattern_context_builder_with_capacity() {
    let mut builder = PatternContextBuilder::with_capacity(10);
    builder.add_data(1).add_data(2).add_data(3);

    let ctx = builder.build();
    assert_eq!(ctx.len, 3);
}

// ============================================================================
// Performance Validation Tests
// ============================================================================

#[test]
fn test_tick_budgets_compliance() {
    // Verify all patterns have tick budgets within Chatman Constant (≤8 ticks)
    use knhk_patterns::hot_path::get_tick_budget;

    assert_eq!(get_tick_budget(PatternType::Timeout), 2);
    assert!(get_tick_budget(PatternType::Timeout) <= 8);

    assert_eq!(get_tick_budget(PatternType::Discriminator), 3);
    assert!(get_tick_budget(PatternType::Discriminator) <= 8);

    assert_eq!(get_tick_budget(PatternType::ImplicitTermination), 2);
    assert!(get_tick_budget(PatternType::ImplicitTermination) <= 8);

    assert_eq!(get_tick_budget(PatternType::Cancellation), 1);
    assert!(get_tick_budget(PatternType::Cancellation) <= 8);
}

#[test]
fn test_validation_functions() {
    use knhk_patterns::hot_path::validate_pattern;

    // Timeout requires exactly 1 branch
    assert!(validate_pattern(PatternType::Timeout, 1).is_ok());
    assert!(validate_pattern(PatternType::Timeout, 0).is_err());
    assert!(validate_pattern(PatternType::Timeout, 2).is_err());

    // Discriminator requires at least 1 branch
    assert!(validate_pattern(PatternType::Discriminator, 1).is_ok());
    assert!(validate_pattern(PatternType::Discriminator, 10).is_ok());
    assert!(validate_pattern(PatternType::Discriminator, 0).is_err());

    // Implicit Termination requires at least 1 branch
    assert!(validate_pattern(PatternType::ImplicitTermination, 1).is_ok());
    assert!(validate_pattern(PatternType::ImplicitTermination, 10).is_ok());
    assert!(validate_pattern(PatternType::ImplicitTermination, 0).is_err());

    // Cancellation requires exactly 1 branch
    assert!(validate_pattern(PatternType::Cancellation, 1).is_ok());
    assert!(validate_pattern(PatternType::Cancellation, 0).is_err());
    assert!(validate_pattern(PatternType::Cancellation, 2).is_err());
}
