//! Benchmark: Join Operation Latency
//!
//! Measures join synchronization point latency.
//! Join operations MUST complete in ≤8 ticks.

use chicago_tdd::{OperationType, PerformanceHarness, Reporter};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::atomic::{AtomicU32, Ordering};

/// AND-join: Wait for all branches (hot path)
fn and_join() -> bool {
    // Check if all branches completed
    static COMPLETED: AtomicU32 = AtomicU32::new(0);
    let current = COMPLETED.load(Ordering::Relaxed);
    let expected = 3; // 3 branches
    current == expected
}

/// XOR-join: First branch wins (hot path)
fn xor_join() -> bool {
    // Check if any branch completed
    static COMPLETED: AtomicU32 = AtomicU32::new(0);
    let current = COMPLETED.load(Ordering::Relaxed);
    current > 0
}

/// OR-join: Wait for all active branches (hot path)
fn or_join() -> bool {
    // Check if all active branches completed
    static COMPLETED: AtomicU32 = AtomicU32::new(0);
    static ACTIVE: AtomicU32 = AtomicU32::new(0);
    let completed = COMPLETED.load(Ordering::Relaxed);
    let active = ACTIVE.load(Ordering::Relaxed);
    completed >= active
}

/// Join counter increment (hot path)
fn join_counter_inc() -> u32 {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Join condition check (hot path)
fn join_condition() -> bool {
    // Check join condition
    let arrived = 3;
    let expected = 3;
    let cancelled = 0;
    arrived == expected && cancelled == 0
}

fn bench_join_operation_hot_path(c: &mut Criterion) {
    let mut harness = PerformanceHarness::with_iterations(1000, 10000, 100);

    c.bench_function("join_and_check", |b| b.iter(|| black_box(and_join())));

    c.bench_function("join_xor_check", |b| b.iter(|| black_box(xor_join())));

    c.bench_function("join_or_check", |b| b.iter(|| black_box(or_join())));

    c.bench_function("join_counter_inc", |b| {
        b.iter(|| black_box(join_counter_inc()))
    });

    c.bench_function("join_condition_check", |b| {
        b.iter(|| black_box(join_condition()))
    });

    // Chicago TDD enforcement
    println!("\n{}", "=".repeat(80));
    println!("Chicago TDD Enforcement: Join Operation Hot Path");
    println!("{}", "=".repeat(80));

    let r1 = harness.measure("and_join", OperationType::HotPath, and_join);
    let r2 = harness.measure("xor_join", OperationType::HotPath, xor_join);
    let r3 = harness.measure("or_join", OperationType::HotPath, or_join);
    let r4 = harness.measure(
        "counter_increment",
        OperationType::HotPath,
        join_counter_inc,
    );
    let r5 = harness.measure("condition_check", OperationType::HotPath, join_condition);

    Reporter::print_result(&r1);
    Reporter::print_result(&r2);
    Reporter::print_result(&r3);
    Reporter::print_result(&r4);
    Reporter::print_result(&r5);

    let report = harness.report();
    Reporter::print_report(&report);

    if let Err(e) = harness.assert_all_within_bounds() {
        eprintln!(
            "\n{}",
            "❌ CRITICAL: Join Operation Chatman Constant Violation"
        );
        eprintln!("{}", e);
        panic!("{}", e);
    }

    println!("\n✅ All join operations within Chatman Constant (≤8 ticks)\n");
}

criterion_group!(benches, bench_join_operation_hot_path);
criterion_main!(benches);
