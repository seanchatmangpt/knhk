// rust/knhk-etl/benches/hot_path_performance.rs
// Performance benchmarks for hot path operations
// Based on simdjson benchmarking practices: measure what matters, reproducible results

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use knhk_etl::load::{LoadResult, LoadStage, SoAArrays};
use knhk_etl::transform::{TransformResult, TypedTriple};
use knhk_hot::kernels::{KernelExecutor, KernelType};

/// Create test SoAArrays with specified number of triples
fn create_test_soa(num_triples: usize) -> SoAArrays {
    let mut soa = SoAArrays::new();
    for i in 0..num_triples.min(8) {
        soa.s[i] = (i + 1) as u64;
        soa.p[i] = 100; // Same predicate for all
        soa.o[i] = (i + 100) as u64;
    }
    soa
}

/// Benchmark hot path ASK_SP operation
fn bench_hot_path_ask_sp(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_path_ask_sp");

    // Benchmark different run lengths (1-8 triples)
    for num_triples in 1..=8 {
        let soa = create_test_soa(num_triples);

        group.bench_with_input(BenchmarkId::from_parameter(num_triples), &soa, |b, soa| {
            b.iter(|| {
                let result =
                    KernelExecutor::execute(KernelType::AskSp, &soa.s, &soa.p, &soa.o, num_triples);
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark hot path COUNT_SP_GE operation
fn bench_hot_path_count_sp_ge(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_path_count_sp_ge");

    for num_triples in 1..=8 {
        let soa = create_test_soa(num_triples);

        group.bench_with_input(BenchmarkId::from_parameter(num_triples), &soa, |b, soa| {
            b.iter(|| {
                let result = KernelExecutor::execute(
                    KernelType::CountSpGe,
                    &soa.s,
                    &soa.p,
                    &soa.o,
                    num_triples,
                );
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark branchless dispatch vs match-based dispatch
fn bench_dispatch_methods(c: &mut Criterion) {
    let soa = create_test_soa(8);

    let mut group = c.benchmark_group("dispatch_methods");

    group.bench_function("match_based", |b| {
        b.iter(|| {
            let result = KernelExecutor::execute(KernelType::AskSp, &soa.s, &soa.p, &soa.o, 8);
            black_box(result);
        });
    });

    group.bench_function("branchless_dispatch", |b| {
        b.iter(|| {
            let result =
                KernelExecutor::execute_dispatch(KernelType::AskSp, &soa.s, &soa.p, &soa.o, 8);
            black_box(result);
        });
    });

    group.finish();
}

/// Benchmark SoAArrays creation (memory allocation cost)
fn bench_soa_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("soa_creation");

    for num_triples in 1..=8 {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_triples),
            &num_triples,
            |b, &num_triples| {
                b.iter(|| {
                    let soa = create_test_soa(num_triples);
                    black_box(soa);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark load stage (triple to SoA conversion)
fn bench_load_stage(c: &mut Criterion) {
    let load_stage = LoadStage::new();

    let mut group = c.benchmark_group("load_stage");

    for num_triples in 1..=8 {
        let triples: Vec<TypedTriple> = (0..num_triples)
            .map(|i| TypedTriple {
                subject: (i + 1) as u64,
                predicate: 100,
                object: (i + 100) as u64,
                datatype: None,
            })
            .collect();

        let transform_result = TransformResult {
            typed_triples: triples,
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(num_triples),
            &transform_result,
            |b, input| {
                b.iter(|| {
                    let result = load_stage.load(black_box(input.clone()));
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_hot_path_ask_sp,
    bench_hot_path_count_sp_ge,
    bench_dispatch_methods,
    bench_soa_creation,
    bench_load_stage
);
criterion_main!(benches);
