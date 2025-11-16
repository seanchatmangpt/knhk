//! μ-Operation Benchmarks
//!
//! Measures performance of individual μ-ops (should be ≤1 tick each)

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use knhk_mu_kernel::isa::{MuOps, DispatchPhase};
use knhk_mu_kernel::guards::GuardContext;

fn bench_load_sigma(c: &mut Criterion) {
    let mut group = c.benchmark_group("mu_ops");
    group.sample_size(100000);

    group.bench_function("load_sigma", |b| {
        b.iter(|| {
            let result = MuOps::load_sigma(black_box(0));
            black_box(result)
        });
    });

    group.finish();
}

fn bench_dispatch_pattern(c: &mut Criterion) {
    let mut group = c.benchmark_group("mu_ops");
    group.sample_size(100000);

    group.bench_function("dispatch_pattern", |b| {
        b.iter(|| {
            let result = MuOps::dispatch_pattern(
                black_box(0),
                black_box(DispatchPhase::Begin as u8),
            );
            black_box(result)
        });
    });

    group.finish();
}

fn bench_eval_guard(c: &mut Criterion) {
    let mut group = c.benchmark_group("mu_ops");
    group.sample_size(100000);

    let ctx = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    group.bench_function("eval_guard", |b| {
        b.iter(|| {
            let result = MuOps::eval_guard(black_box(0), black_box(&ctx));
            black_box(result)
        });
    });

    group.finish();
}

fn bench_emit_obs(c: &mut Criterion) {
    let mut group = c.benchmark_group("mu_ops");
    group.sample_size(100000);

    let ctx = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    group.bench_function("emit_obs", |b| {
        b.iter(|| {
            let result = MuOps::emit_obs(black_box(&ctx));
            black_box(result)
        });
    });

    group.finish();
}

fn bench_gen_receipt(c: &mut Criterion) {
    let mut group = c.benchmark_group("mu_ops");
    group.sample_size(10000);  // Slower operation

    group.bench_function("gen_receipt", |b| {
        b.iter(|| {
            let result = MuOps::gen_receipt(black_box(1));
            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(
    mu_op_benches,
    bench_load_sigma,
    bench_dispatch_pattern,
    bench_eval_guard,
    bench_emit_obs,
    bench_gen_receipt,
);

criterion_main!(mu_op_benches);
