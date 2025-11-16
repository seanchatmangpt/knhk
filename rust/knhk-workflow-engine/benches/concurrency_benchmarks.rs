//! Performance benchmarks for concurrency primitives
//!
//! Target: <100ns task spawn latency (P99)

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "async-v2")]
use knhk_workflow_engine::concurrency::{CancelToken, Nursery, WorkStealingExecutor, WorkerConfig};

#[cfg(feature = "async-v2")]
fn bench_nursery_spawn(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("nursery_spawn_single_task", |b| {
        b.to_async(&runtime).iter(|| async {
            let mut nursery = Nursery::new();
            nursery.spawn(async { Ok(()) }).await;
            nursery.wait_all().await.unwrap();
        });
    });

    c.bench_function("nursery_spawn_10_tasks", |b| {
        b.to_async(&runtime).iter(|| async {
            let mut nursery = Nursery::new();
            for _ in 0..10 {
                nursery.spawn(async { Ok(()) }).await;
            }
            nursery.wait_all().await.unwrap();
        });
    });

    c.bench_function("nursery_spawn_100_tasks", |b| {
        b.to_async(&runtime).iter(|| async {
            let mut nursery = Nursery::new();
            for _ in 0..100 {
                nursery.spawn(async { Ok(()) }).await;
            }
            nursery.wait_all().await.unwrap();
        });
    });
}

#[cfg(feature = "async-v2")]
fn bench_cancel_token(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("cancel_token_create", |b| {
        b.iter(|| {
            let token = CancelToken::new();
            black_box(token);
        });
    });

    c.bench_function("cancel_token_child", |b| {
        let token = CancelToken::new();
        b.iter(|| {
            let child = token.child_token();
            black_box(child);
        });
    });

    c.bench_function("cancel_token_cancel", |b| {
        b.iter(|| {
            let token = CancelToken::new();
            token.cancel();
            black_box(token.is_cancelled());
        });
    });

    c.bench_function("cancel_token_is_cancelled", |b| {
        let token = CancelToken::new();
        b.iter(|| {
            black_box(token.is_cancelled());
        });
    });

    c.bench_function("cancel_token_hierarchical_check", |b| {
        let parent = CancelToken::new();
        let child = parent.child_token();
        let grandchild = child.child_token();

        b.iter(|| {
            black_box(grandchild.is_cancelled());
        });
    });
}

#[cfg(feature = "async-v2")]
fn bench_work_stealing_executor(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Benchmark task spawn latency (CRITICAL: must be <100ns)
    c.bench_function("work_stealing_spawn_latency", |b| {
        let executor = WorkStealingExecutor::new(4);
        let counter = Arc::new(AtomicUsize::new(0));

        b.iter(|| {
            let counter = counter.clone();
            executor.spawn(async move {
                counter.fetch_add(1, Ordering::Relaxed);
            });
        });

        runtime.block_on(async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            executor.shutdown().await;
        });
    });

    // Benchmark throughput
    for task_count in [10, 100, 1000].iter() {
        c.bench_with_input(
            BenchmarkId::new("work_stealing_throughput", task_count),
            task_count,
            |b, &count| {
                b.to_async(&runtime).iter(|| async move {
                    let executor = WorkStealingExecutor::new(4);
                    let counter = Arc::new(AtomicUsize::new(0));

                    for _ in 0..count {
                        let counter = counter.clone();
                        executor.spawn(async move {
                            counter.fetch_add(1, Ordering::Relaxed);
                        });
                    }

                    tokio::time::sleep(Duration::from_millis(50)).await;
                    executor.shutdown().await;

                    assert_eq!(counter.load(Ordering::Relaxed), count);
                });
            },
        );
    }

    // Benchmark CPU-bound work distribution
    c.bench_function("work_stealing_cpu_bound", |b| {
        b.to_async(&runtime).iter(|| async {
            let executor = WorkStealingExecutor::new(4);

            for i in 0..100 {
                executor.spawn(async move {
                    let mut sum = 0;
                    for j in 0..i * 10 {
                        sum += j;
                    }
                    black_box(sum);
                });
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
            executor.shutdown().await;
        });
    });
}

#[cfg(feature = "async-v2")]
fn bench_executor_comparison(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("executor_comparison");

    // Tokio baseline
    group.bench_function("tokio_spawn_100_tasks", |b| {
        b.to_async(&runtime).iter(|| async {
            let counter = Arc::new(AtomicUsize::new(0));
            let mut handles = Vec::new();

            for _ in 0..100 {
                let counter = counter.clone();
                handles.push(tokio::spawn(async move {
                    counter.fetch_add(1, Ordering::Relaxed);
                }));
            }

            for handle in handles {
                handle.await.unwrap();
            }

            assert_eq!(counter.load(Ordering::Relaxed), 100);
        });
    });

    // Work-stealing executor
    group.bench_function("work_stealing_spawn_100_tasks", |b| {
        b.to_async(&runtime).iter(|| async {
            let executor = WorkStealingExecutor::new(4);
            let counter = Arc::new(AtomicUsize::new(0));

            for _ in 0..100 {
                let counter = counter.clone();
                executor.spawn(async move {
                    counter.fetch_add(1, Ordering::Relaxed);
                });
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
            executor.shutdown().await;

            assert_eq!(counter.load(Ordering::Relaxed), 100);
        });
    });

    group.finish();
}

#[cfg(feature = "async-v2")]
criterion_group!(
    benches,
    bench_nursery_spawn,
    bench_cancel_token,
    bench_work_stealing_executor,
    bench_executor_comparison
);

#[cfg(not(feature = "async-v2"))]
criterion_group!(benches,);

criterion_main!(benches);
