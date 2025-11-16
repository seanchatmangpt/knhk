//! Benchmark for atomic promotion operations
//!
//! This benchmark measures the critical hot-path operations to verify
//! they meet the ≤10 tick performance requirement.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use knhk_promotion::hot_path::*;

fn bench_load_current_descriptor(c: &mut Criterion) {
    init_hot_path();

    // Set up a descriptor
    let snapshot_id = [42u8; 32];
    let desc = HotPathDescriptor::new(snapshot_id, 12345, 1);
    store_descriptor(desc).unwrap();

    c.bench_function("load_current_descriptor", |b| {
        b.iter(|| {
            let desc = load_current_descriptor();
            black_box(desc);
        });
    });
}

fn bench_get_current_snapshot_id(c: &mut Criterion) {
    init_hot_path();

    let snapshot_id = [99u8; 32];
    let desc = HotPathDescriptor::new(snapshot_id, 9999, 10);
    store_descriptor(desc).unwrap();

    c.bench_function("get_current_snapshot_id", |b| {
        b.iter(|| {
            let id = get_current_snapshot_id();
            black_box(id);
        });
    });
}

fn bench_store_descriptor(c: &mut Criterion) {
    init_hot_path();

    let mut gen = 1;

    c.bench_function("store_descriptor", |b| {
        b.iter(|| {
            let snapshot_id = [gen as u8; 32];
            let desc = HotPathDescriptor::new(snapshot_id, gen as usize * 1000, gen);
            store_descriptor(desc).unwrap();
            gen += 1;
        });
    });
}

fn bench_atomic_swap_cost(c: &mut Criterion) {
    init_hot_path();

    let mut gen = 1;

    c.bench_function("atomic_swap_full_cycle", |b| {
        b.iter(|| {
            // Create new descriptor
            let snapshot_id = [gen as u8; 32];
            let new_desc = HotPathDescriptor::new(snapshot_id, gen as usize * 1000, gen);

            // Store (atomic swap)
            store_descriptor(new_desc).unwrap();

            // Verify with load
            let loaded = load_current_descriptor();
            black_box(loaded);

            gen += 1;
        });
    });
}

fn bench_concurrent_reads(c: &mut Criterion) {
    use std::thread;

    init_hot_path();

    let snapshot_id = [77u8; 32];
    let desc = HotPathDescriptor::new(snapshot_id, 7777, 100);
    store_descriptor(desc).unwrap();

    let mut group = c.benchmark_group("concurrent_reads");

    for thread_count in [1, 2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let handles: Vec<_> = (0..thread_count)
                        .map(|_| {
                            thread::spawn(|| {
                                for _ in 0..100 {
                                    let desc = load_current_descriptor();
                                    black_box(desc);
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_hot_path_cache_performance(c: &mut Criterion) {
    init_hot_path();

    let snapshot_id = [55u8; 32];
    let desc = HotPathDescriptor::new(snapshot_id, 5555, 50);
    store_descriptor(desc).unwrap();

    c.bench_function("hot_path_tight_loop", |b| {
        b.iter(|| {
            // Simulate tight loop in hot path (like many operators running)
            for _ in 0..1000 {
                let desc = load_current_descriptor();
                black_box(desc);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_load_current_descriptor,
    bench_get_current_snapshot_id,
    bench_store_descriptor,
    bench_atomic_swap_cost,
    bench_concurrent_reads,
    bench_hot_path_cache_performance,
);

criterion_main!(benches);
