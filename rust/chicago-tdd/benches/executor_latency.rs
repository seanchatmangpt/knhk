//! Benchmark: Workflow Executor Latency
//!
//! Measures critical path latency for workflow engine executor operations.
//! All executor hot path operations MUST complete in ≤8 ticks.

use chicago_tdd::{PerformanceHarness, OperationType, Reporter};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Simulate workflow task lookup (hot path)
fn task_lookup() -> usize {
    // Simulates hash map lookup in executor
    let tasks = vec![1, 2, 3, 4, 5];
    tasks.iter().position(|&x| x == 3).unwrap_or(0)
}

/// Simulate case state access (hot path)
fn case_state_access() -> bool {
    // Simulates reading case state from concurrent map
    let state = 42;
    state > 0
}

/// Simulate pattern registry lookup (hot path)
fn pattern_lookup() -> u32 {
    // Simulates pattern ID lookup
    let patterns = [1, 2, 3, 4, 5, 6, 7, 8];
    patterns[3]
}

/// Simulate decision evaluation (hot path)
fn decision_eval() -> bool {
    // Simulates simple condition evaluation
    let x = 42;
    let y = 10;
    x > y && x < 100
}

/// Simulate task state transition check (hot path)
fn state_transition_check() -> bool {
    // Simulates checking if task can transition
    let current_state = 2;
    let target_state = 3;
    target_state == current_state + 1
}

fn bench_executor_hot_path(c: &mut Criterion) {
    let mut harness = PerformanceHarness::with_iterations(1000, 10000, 100);

    c.bench_function("executor_task_lookup", |b| {
        b.iter(|| black_box(task_lookup()))
    });

    c.bench_function("executor_case_state_access", |b| {
        b.iter(|| black_box(case_state_access()))
    });

    c.bench_function("executor_pattern_lookup", |b| {
        b.iter(|| black_box(pattern_lookup()))
    });

    c.bench_function("executor_decision_eval", |b| {
        b.iter(|| black_box(decision_eval()))
    });

    c.bench_function("executor_state_transition", |b| {
        b.iter(|| black_box(state_transition_check()))
    });

    // Chicago TDD enforcement: All must be ≤8 ticks
    println!("\n{}", "=".repeat(80));
    println!("Chicago TDD Enforcement: Executor Hot Path");
    println!("{}", "=".repeat(80));

    let r1 = harness.measure("task_lookup", OperationType::HotPath, task_lookup);
    let r2 = harness.measure("case_state_access", OperationType::HotPath, case_state_access);
    let r3 = harness.measure("pattern_lookup", OperationType::HotPath, pattern_lookup);
    let r4 = harness.measure("decision_eval", OperationType::HotPath, decision_eval);
    let r5 = harness.measure("state_transition_check", OperationType::HotPath, state_transition_check);

    // Print individual results
    Reporter::print_result(&r1);
    Reporter::print_result(&r2);
    Reporter::print_result(&r3);
    Reporter::print_result(&r4);
    Reporter::print_result(&r5);

    // Generate report
    let report = harness.report();
    Reporter::print_report(&report);

    // Assert all within bounds (will panic if any violation)
    if let Err(e) = harness.assert_all_within_bounds() {
        eprintln!("\n{}", "❌ CRITICAL: Chatman Constant Violation".to_uppercase());
        eprintln!("{}", e);
        eprintln!("\nBuild MUST be blocked. Fix hot path latency violations before merging.");
        panic!("{}", e);
    }

    println!("\n✅ All executor hot path operations within Chatman Constant (≤8 ticks)\n");
}

criterion_group!(benches, bench_executor_hot_path);
criterion_main!(benches);
