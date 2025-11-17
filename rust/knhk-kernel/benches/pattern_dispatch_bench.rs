// knhk-kernel: Pattern dispatch performance benchmarks
// Measures dispatch latency for all 43 W3C patterns

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use knhk_kernel::{
    pattern::{PatternConfig, PatternDispatcher, PatternFactory, PatternType},
    timer::read_tsc,
};

fn bench_individual_patterns(c: &mut Criterion) {
    knhk_kernel::init().unwrap();

    let mut group = c.benchmark_group("pattern_dispatch_individual");
    group.significance_level(0.01);

    let dispatcher = PatternDispatcher::new();

    // Benchmark each of the 43 patterns
    for i in 1..=43u8 {
        let pattern_type = unsafe { std::mem::transmute::<u8, PatternType>(i) };

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", pattern_type)),
            &pattern_type,
            |b, &pt| {
                let ctx = PatternFactory::create(pt, i as u32, PatternConfig::default());

                b.iter(|| {
                    let result = dispatcher.dispatch(black_box(&ctx));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

fn bench_pattern_categories(c: &mut Criterion) {
    knhk_kernel::init().unwrap();

    let mut group = c.benchmark_group("pattern_categories");

    let dispatcher = PatternDispatcher::new();

    // Basic Control Flow (1-5)
    group.bench_function("basic_control_flow", |b| {
        let contexts: Vec<_> = (1..=5)
            .map(|i| {
                let pt = unsafe { std::mem::transmute::<u8, PatternType>(i) };
                PatternFactory::create(pt, i as u32, PatternConfig::default())
            })
            .collect();

        let mut idx = 0;
        b.iter(|| {
            let result = dispatcher.dispatch(black_box(&contexts[idx]));
            idx = (idx + 1) % 5;
            black_box(result)
        });
    });

    // Advanced Branching (6-9)
    group.bench_function("advanced_branching", |b| {
        let contexts: Vec<_> = (6..=9)
            .map(|i| {
                let pt = unsafe { std::mem::transmute::<u8, PatternType>(i) };
                PatternFactory::create(pt, i as u32, PatternConfig::default())
            })
            .collect();

        let mut idx = 0;
        b.iter(|| {
            let result = dispatcher.dispatch(black_box(&contexts[idx]));
            idx = (idx + 1) % 4;
            black_box(result)
        });
    });

    // Multiple Instance (10-15)
    group.bench_function("multiple_instance", |b| {
        let contexts: Vec<_> = (10..=15)
            .map(|i| {
                let pt = unsafe { std::mem::transmute::<u8, PatternType>(i) };
                PatternFactory::create(
                    pt,
                    i as u32,
                    PatternConfig {
                        max_instances: 4,
                        ..Default::default()
                    },
                )
            })
            .collect();

        let mut idx = 0;
        b.iter(|| {
            let result = dispatcher.dispatch(black_box(&contexts[idx]));
            idx = (idx + 1) % 6;
            black_box(result)
        });
    });

    group.finish();
}

fn bench_pattern_dispatch_ticks(c: &mut Criterion) {
    knhk_kernel::init().unwrap();

    let mut group = c.benchmark_group("pattern_dispatch_ticks");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(3));

    let dispatcher = PatternDispatcher::new();

    // Measure critical patterns in CPU ticks
    for pattern_id in [
        PatternType::Sequence as u8,
        PatternType::ParallelSplit as u8,
        PatternType::Synchronization as u8,
        PatternType::ExclusiveChoice as u8,
        PatternType::StructuredDiscriminator as u8,
    ] {
        let pattern_type = unsafe { std::mem::transmute::<u8, PatternType>(pattern_id) };

        group.bench_with_input(
            BenchmarkId::new("ticks", format!("{:?}", pattern_type)),
            &pattern_type,
            |b, &pt| {
                let ctx = PatternFactory::create(pt, pattern_id as u32, PatternConfig::default());

                b.iter_custom(|iters| {
                    let mut total_ticks = 0u64;

                    for _ in 0..iters {
                        let start = read_tsc();
                        let _result = dispatcher.dispatch(&ctx);
                        let end = read_tsc();
                        total_ticks += end - start;
                    }

                    std::time::Duration::from_nanos(total_ticks / iters)
                });
            },
        );
    }

    group.finish();
}

fn bench_pattern_throughput(c: &mut Criterion) {
    knhk_kernel::init().unwrap();

    let mut group = c.benchmark_group("pattern_throughput");
    group.throughput(Throughput::Elements(1));

    let dispatcher = PatternDispatcher::new();

    // Create contexts for round-robin dispatch
    let contexts: Vec<_> = (1..=10)
        .map(|i| {
            let pt = unsafe { std::mem::transmute::<u8, PatternType>(i) };
            PatternFactory::create(pt, i as u32, PatternConfig::default())
        })
        .collect();

    group.bench_function("mixed_patterns", |b| {
        let mut idx = 0;
        b.iter(|| {
            let result = dispatcher.dispatch(&contexts[idx]);
            idx = (idx + 1) % 10;
            black_box(result)
        });
    });

    group.finish();
}

fn bench_worst_case_patterns(c: &mut Criterion) {
    knhk_kernel::init().unwrap();

    let mut group = c.benchmark_group("worst_case_patterns");

    let dispatcher = PatternDispatcher::new();

    // Patterns known to be complex
    let complex_patterns = [
        PatternType::Recursion,
        PatternType::InterleavedParallelRouting,
        PatternType::GeneralizedAndJoin,
        PatternType::MultiInstanceUnknownRuntime,
    ];

    for pattern_type in complex_patterns {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", pattern_type)),
            &pattern_type,
            |b, &pt| {
                let ctx = PatternFactory::create(
                    pt,
                    pt as u32,
                    PatternConfig {
                        max_instances: 8,
                        join_threshold: 4,
                        timeout_ticks: 8,
                        ..Default::default()
                    },
                );

                b.iter(|| {
                    let result = dispatcher.dispatch(black_box(&ctx));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_individual_patterns,
    bench_pattern_categories,
    bench_pattern_dispatch_ticks,
    bench_pattern_throughput,
    bench_worst_case_patterns
);

criterion_main!(benches);
