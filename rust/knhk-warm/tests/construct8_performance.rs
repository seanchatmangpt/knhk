//! CONSTRUCT8 Performance Tests
//!
//! Tests that CONSTRUCT8 completes within 8 tick budget (Chatman Constant).
//! This addresses the critical performance gap where CONSTRUCT8 was 41-83 ticks.

use chicago_tdd_tools::{assert_within_tick_budget, measure_ticks};
use knhk_hot::{Ctx, Ir, Op};
use knhk_warm::construct8::WarmPathConstruct8;

#[test]
fn test_construct8_performance_within_budget() {
    // Arrange: Create test context (up to MAX_RUN_LEN=8)
    // CONSTRUCT8 should handle up to 8 triples within tick budget

    // Test with various input sizes
    for len in 1..=8 {
        // Create minimal context for testing
        let mut ctx = Ctx::default();
        ctx.run.len = len;
        ctx.run.off = 0;

        let mut ir = Ir::default();
        ir.op = Op::Construct8;

        // Act: Measure CONSTRUCT8 performance
        let (_result, ticks) = measure_ticks(|| WarmPathConstruct8::execute(&ctx, &mut ir));

        // Assert: CONSTRUCT8 completes within 8 tick budget
        assert_within_tick_budget!(
            ticks,
            format!(
                "CONSTRUCT8 with {} triples should complete within 8 ticks",
                len
            )
        );
    }
}

#[test]
fn test_construct8_performance_optimization() {
    // Arrange: Test that CONSTRUCT8 uses optimized paths
    // This verifies SIMD optimizations and branchless paths

    let mut ctx = Ctx::default();
    ctx.run.len = 8; // MAX_RUN_LEN input
    ctx.run.off = 0;

    let mut ir = Ir::default();
    ir.op = Op::Construct8;

    // Act: Measure performance multiple times to verify consistency
    let mut all_ticks = Vec::new();
    for _ in 0..10 {
        let (_result, ticks) = measure_ticks(|| WarmPathConstruct8::execute(&ctx, &mut ir));
        all_ticks.push(ticks);
    }

    // Assert: All measurements are within budget
    for ticks in &all_ticks {
        assert_within_tick_budget!(
            *ticks,
            "CONSTRUCT8 should consistently complete within 8 ticks"
        );
    }

    // Assert: Performance is consistent (low variance indicates optimization)
    let max_ticks = *all_ticks.iter().max().unwrap();
    let min_ticks = *all_ticks.iter().min().unwrap();
    let variance = max_ticks - min_ticks;

    // Variance should be low for optimized code
    assert!(
        variance <= 2,
        "CONSTRUCT8 performance should be consistent (low variance)"
    );
}
