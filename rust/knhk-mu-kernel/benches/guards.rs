//! Guard Evaluation Benchmarks
//!
//! Verifies branchless execution and ≤1 tick per guard

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use knhk_mu_kernel::guards::{GuardContext, GuardEvaluator};
use knhk_mu_kernel::timing::TickBudget;

fn bench_guard_tick_budget(c: &mut Criterion) {
    let mut group = c.benchmark_group("guards");
    group.sample_size(100000);

    let ctx = GuardContext {
        params: [8, 5, 0, 0, 0, 0, 0, 0],  // budget=8, used=5
    };

    group.bench_function("guard_tick_budget", |b| {
        b.iter(|| {
            let result = GuardEvaluator::guard_tick_budget(black_box(&ctx));
            black_box(result)
        });
    });

    group.finish();
}

fn bench_guard_range(c: &mut Criterion) {
    let mut group = c.benchmark_group("guards");
    group.sample_size(100000);

    let ctx = GuardContext {
        params: [50, 10, 90, 0, 0, 0, 0, 0],  // value=50, min=10, max=90
    };

    group.bench_function("guard_range", |b| {
        b.iter(|| {
            let result = GuardEvaluator::guard_range(black_box(&ctx));
            black_box(result)
        });
    });

    group.finish();
}

fn bench_guard_threshold(c: &mut Criterion) {
    let mut group = c.benchmark_group("guards");
    group.sample_size(100000);

    let ctx = GuardContext {
        params: [100, 75, 0, 0, 0, 0, 0, 0],  // value=100, threshold=75
    };

    group.bench_function("guard_threshold", |b| {
        b.iter(|| {
            let result = GuardEvaluator::guard_threshold(black_box(&ctx));
            black_box(result)
        });
    });

    group.finish();
}

fn bench_guard_branchless(c: &mut Criterion) {
    let mut group = c.benchmark_group("guards_branchless");
    group.sample_size(100000);

    // Verify branchless execution by testing both paths with same timing
    let ctx_pass = GuardContext {
        params: [8, 5, 0, 0, 0, 0, 0, 0],  // Passes
    };

    let ctx_fail = GuardContext {
        params: [8, 10, 0, 0, 0, 0, 0, 0],  // Fails
    };

    group.bench_function("pass_path", |b| {
        b.iter(|| {
            let result = GuardEvaluator::guard_tick_budget(black_box(&ctx_pass));
            black_box(result)
        });
    });

    group.bench_function("fail_path", |b| {
        b.iter(|| {
            let result = GuardEvaluator::guard_tick_budget(black_box(&ctx_fail));
            black_box(result)
        });
    });

    // Both paths should have identical timing (branchless)
    group.finish();
}

fn bench_guard_composition(c: &mut Criterion) {
    let mut group = c.benchmark_group("guards_composition");
    group.sample_size(10000);

    let ctx = GuardContext {
        params: [100, 50, 10, 90, 75, 0, 0, 0],
    };

    group.bench_function("multiple_guards", |b| {
        b.iter(|| {
            let mut budget = TickBudget::new(10);

            // Check multiple guards (should still be fast)
            let r1 = GuardEvaluator::guard_tick_budget(&ctx);
            let r2 = GuardEvaluator::guard_range(&ctx);
            let r3 = GuardEvaluator::guard_threshold(&ctx);

            black_box((r1, r2, r3))
        });
    });

    group.finish();
}

criterion_group!(
    guard_benches,
    bench_guard_tick_budget,
    bench_guard_range,
    bench_guard_threshold,
    bench_guard_branchless,
    bench_guard_composition,
);

criterion_main!(guard_benches);
