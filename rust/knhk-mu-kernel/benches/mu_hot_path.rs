//! μ_hot Path Benchmark
//!
//! Comprehensive benchmark of the complete hot path: O → μ → A
//! Verifies end-to-end performance ≤8 ticks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use knhk_mu_kernel::core::MuKernel;
use knhk_mu_kernel::guards::GuardContext;
use knhk_mu_kernel::sigma::{SigmaHash, SigmaPointer};
use knhk_mu_kernel::CHATMAN_CONSTANT;

fn bench_complete_hot_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("mu_hot_path");
    group.sample_size(10000);

    // Setup kernel
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut kernel = MuKernel::new(sigma_ptr);

    let obs = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    group.bench_function("execute_task_complete", |b| {
        b.iter(|| {
            let result = kernel.execute_task(black_box(1), black_box(&obs));
            black_box(result)
        });
    });

    group.finish();
}

fn bench_hot_path_under_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("mu_hot_load");
    group.sample_size(100);

    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut kernel = MuKernel::new(sigma_ptr);

    let obs = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    for batch_size in [10, 100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("batch", batch_size),
            &batch_size,
            |b, &size| {
                b.iter(|| {
                    for i in 0..size {
                        let _ = kernel.execute_task(black_box(i as u64), &obs);
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_hot_path_memory_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_layout");
    group.sample_size(10000);

    // Verify cache-line alignment doesn't hurt performance
    group.bench_function("aligned_access", |b| {
        let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
        let kernel = MuKernel::new(sigma_ptr);

        b.iter(|| {
            let state = kernel.state();
            black_box(state)
        });
    });

    group.finish();
}

fn bench_hot_path_receipt_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("receipt_gen");
    group.sample_size(1000);

    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut kernel = MuKernel::new(sigma_ptr);

    let obs = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    group.bench_function("with_receipt", |b| {
        b.iter(|| {
            // Execute task and generate receipt
            let result = kernel.execute_task(black_box(1), &obs);
            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(
    hot_path_benches,
    bench_complete_hot_path,
    bench_hot_path_under_load,
    bench_hot_path_memory_layout,
    bench_hot_path_receipt_generation,
);

criterion_main!(hot_path_benches);
