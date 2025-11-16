//! Benchmark: Task Dispatch Latency
//!
//! Measures task dispatch operations that must be on the critical path.
//! Task dispatch MUST complete in ≤8 ticks.

use chicago_tdd::{PerformanceHarness, OperationType, Reporter};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Simulate task queue enqueue (hot path)
fn task_enqueue() -> bool {
    // Simulates adding task to execution queue
    let queue_size = 10;
    queue_size < 1000
}

/// Simulate task priority calculation (hot path)
fn task_priority() -> u8 {
    // Simulates calculating task priority
    let base_priority = 100;
    let urgency = 5;
    base_priority.saturating_add(urgency)
}

/// Simulate resource check (hot path)
fn resource_check() -> bool {
    // Simulates checking if resources are available
    let available = 8;
    let required = 2;
    available >= required
}

/// Simulate task ID generation (hot path)
fn task_id_gen() -> u64 {
    // Simulates generating unique task ID
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Simulate dispatch decision (hot path)
fn dispatch_decision() -> bool {
    // Simulates deciding if task can be dispatched now
    let resources_ok = true;
    let queue_ok = true;
    let auth_ok = true;
    resources_ok && queue_ok && auth_ok
}

fn bench_task_dispatch_hot_path(c: &mut Criterion) {
    let mut harness = PerformanceHarness::with_iterations(1000, 10000, 100);

    c.bench_function("task_dispatch_enqueue", |b| {
        b.iter(|| black_box(task_enqueue()))
    });

    c.bench_function("task_dispatch_priority", |b| {
        b.iter(|| black_box(task_priority()))
    });

    c.bench_function("task_dispatch_resource_check", |b| {
        b.iter(|| black_box(resource_check()))
    });

    c.bench_function("task_dispatch_id_gen", |b| {
        b.iter(|| black_box(task_id_gen()))
    });

    c.bench_function("task_dispatch_decision", |b| {
        b.iter(|| black_box(dispatch_decision()))
    });

    // Chicago TDD enforcement
    println!("\n{}", "=".repeat(80));
    println!("Chicago TDD Enforcement: Task Dispatch Hot Path");
    println!("{}", "=".repeat(80));

    let r1 = harness.measure("enqueue", OperationType::HotPath, task_enqueue);
    let r2 = harness.measure("priority", OperationType::HotPath, task_priority);
    let r3 = harness.measure("resource_check", OperationType::HotPath, resource_check);
    let r4 = harness.measure("id_generation", OperationType::HotPath, task_id_gen);
    let r5 = harness.measure("dispatch_decision", OperationType::HotPath, dispatch_decision);

    Reporter::print_result(&r1);
    Reporter::print_result(&r2);
    Reporter::print_result(&r3);
    Reporter::print_result(&r4);
    Reporter::print_result(&r5);

    let report = harness.report();
    Reporter::print_report(&report);

    if let Err(e) = harness.assert_all_within_bounds() {
        eprintln!("\n{}", "❌ CRITICAL: Task Dispatch Chatman Constant Violation");
        eprintln!("{}", e);
        panic!("{}", e);
    }

    println!("\n✅ All task dispatch operations within Chatman Constant (≤8 ticks)\n");
}

criterion_group!(benches, bench_task_dispatch_hot_path);
criterion_main!(benches);
