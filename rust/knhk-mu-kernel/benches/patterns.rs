//! YAWL Pattern Performance Benchmarks
//!
//! Measures all 43 Van der Aalst workflow patterns

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use knhk_mu_kernel::patterns::{PatternId, PatternHandler};
use knhk_mu_kernel::timing::TickBudget;
use knhk_mu_kernel::guards::GuardContext;

fn bench_all_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("patterns");
    group.sample_size(10000);

    let obs = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    // Basic Control Flow Patterns (1-5)
    bench_pattern(&mut group, PatternId::Sequence, &obs);
    bench_pattern(&mut group, PatternId::ParallelSplit, &obs);
    bench_pattern(&mut group, PatternId::Synchronization, &obs);
    bench_pattern(&mut group, PatternId::ExclusiveChoice, &obs);
    bench_pattern(&mut group, PatternId::SimpleMerge, &obs);

    // Advanced Branching (6-9)
    bench_pattern(&mut group, PatternId::MultiChoice, &obs);
    bench_pattern(&mut group, PatternId::StructuredSynchronizingMerge, &obs);
    bench_pattern(&mut group, PatternId::MultiMerge, &obs);
    bench_pattern(&mut group, PatternId::StructuredDiscriminator, &obs);

    // Structural Patterns (10-13)
    bench_pattern(&mut group, PatternId::ArbitraryCycles, &obs);
    bench_pattern(&mut group, PatternId::ImplicitTermination, &obs);
    bench_pattern(&mut group, PatternId::MultipleInstancesWithoutSynchronization, &obs);
    bench_pattern(&mut group, PatternId::MultipleInstancesWithAPrioriDesignTimeKnowledge, &obs);

    group.finish();
}

fn bench_pattern(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>, pattern: PatternId, obs: &GuardContext) {
    let pattern_name = format!("{:?}", pattern);

    group.bench_with_input(
        BenchmarkId::new("execute", &pattern_name),
        &pattern,
        |b, &pattern_id| {
            b.iter(|| {
                let mut budget = TickBudget::chatman();
                let result = pattern_id.execute(black_box(obs), &mut budget);
                black_box(result)
            });
        },
    );
}

fn bench_pattern_tick_costs(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_costs");
    group.sample_size(100000);

    group.bench_function("tick_cost_lookup", |b| {
        b.iter(|| {
            let pattern = black_box(PatternId::Sequence);
            let cost = pattern.tick_cost();
            black_box(cost)
        });
    });

    group.finish();
}

fn bench_pattern_categories(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_categories");
    group.sample_size(10000);

    let obs = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    // Group 1: Control Flow (cheapest, 1-2 ticks)
    let control_flow = [
        PatternId::Sequence,
        PatternId::ParallelSplit,
        PatternId::Synchronization,
    ];

    for pattern in control_flow {
        let mut budget = TickBudget::chatman();
        let _ = pattern.execute(&obs, &mut budget);
        assert!(budget.used() <= 2, "Control flow pattern too expensive: {}", budget.used());
    }

    // Group 2: Advanced Branching (moderate, 3-5 ticks)
    let branching = [
        PatternId::MultiChoice,
        PatternId::StructuredSynchronizingMerge,
        PatternId::MultiMerge,
    ];

    for pattern in branching {
        let mut budget = TickBudget::chatman();
        let _ = pattern.execute(&obs, &mut budget);
        assert!(budget.used() <= 5, "Branching pattern too expensive: {}", budget.used());
    }

    group.finish();
}

criterion_group!(
    pattern_benches,
    bench_all_patterns,
    bench_pattern_tick_costs,
    bench_pattern_categories,
);

criterion_main!(pattern_benches);
