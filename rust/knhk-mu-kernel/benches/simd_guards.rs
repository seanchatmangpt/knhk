//! SIMD Guard Evaluation Benchmarks
//!
//! Measures performance of SIMD-accelerated guard evaluation vs scalar fallback.
//!
//! # Performance Targets
//!
//! - SIMD: ~2-3 cycles for 8 guards (0.25-0.375 cycles/guard)
//! - Scalar: ~8 cycles for 8 guards (1 cycle/guard)
//! - Speedup: 3-4x
//!
//! # Benchmarks
//!
//! - `simd_batch_evaluate`: Full SIMD batch evaluation (8 guards)
//! - `scalar_batch_evaluate`: Scalar fallback evaluation (8 guards)
//! - `simd_range_check`: SIMD range checking
//! - `simd_threshold`: SIMD threshold comparisons
//! - `simd_bitmask`: SIMD bitmask operations
//! - `batch_conversion`: AoS to SoA conversion overhead

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use knhk_mu_kernel::guards_simd::{
    SimdGuardBatch, SimdGuardEvaluator, evaluate_guards_batch,
    GuardBitmap, SIMD_BATCH_SIZE,
};
use knhk_mu_kernel::guards_simd::vectorized::{
    simd_range_check, simd_threshold_ge, simd_threshold_le,
    simd_equals, simd_bitmask_check, simd_select,
};
use knhk_mu_kernel::guards_simd::fallback::{
    evaluate_scalar, range_check_scalar, threshold_ge_scalar,
    threshold_le_scalar, equals_scalar, bitmask_check_scalar,
    select_scalar, DynamicGuardEvaluator,
};
use knhk_mu_kernel::guards_simd::layout::{
    GuardBatchPool, AosToSoaConverter, CacheAlignedBatch, MemoryStats,
};
use knhk_mu_kernel::guards::GuardContext;

/// Create a test guard batch with range checks
fn create_test_batch() -> SimdGuardBatch {
    SimdGuardBatch {
        values: [5, 10, 15, 20, 25, 30, 35, 40],
        mins: [0, 5, 10, 15, 20, 25, 30, 35],
        maxs: [10, 15, 20, 25, 30, 35, 40, 45],
    }
}

/// Benchmark: SIMD batch evaluation (target: ~2-3 cycles)
fn bench_simd_batch_evaluate(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_batch_evaluate");
    group.throughput(Throughput::Elements(SIMD_BATCH_SIZE as u64));

    let batch = create_test_batch();

    group.bench_function("simd_batch_8_guards", |b| {
        b.iter(|| {
            let result = black_box(&batch).evaluate();
            black_box(result);
        });
    });

    group.finish();
}

/// Benchmark: Scalar batch evaluation (baseline: ~8 cycles)
fn bench_scalar_batch_evaluate(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalar_batch_evaluate");
    group.throughput(Throughput::Elements(SIMD_BATCH_SIZE as u64));

    let batch = create_test_batch();

    group.bench_function("scalar_batch_8_guards", |b| {
        b.iter(|| {
            let result = evaluate_scalar(black_box(&batch));
            black_box(result);
        });
    });

    group.finish();
}

/// Benchmark: SIMD vs Scalar comparison
fn bench_simd_vs_scalar(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_vs_scalar");
    group.throughput(Throughput::Elements(SIMD_BATCH_SIZE as u64));

    let batch = create_test_batch();

    group.bench_function("simd", |b| {
        b.iter(|| {
            let result = black_box(&batch).evaluate();
            black_box(result);
        });
    });

    group.bench_function("scalar", |b| {
        b.iter(|| {
            let result = evaluate_scalar(black_box(&batch));
            black_box(result);
        });
    });

    group.finish();
}

/// Benchmark: SIMD range check operations
fn bench_simd_range_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_range_check");
    group.throughput(Throughput::Elements(SIMD_BATCH_SIZE as u64));

    let values = [5, 10, 15, 20, 25, 30, 35, 40];
    let mins = [0, 5, 10, 15, 20, 25, 30, 35];
    let maxs = [10, 15, 20, 25, 30, 35, 40, 45];

    group.bench_function("simd_range_check", |b| {
        b.iter(|| {
            let result = simd_range_check(
                black_box(&values),
                black_box(&mins),
                black_box(&maxs),
            );
            black_box(result);
        });
    });

    group.bench_function("scalar_range_check", |b| {
        b.iter(|| {
            let result = range_check_scalar(
                black_box(&values),
                black_box(&mins),
                black_box(&maxs),
            );
            black_box(result);
        });
    });

    group.finish();
}

/// Benchmark: SIMD threshold comparisons
fn bench_simd_threshold(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_threshold");
    group.throughput(Throughput::Elements(SIMD_BATCH_SIZE as u64));

    let values = [5, 10, 15, 20, 25, 30, 35, 40];
    let threshold = 20;

    group.bench_function("simd_threshold_ge", |b| {
        b.iter(|| {
            let result = simd_threshold_ge(black_box(&values), black_box(threshold));
            black_box(result);
        });
    });

    group.bench_function("scalar_threshold_ge", |b| {
        b.iter(|| {
            let result = threshold_ge_scalar(black_box(&values), black_box(threshold));
            black_box(result);
        });
    });

    group.bench_function("simd_threshold_le", |b| {
        b.iter(|| {
            let result = simd_threshold_le(black_box(&values), black_box(threshold));
            black_box(result);
        });
    });

    group.bench_function("scalar_threshold_le", |b| {
        b.iter(|| {
            let result = threshold_le_scalar(black_box(&values), black_box(threshold));
            black_box(result);
        });
    });

    group.finish();
}

/// Benchmark: SIMD equality checks
fn bench_simd_equals(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_equals");
    group.throughput(Throughput::Elements(SIMD_BATCH_SIZE as u64));

    let values = [1, 2, 3, 4, 5, 6, 7, 8];
    let expected = [1, 2, 3, 4, 5, 6, 7, 8];

    group.bench_function("simd_equals", |b| {
        b.iter(|| {
            let result = simd_equals(black_box(&values), black_box(&expected));
            black_box(result);
        });
    });

    group.bench_function("scalar_equals", |b| {
        b.iter(|| {
            let result = equals_scalar(black_box(&values), black_box(&expected));
            black_box(result);
        });
    });

    group.finish();
}

/// Benchmark: SIMD bitmask operations
fn bench_simd_bitmask(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_bitmask");
    group.throughput(Throughput::Elements(SIMD_BATCH_SIZE as u64));

    let values = [0xFF, 0xAA, 0x55, 0xF0, 0x0F, 0x99, 0x66, 0xFF];
    let masks = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    let expected = [0xFF, 0xAA, 0x55, 0xF0, 0x0F, 0x99, 0x66, 0xFF];

    group.bench_function("simd_bitmask_check", |b| {
        b.iter(|| {
            let result = simd_bitmask_check(
                black_box(&values),
                black_box(&masks),
                black_box(&expected),
            );
            black_box(result);
        });
    });

    group.bench_function("scalar_bitmask_check", |b| {
        b.iter(|| {
            let result = bitmask_check_scalar(
                black_box(&values),
                black_box(&masks),
                black_box(&expected),
            );
            black_box(result);
        });
    });

    group.finish();
}

/// Benchmark: SIMD select operations
fn bench_simd_select(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_select");
    group.throughput(Throughput::Elements(SIMD_BATCH_SIZE as u64));

    let mask = 0b10101010;
    let true_vals = [1, 1, 1, 1, 1, 1, 1, 1];
    let false_vals = [0, 0, 0, 0, 0, 0, 0, 0];

    group.bench_function("simd_select", |b| {
        b.iter(|| {
            let result = simd_select(
                black_box(mask),
                black_box(&true_vals),
                black_box(&false_vals),
            );
            black_box(result);
        });
    });

    group.bench_function("scalar_select", |b| {
        b.iter(|| {
            let result = select_scalar(
                black_box(mask),
                black_box(&true_vals),
                black_box(&false_vals),
            );
            black_box(result);
        });
    });

    group.finish();
}

/// Benchmark: Guard evaluator with batching
fn bench_guard_evaluator(c: &mut Criterion) {
    let mut group = c.benchmark_group("guard_evaluator");

    for count in [8, 16, 32, 64, 128].iter() {
        group.throughput(Throughput::Elements(*count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                b.iter(|| {
                    let mut evaluator = SimdGuardEvaluator::new();
                    for i in 0..count {
                        let _ = evaluator.add_guard(i as u64, 0, 100);
                    }
                    let result = evaluator.flush();
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Batch conversion (AoS to SoA)
fn bench_batch_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_conversion");

    for count in [8, 16, 32, 64].iter() {
        group.throughput(Throughput::Elements(*count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                let contexts: Vec<GuardContext> = (0..count)
                    .map(|i| GuardContext {
                        task_id: i as u64,
                        obs_data: 0,
                        params: [i as u64 * 10, 0, 100, 0],
                    })
                    .collect();

                b.iter(|| {
                    let mut converter = AosToSoaConverter::new();
                    for ctx in &contexts {
                        converter.add_context(black_box(ctx), 0, 1, 2);
                    }
                    let batches = converter.finish();
                    black_box(batches);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Guard batch pool operations
fn bench_batch_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_pool");

    let batch = create_test_batch();

    group.bench_function("pool_add_batch", |b| {
        b.iter(|| {
            let mut pool = GuardBatchPool::new();
            for _ in 0..100 {
                pool.add_batch(black_box(&batch));
            }
            black_box(pool);
        });
    });

    group.bench_function("pool_get_batch", |b| {
        let mut pool = GuardBatchPool::new();
        for _ in 0..100 {
            pool.add_batch(&batch);
        }

        b.iter(|| {
            for i in 0..100 {
                let result = pool.get_batch(black_box(i));
                black_box(result);
            }
        });
    });

    group.finish();
}

/// Benchmark: Dynamic dispatch overhead
fn bench_dynamic_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("dynamic_dispatch");
    group.throughput(Throughput::Elements(SIMD_BATCH_SIZE as u64));

    let batch = create_test_batch();
    let evaluator = DynamicGuardEvaluator::new();

    group.bench_function("dynamic_evaluate", |b| {
        b.iter(|| {
            let result = evaluator.evaluate(black_box(&batch));
            black_box(result);
        });
    });

    group.bench_function("direct_evaluate", |b| {
        b.iter(|| {
            let result = black_box(&batch).evaluate();
            black_box(result);
        });
    });

    group.finish();
}

/// Benchmark: Batch evaluation with varying guard counts
fn bench_varying_guard_counts(c: &mut Criterion) {
    let mut group = c.benchmark_group("varying_guard_counts");

    for count in [1, 2, 4, 8, 16, 32, 64, 128].iter() {
        group.throughput(Throughput::Elements(*count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                let guards: Vec<(u64, u64, u64)> = (0..count)
                    .map(|i| (i as u64, 0, 100))
                    .collect();

                b.iter(|| {
                    let result = evaluate_guards_batch(black_box(&guards));
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Cache-aligned batch access
fn bench_cache_aligned_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_aligned_batch");
    group.throughput(Throughput::Elements(SIMD_BATCH_SIZE as u64));

    let aligned = CacheAlignedBatch::from_batch(create_test_batch());
    let unaligned = create_test_batch();

    group.bench_function("aligned_access", |b| {
        b.iter(|| {
            let result = black_box(&aligned.batch).evaluate();
            black_box(result);
        });
    });

    group.bench_function("unaligned_access", |b| {
        b.iter(|| {
            let result = black_box(&unaligned).evaluate();
            black_box(result);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simd_batch_evaluate,
    bench_scalar_batch_evaluate,
    bench_simd_vs_scalar,
    bench_simd_range_check,
    bench_simd_threshold,
    bench_simd_equals,
    bench_simd_bitmask,
    bench_simd_select,
    bench_guard_evaluator,
    bench_batch_conversion,
    bench_batch_pool,
    bench_dynamic_dispatch,
    bench_varying_guard_counts,
    bench_cache_aligned_batch,
);

criterion_main!(benches);
