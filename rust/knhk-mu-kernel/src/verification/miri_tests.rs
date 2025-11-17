//! MIRI Tests for Memory Safety and Undefined Behavior Detection
//!
//! These tests run under Miri to detect:
//! - Undefined behavior
//! - Invalid memory accesses
//! - Aliasing violations
//! - Stacked borrows violations
//! - Use-after-free
//! - Double-free
//! - Data races
//!
//! # Running MIRI
//!
//! ```bash
//! cargo +nightly miri test --package knhk-mu-kernel --test miri_tests
//! cargo +nightly miri test --features verification
//! ```

#![cfg(test)]

use crate::{
    guards::GuardContext,
    memory,
    patterns::PatternId,
    receipts::Receipt,
    sigma::{SigmaCompiled, SigmaHash},
    timing::{BudgetStatus, TickBudget, TickCounter},
    CHATMAN_CONSTANT,
};

/// Test: TickBudget has no UB in construction
#[test]
fn miri_tick_budget_construction() {
    let budget = TickBudget::chatman();
    assert_eq!(budget.limit, 8);
    assert_eq!(budget.used, 0);

    let custom = TickBudget::new(100);
    assert_eq!(custom.limit, 100);
    assert_eq!(custom.used, 0);
}

/// Test: TickBudget consume has no UB
#[test]
fn miri_tick_budget_consume() {
    let mut budget = TickBudget::chatman();

    for i in 1..=10 {
        let status = budget.consume(1);

        if i <= 8 {
            assert_eq!(status, BudgetStatus::Ok);
        } else {
            assert_eq!(status, BudgetStatus::Exhausted);
        }
    }

    assert_eq!(budget.used, 10);
}

/// Test: TickBudget remaining calculation has no UB
#[test]
fn miri_tick_budget_remaining() {
    let mut budget = TickBudget::new(10);

    assert_eq!(budget.remaining(), 10);

    budget.consume(3);
    assert_eq!(budget.remaining(), 7);

    budget.consume(20); // Over budget
    assert_eq!(budget.remaining(), 0); // Saturating
}

/// Test: TickBudget reset has no UB
#[test]
fn miri_tick_budget_reset() {
    let mut budget = TickBudget::chatman();

    budget.consume(5);
    assert_eq!(budget.used, 5);

    budget.reset();
    assert_eq!(budget.used, 0);
    assert_eq!(budget.remaining(), 8);
}

/// Test: TickCounter has no UB in construction
#[test]
fn miri_tick_counter_construction() {
    let counter = TickCounter::new();
    // Just verify no UB during construction
    let _ = counter.ticks();
}

/// Test: TickCounter start/ticks has no UB
#[test]
fn miri_tick_counter_measurement() {
    let mut counter = TickCounter::new();
    counter.start();

    // Do some work
    let mut sum = 0u64;
    for i in 0..100 {
        sum = sum.wrapping_add(i);
    }

    let ticks = counter.ticks();
    assert!(ticks < u64::MAX); // Sanity check
    assert!(sum > 0); // Prevent optimization
}

/// Test: GuardContext has no UB in construction
#[test]
fn miri_guard_context() {
    let ctx = GuardContext {
        task_id: 42,
        obs_data: 0xDEADBEEF,
        params: [1, 2, 3, 4],
    };

    assert_eq!(ctx.task_id, 42);
    assert_eq!(ctx.obs_data, 0xDEADBEEF);
    assert_eq!(ctx.params[0], 1);
    assert_eq!(ctx.params[3], 4);
}

/// Test: PatternId transmute is safe for valid values
#[test]
fn miri_pattern_id_transmute() {
    // Test all valid pattern IDs
    for i in 0..43u8 {
        let pattern: PatternId = unsafe { core::mem::transmute(i) };
        let cost = pattern.tick_cost();

        // Verify tick cost is reasonable
        assert!(cost > 0);
        assert!(cost <= CHATMAN_CONSTANT as u8);
    }
}

/// Test: Memory layout constants are valid
#[test]
fn miri_memory_layout() {
    // Just verify the constants are accessible
    assert!(memory::SIGMA_BASE < memory::SIGMA_BASE + memory::SIGMA_SIZE);
    assert!(memory::PATTERN_BASE < memory::PATTERN_BASE + memory::PATTERN_SIZE);
    assert!(memory::GUARD_BASE < memory::GUARD_BASE + memory::GUARD_SIZE);
    assert!(memory::OBS_BASE < memory::OBS_BASE + memory::OBS_SIZE);
    assert!(memory::RECEIPT_BASE < memory::RECEIPT_BASE + memory::RECEIPT_SIZE);
}

/// Test: Memory regions don't overlap
#[test]
fn miri_memory_non_overlapping() {
    use memory::*;

    assert!(SIGMA_BASE + SIGMA_SIZE <= PATTERN_BASE);
    assert!(PATTERN_BASE + PATTERN_SIZE <= GUARD_BASE);
    assert!(GUARD_BASE + GUARD_SIZE <= OBS_BASE);
    assert!(OBS_BASE + OBS_SIZE <= RECEIPT_BASE);
    assert!(RECEIPT_BASE + RECEIPT_SIZE <= WARM_BASE);
}

/// Test: Saturating arithmetic has no UB
#[test]
fn miri_saturating_arithmetic() {
    let max = u64::MAX;

    // Saturating add
    let result = max.saturating_add(1);
    assert_eq!(result, max);

    let result = max.saturating_add(max);
    assert_eq!(result, max);

    // Saturating sub
    let result = 0u64.saturating_sub(1);
    assert_eq!(result, 0);

    let result = 5u64.saturating_sub(10);
    assert_eq!(result, 0);
}

/// Test: BudgetStatus enum has valid discriminants
#[test]
fn miri_budget_status_discriminants() {
    let ok = BudgetStatus::Ok;
    let exhausted = BudgetStatus::Exhausted;

    // Verify discriminants are as expected
    assert_eq!(ok as u8, 0);
    assert_eq!(exhausted as u8, 1);

    // Verify equality works
    assert_eq!(ok, BudgetStatus::Ok);
    assert_ne!(ok, exhausted);
}

/// Test: Array indexing is bounds-checked
#[test]
fn miri_array_indexing() {
    let ctx = GuardContext {
        task_id: 1,
        obs_data: 2,
        params: [10, 20, 30, 40],
    };

    // Valid indices
    assert_eq!(ctx.params[0], 10);
    assert_eq!(ctx.params[1], 20);
    assert_eq!(ctx.params[2], 30);
    assert_eq!(ctx.params[3], 40);

    // Iterator access (always safe)
    let sum: u64 = ctx.params.iter().sum();
    assert_eq!(sum, 100);
}

/// Test: Wrapping arithmetic has no UB
#[test]
fn miri_wrapping_arithmetic() {
    let max = u64::MAX;

    // Wrapping add
    let result = max.wrapping_add(1);
    assert_eq!(result, 0);

    let result = max.wrapping_add(2);
    assert_eq!(result, 1);

    // Wrapping sub
    let result = 0u64.wrapping_sub(1);
    assert_eq!(result, max);
}

/// Test: Clone implementation has no UB
#[test]
fn miri_clone_implementation() {
    let budget1 = TickBudget::chatman();
    let budget2 = budget1.clone();

    assert_eq!(budget1.limit, budget2.limit);
    assert_eq!(budget1.used, budget2.used);
}

/// Test: Default implementation has no UB
#[test]
fn miri_default_implementation() {
    let counter = TickCounter::default();
    let _ = counter.ticks();
}

/// Test: Multiple references to TickBudget (aliasing test)
#[test]
fn miri_aliasing_immutable_refs() {
    let budget = TickBudget::chatman();

    let ref1 = &budget;
    let ref2 = &budget;
    let ref3 = &budget;

    assert_eq!(ref1.limit, ref2.limit);
    assert_eq!(ref2.limit, ref3.limit);
    assert_eq!(ref1.used, ref2.used);
}

/// Test: Mutable reference exclusivity
#[test]
fn miri_aliasing_mutable_ref() {
    let mut budget = TickBudget::chatman();

    {
        let ref_mut = &mut budget;
        ref_mut.consume(3);
        assert_eq!(ref_mut.used, 3);
    }

    // Mutable reference dropped, can access again
    assert_eq!(budget.used, 3);
}

/// Test: Stacked borrows - read after write
#[test]
fn miri_stacked_borrows_read_after_write() {
    let mut budget = TickBudget::chatman();

    // Write
    budget.consume(5);

    // Read
    let used = budget.used;
    assert_eq!(used, 5);

    // Write again
    budget.consume(2);

    // Read again
    assert_eq!(budget.used, 7);
}

/// Test: Interior mutability (AtomicU64 in TickCounter)
#[test]
fn miri_interior_mutability() {
    let counter = TickCounter::new();

    // Even though counter is not mut, we can call ticks()
    // because it uses AtomicU64 internally
    let t1 = counter.ticks();
    let t2 = counter.ticks();

    // Both should succeed without UB
    assert!(t1 <= t2 || t1 >= t2); // Some ordering exists
}

/// Test: Const fn construction
#[test]
fn miri_const_fn_construction() {
    const BUDGET: TickBudget = TickBudget::chatman();
    const COUNTER: TickCounter = TickCounter::new();

    assert_eq!(BUDGET.limit, 8);
    assert_eq!(BUDGET.used, 0);
}

/// Test: Repr(C) layout is stable
#[test]
fn miri_repr_c_layout() {
    use core::mem::{align_of, size_of};

    // TickBudget is repr(C, align(8))
    assert_eq!(size_of::<TickBudget>(), 16); // 2 × u64
    assert_eq!(align_of::<TickBudget>(), 8);

    // GuardContext is repr(C, align(64))
    assert!(align_of::<GuardContext>() >= 64);
}

/// Test: Zero-sized types
#[test]
fn miri_zero_sized_types() {
    use core::marker::PhantomData;
    use core::mem::size_of;

    // PhantomData is zero-sized
    assert_eq!(size_of::<PhantomData<u64>>(), 0);

    // Multiple PhantomData still zero-sized
    assert_eq!(size_of::<(PhantomData<u64>, PhantomData<u8>)>(), 0);
}

/// Test: Uninitialized memory handling
#[test]
fn miri_uninitialized_memory() {
    use core::mem::MaybeUninit;

    let mut uninit: MaybeUninit<TickBudget> = MaybeUninit::uninit();

    // Initialize
    uninit.write(TickBudget::chatman());

    // Now safe to assume init
    let budget = unsafe { uninit.assume_init() };
    assert_eq!(budget.limit, 8);
}

/// Test: Slice iteration is safe
#[test]
fn miri_slice_iteration() {
    let params = [1u64, 2, 3, 4, 5];

    let mut sum = 0u64;
    for &param in params.iter() {
        sum = sum.saturating_add(param);
    }

    assert_eq!(sum, 15);
}

/// Test: Range iteration is safe
#[test]
fn miri_range_iteration() {
    let mut count = 0;

    for i in 0..CHATMAN_CONSTANT {
        count += 1;
        assert!(i < CHATMAN_CONSTANT);
    }

    assert_eq!(count, 8);
}

/// Test: Pattern matching on enums
#[test]
fn miri_pattern_matching() {
    let status_ok = BudgetStatus::Ok;
    let status_exhausted = BudgetStatus::Exhausted;

    match status_ok {
        BudgetStatus::Ok => assert!(true),
        BudgetStatus::Exhausted => panic!("Should be Ok"),
    }

    match status_exhausted {
        BudgetStatus::Ok => panic!("Should be Exhausted"),
        BudgetStatus::Exhausted => assert!(true),
    }
}

/// Test: Transmute between repr(u8) enum and u8
#[test]
fn miri_enum_transmute() {
    let ok = BudgetStatus::Ok;
    let ok_u8: u8 = unsafe { core::mem::transmute(ok) };
    assert_eq!(ok_u8, 0);

    let exhausted = BudgetStatus::Exhausted;
    let exhausted_u8: u8 = unsafe { core::mem::transmute(exhausted) };
    assert_eq!(exhausted_u8, 1);

    // Reverse transmute
    let back_to_ok: BudgetStatus = unsafe { core::mem::transmute(0u8) };
    assert_eq!(back_to_ok, BudgetStatus::Ok);
}

/// Test: Inline assembly is properly isolated (arch-specific)
#[test]
#[cfg(target_arch = "x86_64")]
fn miri_inline_asm_isolation() {
    // Miri can't execute inline assembly, so this test would be skipped under Miri
    // But we verify the function compiles and doesn't cause UB in non-asm code

    let mut counter = TickCounter::new();
    counter.start();

    // The inline asm in read_cycle_counter() won't execute under Miri,
    // but the fallback path should work
    let _ = counter.ticks();
}

/// Test: Const evaluation doesn't cause UB
#[test]
fn miri_const_evaluation() {
    const LIMIT: u64 = CHATMAN_CONSTANT;
    const DOUBLE: u64 = LIMIT * 2;
    const HALF: u64 = LIMIT / 2;

    assert_eq!(LIMIT, 8);
    assert_eq!(DOUBLE, 16);
    assert_eq!(HALF, 4);
}

#[cfg(test)]
mod miri_stress_tests {
    use super::*;

    /// Stress test: Many sequential budget operations
    #[test]
    fn miri_stress_sequential_operations() {
        let mut budget = TickBudget::chatman();

        for _ in 0..1000 {
            budget.reset();

            for tick in 1..=8 {
                let status = budget.consume(1);
                if tick < 8 {
                    assert_eq!(status, BudgetStatus::Ok);
                } else {
                    assert_eq!(status, BudgetStatus::Exhausted);
                }
            }
        }
    }

    /// Stress test: Many counter measurements
    #[test]
    fn miri_stress_counter_measurements() {
        let mut counter = TickCounter::new();
        counter.start();

        for _ in 0..1000 {
            let _ = counter.ticks();
        }
    }

    /// Stress test: Array access patterns
    #[test]
    fn miri_stress_array_access() {
        let ctx = GuardContext {
            task_id: 0,
            obs_data: 0,
            params: [1, 2, 3, 4],
        };

        for _ in 0..1000 {
            for i in 0..4 {
                let _ = ctx.params[i];
            }
        }
    }
}
