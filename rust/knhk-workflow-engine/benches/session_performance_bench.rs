// benches/session_performance_bench.rs
//! Performance benchmarks for Session-Scoped Autonomic Runtime
//!
//! Measures:
//! - Session creation/update costs
//! - Lock-free atomic operations
//! - Session table scalability
//! - Aggregation performance with millions of sessions

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use knhk_workflow_engine::autonomic::{
    KnowledgeBase, SessionAdapter, SessionAggregator, SessionTable, TenantId,
};
use knhk_workflow_engine::case::CaseId;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

fn bench_session_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_creation");

    for size in [100, 1_000, 10_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let table = SessionTable::new();
                let tenant = TenantId::default_tenant();

                for _ in 0..size {
                    let case = CaseId::new();
                    let handle = table.create_session(case, tenant);
                    black_box(handle);
                }
            });
        });
    }

    group.finish();
}

fn bench_session_metrics_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_metrics_updates");

    let table = SessionTable::new();
    let case = CaseId::new();
    let handle = table.create_session(case, TenantId::default_tenant());

    group.bench_function("record_task_execution", |b| {
        b.iter(|| {
            handle.record_task_execution(black_box(Duration::from_micros(1000)));
        });
    });

    group.bench_function("record_retry", |b| {
        b.iter(|| {
            handle.record_retry();
        });
    });

    group.bench_function("record_violation", |b| {
        b.iter(|| {
            handle.record_violation();
        });
    });

    group.bench_function("snapshot", |b| {
        b.iter(|| {
            black_box(handle.snapshot());
        });
    });

    group.finish();
}

fn bench_session_table_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_table_lookup");

    for size in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            // Setup: Create table with many sessions
            let table = SessionTable::new();
            let tenant = TenantId::default_tenant();
            let mut ids = Vec::new();

            for _ in 0..size {
                let case = CaseId::new();
                let handle = table.create_session(case, tenant);
                ids.push(handle.id);
            }

            let lookup_id = ids[size / 2]; // Middle element

            b.iter(|| {
                black_box(table.get(&lookup_id));
            });
        });
    }

    group.finish();
}

fn bench_session_table_filtering(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_table_filtering");

    for size in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            // Setup: Create table with sessions from multiple tenants
            let table = SessionTable::new();
            let tenant1 = TenantId::new();
            let tenant2 = TenantId::new();

            for i in 0..size {
                let case = CaseId::new();
                let tenant = if i % 2 == 0 { tenant1 } else { tenant2 };
                table.create_session(case, tenant);
            }

            b.iter(|| {
                black_box(table.sessions_by_tenant(tenant1));
            });
        });
    }

    group.finish();
}

fn bench_session_aggregation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("session_aggregation");
    group.sample_size(10); // Reduce sample size for large benchmarks

    for size in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            // Setup
            let table = SessionTable::new();
            let tenant = TenantId::default_tenant();
            let mut handles = Vec::new();

            for _ in 0..size {
                let case = CaseId::new();
                let handle = table.create_session(case, tenant);
                handle.start();
                handle.record_task_execution(Duration::from_micros(1000));
                handle.complete();
                handles.push(handle);
            }

            let kb = Arc::new(KnowledgeBase::new());
            let aggregator = SessionAggregator::new(kb);

            b.to_async(&rt).iter(|| async {
                black_box(aggregator.aggregate_sessions(&handles).await.unwrap());
            });
        });
    }

    group.finish();
}

fn bench_session_adaptation_analysis(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("session_adaptation_analysis");

    let table = SessionTable::new();
    let case = CaseId::new();
    let handle = table.create_session(case, TenantId::default_tenant());
    handle.start();

    // Trigger adaptation need
    for _ in 0..5 {
        handle.record_retry();
    }

    let kb = Arc::new(KnowledgeBase::new());
    let adapter = SessionAdapter::new(kb);

    group.bench_function("analyze_session", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(adapter.analyze_session(&handle).await.unwrap());
        });
    });

    group.finish();
}

fn bench_concurrent_session_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_session_operations");

    for concurrency in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*concurrency as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let table = Arc::new(SessionTable::new());
                    let mut tasks = Vec::new();

                    for _ in 0..concurrency {
                        let table_clone = table.clone();
                        let task = tokio::spawn(async move {
                            let case = CaseId::new();
                            let handle =
                                table_clone.create_session(case, TenantId::default_tenant());
                            handle.start();
                            handle.record_task_execution(Duration::from_micros(1000));
                            handle.complete();
                        });
                        tasks.push(task);
                    }

                    for task in tasks {
                        task.await.unwrap();
                    }

                    black_box(table);
                });
            },
        );
    }

    group.finish();
}

fn bench_session_cleanup(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_cleanup");

    for size in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    // Setup: Create table with completed sessions
                    let table = SessionTable::new();
                    let tenant = TenantId::default_tenant();

                    for _ in 0..size {
                        let case = CaseId::new();
                        let handle = table.create_session(case, tenant);
                        handle.start();
                        handle.complete();
                    }

                    // Sleep to ensure sessions are old
                    std::thread::sleep(Duration::from_millis(10));

                    table
                },
                |table| {
                    // Cleanup
                    black_box(table.cleanup_terminal_sessions(Duration::from_millis(5)));
                },
                criterion::BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

fn bench_memory_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_overhead");

    group.bench_function("session_handle_size", |b| {
        b.iter(|| {
            black_box(std::mem::size_of::<
                knhk_workflow_engine::autonomic::SessionHandle,
            >());
        });
    });

    group.bench_function("session_metrics_size", |b| {
        b.iter(|| {
            black_box(std::mem::size_of::<
                knhk_workflow_engine::autonomic::SessionMetrics,
            >());
        });
    });

    group.bench_function("session_snapshot_size", |b| {
        b.iter(|| {
            black_box(std::mem::size_of::<
                knhk_workflow_engine::autonomic::SessionMetricsSnapshot,
            >());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_session_creation,
    bench_session_metrics_updates,
    bench_session_table_lookup,
    bench_session_table_filtering,
    bench_session_aggregation,
    bench_session_adaptation_analysis,
    bench_concurrent_session_operations,
    bench_session_cleanup,
    bench_memory_overhead,
);
criterion_main!(benches);
