//! Σ* Compilation Benchmark
//!
//! Measures Σ → Σ* compilation and loading performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use knhk_mu_kernel::sigma::{SigmaCompiled, SigmaHeader, SigmaHash, TaskDescriptor, GuardDescriptor, PatternBinding, SigmaPointer};
use knhk_mu_kernel::compiler::SigmaCompiler;

fn bench_sigma_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("sigma_load");
    group.sample_size(1000);

    let sigma = create_test_sigma();

    group.bench_function("load_sigma_ptr", |b| {
        let sigma_ptr = SigmaPointer::new();
        sigma_ptr.store(&sigma);

        b.iter(|| {
            let loaded = sigma_ptr.load();
            black_box(loaded)
        });
    });

    group.finish();
}

fn bench_sigma_atomic_swap(c: &mut Criterion) {
    let mut group = c.benchmark_group("sigma_swap");
    group.sample_size(1000);

    let sigma1 = create_test_sigma();
    let mut sigma2 = sigma1;
    sigma2.header.hash = SigmaHash([2; 32]);

    group.bench_function("atomic_swap", |b| {
        let sigma_ptr = SigmaPointer::new();
        sigma_ptr.store(&sigma1);

        b.iter(|| {
            sigma_ptr.store(black_box(&sigma2));
            let loaded = sigma_ptr.load();
            black_box(loaded)
        });
    });

    group.finish();
}

fn bench_sigma_hash_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("sigma_hash");
    group.sample_size(10000);

    let sigma = create_test_sigma();

    group.bench_function("compute_hash", |b| {
        b.iter(|| {
            let hash = sigma.hash();
            black_box(hash)
        });
    });

    group.finish();
}

fn bench_sigma_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("sigma_compile");
    group.sample_size(100);

    // Different Σ sizes
    for task_count in [10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("tasks", task_count),
            &task_count,
            |b, &count| {
                b.iter(|| {
                    let sigma = create_test_sigma_with_tasks(black_box(count));
                    black_box(sigma)
                });
            },
        );
    }

    group.finish();
}

fn bench_task_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("task_lookup");
    group.sample_size(100000);

    let sigma = create_test_sigma();

    group.bench_function("get_task", |b| {
        b.iter(|| {
            let task = sigma.get_task(black_box(0));
            black_box(task)
        });
    });

    group.finish();
}

fn bench_guard_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("guard_lookup");
    group.sample_size(100000);

    let sigma = create_test_sigma();

    group.bench_function("get_guard", |b| {
        b.iter(|| {
            let guard = sigma.get_guard(black_box(0));
            black_box(guard)
        });
    });

    group.finish();
}

// Helper functions
fn create_test_sigma() -> SigmaCompiled {
    create_test_sigma_with_tasks(10)
}

fn create_test_sigma_with_tasks(task_count: usize) -> SigmaCompiled {
    let mut sigma = SigmaCompiled {
        header: SigmaHeader {
            magic: knhk_mu_kernel::sigma::SIGMA_MAGIC,
            version: 1,
            hash: SigmaHash([1; 32]),
            tasks_offset: 0,
            guards_offset: 0,
            patterns_offset: 0,
            timestamp: 0,
            flags: 0,
            task_count: task_count as u64,
            guard_count: 10,
            pattern_count: 5,
            _reserved: [0; 24],
        },
        tasks: [TaskDescriptor::default(); 1024],
        guards: [GuardDescriptor::default(); 1024],
        patterns: [PatternBinding::default(); 256],
    };

    // Populate some test tasks
    for i in 0..task_count.min(1024) {
        sigma.tasks[i] = TaskDescriptor {
            task_id: i as u64,
            pattern_id: (i % 43) as u8,
            guard_count: 2,
            guards: [(i % 16) as u16, ((i + 1) % 16) as u16, 0, 0, 0, 0, 0, 0],
            tick_budget: 8,
            _reserved: [0; 7],
        };
    }

    sigma
}

criterion_group!(
    sigma_benches,
    bench_sigma_load,
    bench_sigma_atomic_swap,
    bench_sigma_hash_computation,
    bench_sigma_compilation,
    bench_task_lookup,
    bench_guard_lookup,
);

criterion_main!(sigma_benches);
