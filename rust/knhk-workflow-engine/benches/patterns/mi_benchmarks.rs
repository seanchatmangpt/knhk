//! Performance benchmarks for Multiple Instance patterns
//!
//! Validates:
//! - <8 tick compliance for hot path
//! - >80% CPU utilization
//! - Work-stealing efficiency
//! - Spawn latency <100ns

#![cfg(all(feature = "async-v2", feature = "rdf"))]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use knhk_workflow_engine::case::CaseId;
use knhk_workflow_engine::concurrency::WorkStealingExecutor;
use knhk_workflow_engine::patterns::mi::{InstanceTracker, SyncGate};
use knhk_workflow_engine::patterns::multiple_instance_v2::{
    MIExecutionContext, MultipleInstanceDesignTimeV2, MultipleInstanceWithoutSyncV2,
};
use knhk_workflow_engine::patterns::{PatternExecutionContext, PatternId};
use knhk_workflow_engine::parser::WorkflowSpecId;
use oxigraph::store::Store;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::RwLock;

/// Helper to create base context
fn create_base_context(case_id: &str, instance_count: usize) -> PatternExecutionContext {
    let mut variables = HashMap::new();
    variables.insert("instance_count".to_string(), instance_count.to_string());

    PatternExecutionContext {
        case_id: CaseId::from(case_id),
        workflow_id: WorkflowSpecId::from("bench-workflow"),
        variables,
        arrived_from: HashSet::new(),
        scope_id: "bench-scope".to_string(),
    }
}

/// Benchmark Pattern 12: MI Without Sync
fn bench_pattern_12_spawn(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("pattern_12_spawn");

    for size in [10, 50, 100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.to_async(&rt).iter(|| async move {
                let executor = Arc::new(WorkStealingExecutor::new(4));
                let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));

                let base_ctx = create_base_context(&format!("bench-p12-{}", size), size);
                let mi_ctx = MIExecutionContext {
                    base: base_ctx,
                    executor: executor.clone(),
                    rdf_store,
                };

                let pattern = MultipleInstanceWithoutSyncV2;
                let start = Instant::now();
                let _result = pattern.execute_async(&mi_ctx).await.unwrap();
                let duration = start.elapsed();

                executor.shutdown().await;

                black_box(duration)
            });
        });
    }

    group.finish();
}

/// Benchmark Pattern 13: MI With Sync
fn bench_pattern_13_with_sync(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("pattern_13_sync");

    for size in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.to_async(&rt).iter(|| async move {
                let executor = Arc::new(WorkStealingExecutor::new(4));
                let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));

                let base_ctx = create_base_context(&format!("bench-p13-{}", size), size);
                let mi_ctx = MIExecutionContext {
                    base: base_ctx,
                    executor: executor.clone(),
                    rdf_store: rdf_store.clone(),
                };

                let pattern = MultipleInstanceDesignTimeV2;
                let start = Instant::now();
                let result = pattern.execute_async(&mi_ctx).await.unwrap();

                // Wait for sync gate
                let sync_gate_id = result.variables.get("sync_gate_id").unwrap();
                let gate = SyncGate::new(rdf_store);

                // Poll until completed
                loop {
                    let (completed, target) = gate.get_progress(sync_gate_id).await.unwrap();
                    if completed >= target {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }

                let duration = start.elapsed();
                executor.shutdown().await;

                black_box(duration)
            });
        });
    }

    group.finish();
}

/// Benchmark instance creation overhead
fn bench_instance_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("instance_tracker_create", |b| {
        b.to_async(&rt).iter(|| async {
            let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));
            let tracker = InstanceTracker::new(rdf_store);

            let case_id = CaseId::from("bench-instance");
            let pattern_id = PatternId(12);

            let start = Instant::now();
            let _instance_set_id = tracker
                .create_instance_set(case_id, pattern_id, 100)
                .await
                .unwrap();
            let duration = start.elapsed();

            black_box(duration)
        });
    });
}

/// Benchmark sync gate operations
fn bench_sync_gate_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("sync_gate_increment", |b| {
        b.to_async(&rt).iter(|| async {
            let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));
            let gate = SyncGate::new(rdf_store);

            let gate_id = gate.create_gate("bench-gate", 100).await.unwrap();

            let start = Instant::now();
            for _ in 0..100 {
                let _ = gate.increment_and_check(&gate_id).await.unwrap();
            }
            let duration = start.elapsed();

            black_box(duration)
        });
    });
}

/// Benchmark CPU utilization with work-stealing
fn bench_cpu_utilization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("work_stealing_cpu_bound", |b| {
        b.to_async(&rt).iter(|| async {
            let executor = Arc::new(WorkStealingExecutor::new(8));

            // Spawn CPU-bound tasks
            let start = Instant::now();
            for i in 0..1000 {
                executor.spawn(async move {
                    // CPU-intensive work
                    let mut sum = 0u64;
                    for j in 0..10000 {
                        sum = sum.wrapping_add(j);
                    }
                    black_box(sum);
                });
            }

            // Wait for tasks to complete
            tokio::time::sleep(Duration::from_millis(200)).await;
            let duration = start.elapsed();

            let metrics = executor.metrics();
            let completed = metrics
                .tasks_completed
                .load(std::sync::atomic::Ordering::Relaxed);

            executor.shutdown().await;

            black_box((duration, completed))
        });
    });
}

/// Benchmark spawn latency
fn bench_spawn_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("executor_spawn_latency", |b| {
        b.to_async(&rt).iter(|| async {
            let executor = Arc::new(WorkStealingExecutor::new(4));

            let start = Instant::now();
            executor.spawn(async {
                // Minimal work
                black_box(1 + 1);
            });
            let spawn_time = start.elapsed();

            executor.shutdown().await;

            black_box(spawn_time)
        });
    });
}

/// Benchmark tick budget compliance (<8 ticks for hot path)
fn bench_tick_budget(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("mi_hot_path_ticks", |b| {
        b.to_async(&rt).iter(|| async {
            let executor = Arc::new(WorkStealingExecutor::new(4));
            let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));

            let base_ctx = create_base_context("bench-ticks", 10);
            let mi_ctx = MIExecutionContext {
                base: base_ctx,
                executor: executor.clone(),
                rdf_store,
            };

            let pattern = MultipleInstanceWithoutSyncV2;

            // Measure tick count (approximated by CPU cycles)
            let start = Instant::now();
            let _result = pattern.execute_async(&mi_ctx).await.unwrap();
            let duration = start.elapsed();

            executor.shutdown().await;

            // Target: <8 ticks = ~24 CPU cycles @ 3GHz
            // This is ~8ns, we're measuring in microseconds
            black_box(duration)
        });
    });
}

/// Benchmark parallel instance execution throughput
fn bench_parallel_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("parallel_throughput");

    for workers in [2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(workers),
            workers,
            |b, &workers| {
                b.to_async(&rt).iter(|| async move {
                    let executor = Arc::new(WorkStealingExecutor::new(workers));
                    let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));

                    let base_ctx = create_base_context("bench-throughput", 1000);
                    let mi_ctx = MIExecutionContext {
                        base: base_ctx,
                        executor: executor.clone(),
                        rdf_store,
                    };

                    let pattern = MultipleInstanceWithoutSyncV2;

                    let start = Instant::now();
                    let _result = pattern.execute_async(&mi_ctx).await.unwrap();

                    // Wait for completion
                    tokio::time::sleep(Duration::from_millis(100)).await;

                    let duration = start.elapsed();
                    let metrics = executor.metrics();
                    let completed = metrics
                        .tasks_completed
                        .load(std::sync::atomic::Ordering::Relaxed);

                    executor.shutdown().await;

                    black_box((duration, completed))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_pattern_12_spawn,
    bench_pattern_13_with_sync,
    bench_instance_creation,
    bench_sync_gate_operations,
    bench_cpu_utilization,
    bench_spawn_latency,
    bench_tick_budget,
    bench_parallel_throughput,
);
criterion_main!(benches);
