// Criterion benchmarks for workflow patterns
// Validates ≤8 tick Chatman Constant compliance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use knhk_patterns::*;
use std::sync::Arc;

// ============================================================================
// Pattern 1: Sequence
// ============================================================================

fn bench_sequence(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_sequence");

    for num_branches in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_branches),
            num_branches,
            |b, &num_branches| {
                let branches: Vec<_> = (0..num_branches)
                    .map(|_| {
                        Arc::new(|mut x: i32| Ok(x + 1))
                            as Arc<dyn Fn(i32) -> PatternResult<i32> + Send + Sync>
                    })
                    .collect();

                let pattern = SequencePattern::new(branches).unwrap();

                b.iter(|| {
                    let result = pattern.execute(black_box(42));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Pattern 2: Parallel Split
// ============================================================================

fn bench_parallel_split(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_parallel_split");

    for num_branches in [2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_branches),
            num_branches,
            |b, &num_branches| {
                let branches: Vec<_> = (0..num_branches)
                    .map(|i| {
                        Arc::new(move |mut x: i32| Ok(x * (i as i32 + 1)))
                            as Arc<dyn Fn(i32) -> PatternResult<i32> + Send + Sync>
                    })
                    .collect();

                let pattern = ParallelSplitPattern::new(branches).unwrap();

                b.iter(|| {
                    let result = pattern.execute(black_box(10));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Pattern 4: Exclusive Choice
// ============================================================================

fn bench_exclusive_choice(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_exclusive_choice");

    for num_choices in [2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_choices),
            num_choices,
            |b, &num_choices| {
                let choices: Vec<_> = (0..num_choices)
                    .map(|i| {
                        let threshold = (i as i32 * 10);
                        (
                            Arc::new(move |x: &i32| *x >= threshold)
                                as Arc<dyn Fn(&i32) -> bool + Send + Sync>,
                            Arc::new(move |x: i32| Ok(x + i as i32))
                                as Arc<dyn Fn(i32) -> PatternResult<i32> + Send + Sync>,
                        )
                    })
                    .collect();

                let pattern = ExclusiveChoicePattern::new(choices).unwrap();

                b.iter(|| {
                    let result = pattern.execute(black_box(15));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Pattern 6: Multi-Choice
// ============================================================================

fn bench_multi_choice(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_multi_choice");

    for num_choices in [2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_choices),
            num_choices,
            |b, &num_choices| {
                let choices: Vec<_> = (0..num_choices)
                    .map(|i| {
                        (
                            Arc::new(move |x: &i32| *x % (i as i32 + 1) == 0)
                                as Arc<dyn Fn(&i32) -> bool + Send + Sync>,
                            Arc::new(move |x: i32| Ok(x * (i as i32 + 1)))
                                as Arc<dyn Fn(i32) -> PatternResult<i32> + Send + Sync>,
                        )
                    })
                    .collect();

                let pattern = MultiChoicePattern::new(choices).unwrap();

                b.iter(|| {
                    let result = pattern.execute(black_box(12));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Pattern 10: Arbitrary Cycles
// ============================================================================

fn bench_arbitrary_cycles(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_arbitrary_cycles");

    for max_iterations in [3, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(max_iterations),
            max_iterations,
            |b, &max_iterations| {
                let branch = Arc::new(|mut x: i32| Ok(x + 1))
                    as Arc<dyn Fn(i32) -> PatternResult<i32> + Send + Sync>;
                let condition =
                    Arc::new(|x: &i32| *x < 50) as Arc<dyn Fn(&i32) -> bool + Send + Sync>;

                let pattern =
                    ArbitraryCyclesPattern::new(branch, condition, max_iterations).unwrap();

                b.iter(|| {
                    let result = pattern.execute(black_box(0));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Composition Benchmarks
// ============================================================================

fn bench_composite_workflow(c: &mut Criterion) {
    c.bench_function("composite_workflow", |b| {
        let workflow = PatternBuilder::new()
            .then(Arc::new(|mut x: i32| Ok(x + 1)))
            .parallel(vec![
                Arc::new(|mut x: i32| Ok(x * 2)),
                Arc::new(|mut x: i32| Ok(x * 3)),
            ])
            .build();

        b.iter(|| {
            let result = workflow.execute(black_box(5));
            black_box(result)
        });
    });
}

criterion_group!(
    benches,
    bench_sequence,
    bench_parallel_split,
    bench_exclusive_choice,
    bench_multi_choice,
    bench_arbitrary_cycles,
    bench_composite_workflow
);

criterion_main!(benches);
