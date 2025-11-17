// knhk-kernel: Guard evaluation performance benchmarks
// Measures boolean gate evaluation in CPU ticks

use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use knhk_kernel::{
    descriptor::{ExecutionContext, ObservationBuffer, ResourceState},
    guard::{Guard, GuardType, Predicate, ResourceType, StateFlags},
    timer::read_tsc,
};

fn create_test_context() -> ExecutionContext {
    ExecutionContext {
        task_id: 42,
        timestamp: 1000000,
        resources: ResourceState {
            cpu_available: 80,
            memory_available: 1024,
            io_capacity: 100,
            queue_depth: 10,
        },
        observations: ObservationBuffer {
            count: 8,
            observations: [1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0],
        },
        state_flags: (StateFlags::INITIALIZED | StateFlags::RUNNING).bits(),
    }
}

fn bench_simple_guards(c: &mut Criterion) {
    let mut group = c.benchmark_group("guard_simple");

    let context = create_test_context();

    // Predicate guards
    group.bench_function("predicate_equal", |b| {
        let guard = Guard::predicate(Predicate::Equal, 0, 42);
        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    group.bench_function("predicate_greater", |b| {
        let guard = Guard::predicate(Predicate::GreaterThan, 1, 500000);
        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    group.bench_function("predicate_range", |b| {
        let guard = Guard::predicate(Predicate::InRange, 900000, 1100000);
        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    // Resource guards
    group.bench_function("resource_cpu", |b| {
        let guard = Guard::resource(ResourceType::Cpu, 50);
        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    group.bench_function("resource_memory", |b| {
        let guard = Guard::resource(ResourceType::Memory, 512);
        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    // State guards
    group.bench_function("state_flags", |b| {
        let guard = Guard::state(StateFlags::INITIALIZED | StateFlags::RUNNING);
        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    group.finish();
}

fn bench_compound_guards(c: &mut Criterion) {
    let mut group = c.benchmark_group("guard_compound");

    let context = create_test_context();

    // AND guards (short-circuit on first false)
    group.bench_function("and_2_guards", |b| {
        let g1 = Guard::predicate(Predicate::Equal, 0, 42);
        let g2 = Guard::resource(ResourceType::Cpu, 50);
        let guard = Guard::and(vec![g1, g2]);

        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    group.bench_function("and_4_guards", |b| {
        let g1 = Guard::predicate(Predicate::Equal, 0, 42);
        let g2 = Guard::resource(ResourceType::Cpu, 50);
        let g3 = Guard::resource(ResourceType::Memory, 512);
        let g4 = Guard::state(StateFlags::RUNNING);
        let guard = Guard::and(vec![g1, g2, g3, g4]);

        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    // OR guards (short-circuit on first true)
    group.bench_function("or_2_guards", |b| {
        let g1 = Guard::predicate(Predicate::Equal, 0, 99); // False
        let g2 = Guard::resource(ResourceType::Cpu, 50); // True
        let guard = Guard::or(vec![g1, g2]);

        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    // NOT guards
    group.bench_function("not_guard", |b| {
        let g1 = Guard::predicate(Predicate::Equal, 0, 99); // False
        let guard = Guard::not(g1);

        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    // Nested compound guards
    group.bench_function("nested_compound", |b| {
        let g1 = Guard::predicate(Predicate::Equal, 0, 42);
        let g2 = Guard::resource(ResourceType::Cpu, 50);
        let and1 = Guard::and(vec![g1, g2]);

        let g3 = Guard::resource(ResourceType::Memory, 2048); // False
        let g4 = Guard::state(StateFlags::COMPLETED); // False
        let and2 = Guard::and(vec![g3, g4]);

        let guard = Guard::or(vec![and1, and2]); // First AND is true

        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    group.finish();
}

fn bench_guard_ticks(c: &mut Criterion) {
    let mut group = c.benchmark_group("guard_ticks");
    group.warm_up_time(std::time::Duration::from_millis(500));

    let context = create_test_context();

    // Measure in CPU ticks
    let guards = vec![
        (
            "simple_predicate",
            Guard::predicate(Predicate::Equal, 0, 42),
        ),
        ("resource_check", Guard::resource(ResourceType::Cpu, 50)),
        ("state_check", Guard::state(StateFlags::RUNNING)),
        (
            "compound_and",
            Guard::and(vec![
                Guard::predicate(Predicate::Equal, 0, 42),
                Guard::resource(ResourceType::Cpu, 50),
            ]),
        ),
    ];

    for (name, guard) in guards {
        group.bench_with_input(BenchmarkId::new("ticks", name), &guard, |b, guard| {
            b.iter_custom(|iters| {
                let mut total_ticks = 0u64;

                for _ in 0..iters {
                    let start = read_tsc();
                    let _result = guard.evaluate(&context);
                    let end = read_tsc();
                    total_ticks += end - start;
                }

                std::time::Duration::from_nanos(total_ticks / iters)
            });
        });
    }

    group.finish();
}

fn bench_guard_evaluator(c: &mut Criterion) {
    use knhk_kernel::guard::GuardEvaluator;

    let mut group = c.benchmark_group("guard_evaluator");

    let context = create_test_context();
    let guard = Guard::and(vec![
        Guard::predicate(Predicate::Equal, 0, 42),
        Guard::resource(ResourceType::Cpu, 50),
        Guard::state(StateFlags::RUNNING),
    ]);

    // Without caching
    group.bench_function("without_cache", |b| {
        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    // With caching
    group.bench_function("with_cache", |b| {
        let mut evaluator = GuardEvaluator::new(100);

        b.iter(|| {
            let result = evaluator.evaluate_cached(1, &guard, black_box(&context));
            black_box(result)
        });
    });

    group.finish();
}

fn bench_worst_case_guards(c: &mut Criterion) {
    let mut group = c.benchmark_group("guard_worst_case");

    let context = create_test_context();

    // Deep nesting
    group.bench_function("deep_nesting", |b| {
        let mut guard = Guard::predicate(Predicate::Equal, 0, 42);

        for _ in 0..5 {
            guard = Guard::and(vec![guard.clone(), Guard::resource(ResourceType::Cpu, 50)]);
        }

        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    // Many guards (no short circuit)
    group.bench_function("many_guards_and", |b| {
        let guards: Vec<Guard> = (0..8)
            .map(|i| Guard::predicate(Predicate::GreaterThan, 0, i * 10))
            .collect();

        let guard = Guard::and(guards);

        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    group.finish();
}

fn bench_guard_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("guard_patterns");

    let context = create_test_context();

    // Common patterns
    group.bench_function("auth_check", |b| {
        // Typical authentication guard pattern
        let guard = Guard::and(vec![
            Guard::state(StateFlags::INITIALIZED),
            Guard::predicate(Predicate::NotEqual, 0, 0), // User ID != 0
            Guard::resource(ResourceType::Cpu, 10),      // Has resources
        ]);

        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    group.bench_function("resource_limits", |b| {
        // Resource limit checking pattern
        let guard = Guard::and(vec![
            Guard::resource(ResourceType::Cpu, 20),
            Guard::resource(ResourceType::Memory, 256),
            Guard::resource(ResourceType::Io, 50),
            Guard::resource(ResourceType::Queue, 5),
        ]);

        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    group.bench_function("fallback_chain", |b| {
        // Try primary, fallback to secondary
        let primary = Guard::and(vec![
            Guard::resource(ResourceType::Cpu, 90), // Needs high CPU (will fail)
            Guard::resource(ResourceType::Memory, 2048), // Needs high memory (will fail)
        ]);

        let secondary = Guard::and(vec![
            Guard::resource(ResourceType::Cpu, 50), // Lower requirements
            Guard::resource(ResourceType::Memory, 512),
        ]);

        let guard = Guard::or(vec![primary, secondary]);

        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_guards,
    bench_compound_guards,
    bench_guard_ticks,
    bench_guard_evaluator,
    bench_worst_case_guards,
    bench_guard_patterns
);

criterion_main!(benches);
