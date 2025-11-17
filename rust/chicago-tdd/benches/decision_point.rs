//! Benchmark: Decision Point Latency
//!
//! Measures split/join decision point evaluation latency.
//! All decision evaluations MUST complete in ≤8 ticks.

use chicago_tdd::{OperationType, PerformanceHarness, Reporter};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// AND-split decision (hot path)
fn and_split() -> Vec<usize> {
    // All branches execute
    vec![0, 1, 2]
}

/// XOR-split decision (hot path)
fn xor_split() -> usize {
    // Exactly one branch
    let condition = 42;
    if condition > 50 {
        0
    } else if condition > 25 {
        1
    } else {
        2
    }
}

/// OR-split decision (hot path)
fn or_split() -> Vec<usize> {
    // One or more branches
    let x = 42;
    let y = 30;
    let mut branches = Vec::with_capacity(3);
    if x > 40 {
        branches.push(0);
    }
    if y > 20 {
        branches.push(1);
    }
    if x + y > 70 {
        branches.push(2);
    }
    branches
}

/// Guard condition evaluation (hot path)
fn guard_eval() -> bool {
    // Evaluate guard condition
    let var1 = 100;
    let var2 = 50;
    var1 > var2 && var1 < 200
}

/// Branch selection (hot path)
fn branch_select() -> usize {
    // Select branch based on variable
    let status = "active";
    match status {
        "active" => 0,
        "pending" => 1,
        "completed" => 2,
        _ => 3,
    }
}

fn bench_decision_point_hot_path(c: &mut Criterion) {
    let mut harness = PerformanceHarness::with_iterations(1000, 10000, 100);

    c.bench_function("decision_and_split", |b| b.iter(|| black_box(and_split())));

    c.bench_function("decision_xor_split", |b| b.iter(|| black_box(xor_split())));

    c.bench_function("decision_or_split", |b| b.iter(|| black_box(or_split())));

    c.bench_function("decision_guard_eval", |b| {
        b.iter(|| black_box(guard_eval()))
    });

    c.bench_function("decision_branch_select", |b| {
        b.iter(|| black_box(branch_select()))
    });

    // Chicago TDD enforcement
    println!("\n{}", "=".repeat(80));
    println!("Chicago TDD Enforcement: Decision Point Hot Path");
    println!("{}", "=".repeat(80));

    let r1 = harness.measure("and_split", OperationType::HotPath, and_split);
    let r2 = harness.measure("xor_split", OperationType::HotPath, xor_split);
    let r3 = harness.measure("or_split", OperationType::HotPath, or_split);
    let r4 = harness.measure("guard_eval", OperationType::HotPath, guard_eval);
    let r5 = harness.measure("branch_select", OperationType::HotPath, branch_select);

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
            "❌ CRITICAL: Decision Point Chatman Constant Violation"
        );
        eprintln!("{}", e);
        panic!("{}", e);
    }

    println!("\n✅ All decision point operations within Chatman Constant (≤8 ticks)\n");
}

criterion_group!(benches, bench_decision_point_hot_path);
criterion_main!(benches);
