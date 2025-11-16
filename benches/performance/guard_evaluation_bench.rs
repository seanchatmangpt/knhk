///! Guard Evaluation Benchmarks
//! Measures performance of guard evaluation in decision-making

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

/// Guard types
#[derive(Debug, Clone)]
enum GuardType {
    TickBudget { max_ticks: u64 },
    DataSize { max_size: usize },
    QueryComplexity { max_complexity: u32 },
    CacheHitRate { min_rate: f64 },
    Composite { guards: Vec<Box<GuardType>> },
}

/// Guard evaluator
struct GuardEvaluator;

impl GuardEvaluator {
    fn evaluate(guard: &GuardType, context: &EvaluationContext) -> bool {
        match guard {
            GuardType::TickBudget { max_ticks } => context.estimated_ticks <= *max_ticks,
            GuardType::DataSize { max_size } => context.data_size <= *max_size,
            GuardType::QueryComplexity { max_complexity } => {
                context.query_complexity <= *max_complexity
            }
            GuardType::CacheHitRate { min_rate } => context.cache_hit_rate >= *min_rate,
            GuardType::Composite { guards } => {
                guards.iter().all(|g| Self::evaluate(g, context))
            }
        }
    }

    fn evaluate_batch(guards: &[GuardType], context: &EvaluationContext) -> Vec<bool> {
        guards.iter().map(|g| Self::evaluate(g, context)).collect()
    }
}

#[derive(Debug, Clone)]
struct EvaluationContext {
    estimated_ticks: u64,
    data_size: usize,
    query_complexity: u32,
    cache_hit_rate: f64,
}

impl EvaluationContext {
    fn new() -> Self {
        Self {
            estimated_ticks: 5,
            data_size: 100,
            query_complexity: 10,
            cache_hit_rate: 0.8,
        }
    }
}

fn benchmark_tick_budget_guard(c: &mut Criterion) {
    let guard = GuardType::TickBudget { max_ticks: 8 };
    let context = EvaluationContext::new();

    c.bench_function("guard_tick_budget", |b| {
        b.iter(|| {
            black_box(GuardEvaluator::evaluate(&guard, &context))
        });
    });
}

fn benchmark_data_size_guard(c: &mut Criterion) {
    let guard = GuardType::DataSize { max_size: 1000 };
    let context = EvaluationContext::new();

    c.bench_function("guard_data_size", |b| {
        b.iter(|| {
            black_box(GuardEvaluator::evaluate(&guard, &context))
        });
    });
}

fn benchmark_composite_guard(c: &mut Criterion) {
    let guard = GuardType::Composite {
        guards: vec![
            Box::new(GuardType::TickBudget { max_ticks: 8 }),
            Box::new(GuardType::DataSize { max_size: 1000 }),
            Box::new(GuardType::QueryComplexity { max_complexity: 20 }),
        ],
    };
    let context = EvaluationContext::new();

    c.bench_function("guard_composite", |b| {
        b.iter(|| {
            black_box(GuardEvaluator::evaluate(&guard, &context))
        });
    });
}

fn benchmark_batch_evaluation(c: &mut Criterion) {
    let guards = vec![
        GuardType::TickBudget { max_ticks: 8 },
        GuardType::DataSize { max_size: 1000 },
        GuardType::QueryComplexity { max_complexity: 20 },
        GuardType::CacheHitRate { min_rate: 0.7 },
    ];
    let context = EvaluationContext::new();

    c.bench_function("guard_batch_evaluation", |b| {
        b.iter(|| {
            black_box(GuardEvaluator::evaluate_batch(&guards, &context))
        });
    });
}

fn benchmark_guard_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("guard_scaling");

    for num_guards in [1, 5, 10, 20, 50].iter() {
        let guards: Vec<_> = (0..*num_guards)
            .map(|i| GuardType::TickBudget {
                max_ticks: 8 + (i % 5),
            })
            .collect();

        let context = EvaluationContext::new();

        group.bench_with_input(
            BenchmarkId::from_parameter(num_guards),
            num_guards,
            |b, _| {
                b.iter(|| {
                    black_box(GuardEvaluator::evaluate_batch(&guards, &context))
                });
            },
        );
    }

    group.finish();
}

fn benchmark_nested_composite_guards(c: &mut Criterion) {
    let guard = GuardType::Composite {
        guards: vec![
            Box::new(GuardType::Composite {
                guards: vec![
                    Box::new(GuardType::TickBudget { max_ticks: 8 }),
                    Box::new(GuardType::DataSize { max_size: 1000 }),
                ],
            }),
            Box::new(GuardType::Composite {
                guards: vec![
                    Box::new(GuardType::QueryComplexity { max_complexity: 20 }),
                    Box::new(GuardType::CacheHitRate { min_rate: 0.7 }),
                ],
            }),
        ],
    };
    let context = EvaluationContext::new();

    c.bench_function("guard_nested_composite", |b| {
        b.iter(|| {
            black_box(GuardEvaluator::evaluate(&guard, &context))
        });
    });
}

criterion_group! {
    name = guard_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(1000);
    targets = benchmark_tick_budget_guard,
              benchmark_data_size_guard,
              benchmark_composite_guard,
              benchmark_batch_evaluation,
              benchmark_guard_scaling,
              benchmark_nested_composite_guards
}

criterion_main!(guard_benches);
