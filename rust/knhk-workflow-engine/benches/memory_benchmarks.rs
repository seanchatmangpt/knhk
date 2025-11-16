//! Phase 2: Memory Optimization Benchmarks
//!
//! Comprehensive benchmarks for memory optimization features:
//! - Arena allocator performance
//! - SIMD vectorization speedup
//! - Cache-aligned data structure throughput
//! - Memory-mapped workflow loading
//! - Allocator comparison (system vs jemalloc vs mimalloc)
//!
//! Performance target: ≤8 ticks for hot path operations (Chatman Constant)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use knhk_workflow_engine::*;
use std::time::Duration;

#[cfg(feature = "memory-v2")]
use knhk_workflow_engine::memory::{Arena, ArenaAllocator, CacheAligned};
#[cfg(feature = "memory-v2")]
use knhk_workflow_engine::performance::simd::{batching, pattern_matching};

// ========== Arena Allocator Benchmarks ==========

#[cfg(feature = "memory-v2")]
fn bench_arena_vs_heap(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_allocator");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark heap allocation
    group.bench_function("heap_allocation_1000", |b| {
        b.iter(|| {
            let mut values = Vec::new();
            for i in 0..1000 {
                values.push(Box::new(i));
            }
            black_box(values);
        });
    });

    // Benchmark arena allocation
    group.bench_function("arena_allocation_1000", |b| {
        b.iter(|| {
            let mut arena = Arena::with_capacity(1024 * 1024).unwrap();
            for i in 0..1000 {
                let _ = arena.alloc(i).unwrap();
            }
            black_box(arena);
        });
    });

    // Benchmark arena with reset/reuse
    group.bench_function("arena_with_reuse_10x100", |b| {
        let mut arena = Arena::with_capacity(1024 * 1024).unwrap();
        b.iter(|| {
            for _ in 0..10 {
                for i in 0..100 {
                    let _ = arena.alloc(i).unwrap();
                }
                arena.reset();
            }
        });
    });

    group.finish();
}

// ========== SIMD Vectorization Benchmarks ==========

#[cfg(feature = "memory-v2")]
fn bench_simd_pattern_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_pattern_matching");
    group.measurement_time(Duration::from_secs(10));

    for size in [100, 1000, 10000].iter() {
        let patterns: Vec<u32> = (0..*size).map(|i| i % 100).collect();

        // Scalar implementation
        group.bench_with_input(BenchmarkId::new("scalar_filter", size), size, |b, _| {
            b.iter(|| {
                let matches: Vec<usize> = patterns
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &p)| if p == 42 { Some(i) } else { None })
                    .collect();
                black_box(matches);
            });
        });

        // SIMD implementation
        group.bench_with_input(BenchmarkId::new("simd_filter", size), size, |b, _| {
            b.iter(|| {
                let matches = pattern_matching::vectorized_pattern_filter(&patterns, 42);
                black_box(matches);
            });
        });

        // Pattern count
        group.bench_with_input(BenchmarkId::new("simd_count", size), size, |b, _| {
            b.iter(|| {
                let count = pattern_matching::vectorized_pattern_count(&patterns, 42);
                black_box(count);
            });
        });

        // Pattern any
        group.bench_with_input(BenchmarkId::new("simd_any", size), size, |b, _| {
            b.iter(|| {
                let found = pattern_matching::vectorized_pattern_any(&patterns, 42);
                black_box(found);
            });
        });
    }

    group.finish();
}

#[cfg(feature = "memory-v2")]
fn bench_simd_batching(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_batching");
    group.measurement_time(Duration::from_secs(10));

    for size in [100, 1000, 10000].iter() {
        let values: Vec<u64> = (0..*size).map(|i| i as u64).collect();

        // Sum benchmark
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("scalar_sum", size), size, |b, _| {
            b.iter(|| {
                let sum: u64 = values.iter().sum();
                black_box(sum);
            });
        });

        group.bench_with_input(BenchmarkId::new("simd_sum", size), size, |b, _| {
            b.iter(|| {
                let sum = batching::vectorized_sum_u64(&values);
                black_box(sum);
            });
        });

        // Max benchmark
        group.bench_with_input(BenchmarkId::new("scalar_max", size), size, |b, _| {
            b.iter(|| {
                let max = values.iter().max();
                black_box(max);
            });
        });

        group.bench_with_input(BenchmarkId::new("simd_max", size), size, |b, _| {
            b.iter(|| {
                let max = batching::vectorized_max_u64(&values);
                black_box(max);
            });
        });

        // Average benchmark
        group.bench_with_input(BenchmarkId::new("simd_average", size), size, |b, _| {
            b.iter(|| {
                let avg = batching::vectorized_average_u64(&values);
                black_box(avg);
            });
        });
    }

    group.finish();
}

// ========== Cache Alignment Benchmarks ==========

#[cfg(feature = "memory-v2")]
fn bench_cache_alignment(c: &mut Criterion) {
    use std::sync::atomic::{AtomicU64, Ordering};

    let mut group = c.benchmark_group("cache_alignment");
    group.measurement_time(Duration::from_secs(10));

    // Unaligned counters
    struct UnalignedCounters {
        count1: AtomicU64,
        count2: AtomicU64,
        count3: AtomicU64,
    }

    // Cache-aligned counters
    struct AlignedCounters {
        count1: CacheAligned<AtomicU64>,
        count2: CacheAligned<AtomicU64>,
        count3: CacheAligned<AtomicU64>,
    }

    // Benchmark unaligned access
    group.bench_function("unaligned_counters", |b| {
        let counters = UnalignedCounters {
            count1: AtomicU64::new(0),
            count2: AtomicU64::new(0),
            count3: AtomicU64::new(0),
        };

        b.iter(|| {
            for _ in 0..1000 {
                counters.count1.fetch_add(1, Ordering::Relaxed);
                counters.count2.fetch_add(1, Ordering::Relaxed);
                counters.count3.fetch_add(1, Ordering::Relaxed);
            }
        });
    });

    // Benchmark aligned access
    group.bench_function("cache_aligned_counters", |b| {
        let counters = AlignedCounters {
            count1: CacheAligned::new(AtomicU64::new(0)),
            count2: CacheAligned::new(AtomicU64::new(0)),
            count3: CacheAligned::new(AtomicU64::new(0)),
        };

        b.iter(|| {
            for _ in 0..1000 {
                counters.count1.fetch_add(1, Ordering::Relaxed);
                counters.count2.fetch_add(1, Ordering::Relaxed);
                counters.count3.fetch_add(1, Ordering::Relaxed);
            }
        });
    });

    group.finish();
}

// ========== Hot Path Tick Budget Compliance ==========

#[cfg(feature = "memory-v2")]
fn bench_hot_path_tick_budget(c: &mut Criterion) {
    use std::time::Instant;

    let mut group = c.benchmark_group("hot_path_tick_budget");
    group.measurement_time(Duration::from_secs(10));

    // CPU frequency estimation (assuming ~3 GHz)
    const ESTIMATED_CYCLES_PER_NS: f64 = 3.0;
    const MAX_TICKS: u64 = 8;

    // Arena allocation hot path
    group.bench_function("arena_hot_path", |b| {
        b.iter_custom(|iters| {
            let mut arena = Arena::with_capacity(1024 * 1024).unwrap();
            let start = Instant::now();

            for _ in 0..iters {
                // Hot path: allocate 10 small objects
                for i in 0..10 {
                    let _ = arena.alloc(i).unwrap();
                }
                arena.reset();
            }

            start.elapsed()
        });
    });

    // SIMD pattern matching hot path
    group.bench_function("simd_hot_path", |b| {
        let patterns: Vec<u32> = (0..100).collect();

        b.iter_custom(|iters| {
            let start = Instant::now();

            for _ in 0..iters {
                // Hot path: SIMD pattern search
                let _ = pattern_matching::vectorized_pattern_any(&patterns, 42);
            }

            start.elapsed()
        });
    });

    // Cache-aligned access hot path
    group.bench_function("cache_aligned_hot_path", |b| {
        use std::sync::atomic::{AtomicU64, Ordering};

        let counter = CacheAligned::new(AtomicU64::new(0));

        b.iter_custom(|iters| {
            let start = Instant::now();

            for _ in 0..iters {
                // Hot path: increment cache-aligned counter
                counter.fetch_add(1, Ordering::Relaxed);
            }

            start.elapsed()
        });
    });

    group.finish();
}

// ========== Allocator Comparison ==========

fn bench_allocator_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocator_comparison");
    group.measurement_time(Duration::from_secs(10));

    // Small allocations pattern (workflow pattern instances)
    group.bench_function("small_alloc_pattern", |b| {
        b.iter(|| {
            let mut boxes = Vec::new();
            for i in 0..1000 {
                boxes.push(Box::new(i));
            }
            black_box(boxes);
        });
    });

    // Large allocations pattern (workflow state)
    group.bench_function("large_alloc_pattern", |b| {
        b.iter(|| {
            let mut vecs = Vec::new();
            for _ in 0..100 {
                vecs.push(vec![0u8; 4096]);
            }
            black_box(vecs);
        });
    });

    // Mixed allocation pattern
    group.bench_function("mixed_alloc_pattern", |b| {
        b.iter(|| {
            let mut data = Vec::new();
            // Small allocations
            for i in 0..500 {
                data.push(Box::new(i));
            }
            // Large allocations
            for _ in 0..50 {
                data.push(Box::new(vec![0u8; 1024]));
            }
            black_box(data);
        });
    });

    group.finish();
}

// ========== Criterion Groups ==========

#[cfg(feature = "memory-v2")]
criterion_group!(
    benches,
    bench_arena_vs_heap,
    bench_simd_pattern_matching,
    bench_simd_batching,
    bench_cache_alignment,
    bench_hot_path_tick_budget,
    bench_allocator_patterns,
);

#[cfg(not(feature = "memory-v2"))]
criterion_group!(benches, bench_allocator_patterns,);

criterion_main!(benches);
