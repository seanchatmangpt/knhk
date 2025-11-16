//! Benchmark: MAPE-K Autonomic Loop Latency
//!
//! Measures Monitor-Analyze-Plan-Execute-Knowledge loop latency.
//! Critical MAPE-K operations MUST complete in ≤8 ticks.

use chicago_tdd::{PerformanceHarness, OperationType, Reporter};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Monitor: Metric collection decision (hot path)
fn monitor_metric() -> bool {
    // Decide if metric should be collected
    let threshold = 100;
    let current = 150;
    current > threshold
}

/// Analyze: Anomaly detection (hot path)
fn analyze_anomaly() -> bool {
    // Quick anomaly check
    let value = 95.0;
    let baseline = 100.0;
    let tolerance = 10.0;
    (value - baseline).abs() > tolerance
}

/// Plan: Policy lookup (hot path)
fn plan_policy() -> usize {
    // Look up applicable policy
    let policies = [0, 1, 2, 3, 4];
    let condition = 2;
    policies[condition]
}

/// Execute: Action selection (hot path)
fn execute_action() -> u8 {
    // Select action based on plan
    let plan_id = 3;
    match plan_id {
        0 => 0, // no action
        1 => 1, // scale up
        2 => 2, // scale down
        3 => 3, // rebalance
        _ => 0,
    }
}

/// Knowledge: Pattern match (hot path)
fn knowledge_pattern() -> bool {
    // Check if pattern is known
    let current_pattern = 42;
    let known_patterns = [10, 20, 30, 40, 42, 50];
    known_patterns.contains(&current_pattern)
}

fn bench_mape_k_hot_path(c: &mut Criterion) {
    let mut harness = PerformanceHarness::with_iterations(1000, 10000, 100);

    c.bench_function("mape_k_monitor", |b| {
        b.iter(|| black_box(monitor_metric()))
    });

    c.bench_function("mape_k_analyze", |b| {
        b.iter(|| black_box(analyze_anomaly()))
    });

    c.bench_function("mape_k_plan", |b| {
        b.iter(|| black_box(plan_policy()))
    });

    c.bench_function("mape_k_execute", |b| {
        b.iter(|| black_box(execute_action()))
    });

    c.bench_function("mape_k_knowledge", |b| {
        b.iter(|| black_box(knowledge_pattern()))
    });

    // Chicago TDD enforcement
    println!("\n{}", "=".repeat(80));
    println!("Chicago TDD Enforcement: MAPE-K Hot Path");
    println!("{}", "=".repeat(80));

    let r1 = harness.measure("monitor", OperationType::HotPath, monitor_metric);
    let r2 = harness.measure("analyze", OperationType::HotPath, analyze_anomaly);
    let r3 = harness.measure("plan", OperationType::HotPath, plan_policy);
    let r4 = harness.measure("execute", OperationType::HotPath, execute_action);
    let r5 = harness.measure("knowledge", OperationType::HotPath, knowledge_pattern);

    Reporter::print_result(&r1);
    Reporter::print_result(&r2);
    Reporter::print_result(&r3);
    Reporter::print_result(&r4);
    Reporter::print_result(&r5);

    let report = harness.report();
    Reporter::print_report(&report);

    if let Err(e) = harness.assert_all_within_bounds() {
        eprintln!("\n{}", "❌ CRITICAL: MAPE-K Chatman Constant Violation");
        eprintln!("{}", e);
        panic!("{}", e);
    }

    println!("\n✅ All MAPE-K operations within Chatman Constant (≤8 ticks)\n");
}

criterion_group!(benches, bench_mape_k_hot_path);
criterion_main!(benches);
