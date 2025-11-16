// knhk-kernel: Hot path performance benchmarks
// Measures all operations in CPU ticks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use knhk_kernel::prelude::*;
use knhk_kernel::{pattern::PatternType, descriptor::PatternEntry};

fn setup_kernel() {
    knhk_kernel::init().expect("Kernel init failed");

    // Setup descriptor with all pattern types
    let mut builder = DescriptorBuilder::new().with_tick_budget(8);

    for i in 1..=43u8 {
        let pattern_type = unsafe { std::mem::transmute::<u8, PatternType>(i) };
        let pattern = PatternEntry::new(
            pattern_type,
            i as u32,
            10,
            knhk_kernel::pattern::PatternConfig::default(),
        );
        builder = builder.add_pattern(pattern);
    }

    let desc = Box::new(builder.build());
    DescriptorManager::load_descriptor(desc).expect("Descriptor load failed");
}

fn bench_hot_path_execution(c: &mut Criterion) {
    setup_kernel();

    let mut group = c.benchmark_group("hot_path_execution");
    group.significance_level(0.01);

    // Benchmark different task complexities
    for observation_count in [1, 2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::new("observations", observation_count),
            &observation_count,
            |b, &count| {
                let executor = Executor::new();

                b.iter(|| {
                    let mut task = Task::new(1, 1);
                    for i in 0..count {
                        task.add_observation(i as u64);
                    }
                    task.transition(TaskState::Ready);

                    let receipt = executor.execute(black_box(&task));
                    black_box(receipt)
                });
            },
        );
    }

    group.finish();
}

fn bench_pattern_dispatch(c: &mut Criterion) {
    setup_kernel();

    let mut group = c.benchmark_group("pattern_dispatch");

    let dispatcher = knhk_kernel::pattern::PatternDispatcher::new();

    // Benchmark each pattern type
    for i in [1, 2, 3, 4, 5, 9, 15, 26, 43] {
        let pattern_type = unsafe { std::mem::transmute::<u8, PatternType>(i) };

        group.bench_with_input(
            BenchmarkId::new("pattern", format!("{:?}", pattern_type)),
            &pattern_type,
            |b, &pt| {
                let ctx = knhk_kernel::pattern::PatternFactory::create(
                    pt,
                    i as u32,
                    knhk_kernel::pattern::PatternConfig::default()
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

fn bench_guard_evaluation(c: &mut Criterion) {
    use knhk_kernel::guard::{Guard, Predicate, ResourceType};
    use knhk_kernel::descriptor::{ExecutionContext, ResourceState, ObservationBuffer};

    let mut group = c.benchmark_group("guard_evaluation");

    let context = ExecutionContext {
        task_id: 42,
        timestamp: 1000,
        resources: ResourceState {
            cpu_available: 80,
            memory_available: 1024,
            io_capacity: 100,
            queue_depth: 10,
        },
        observations: ObservationBuffer {
            count: 5,
            observations: [1, 2, 3, 4, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        },
        state_flags: 0xFF,
    };

    // Simple predicate guard
    group.bench_function("predicate_guard", |b| {
        let guard = Guard::predicate(Predicate::Equal, 0, 42);

        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    // Resource guard
    group.bench_function("resource_guard", |b| {
        let guard = Guard::resource(ResourceType::Cpu, 50);

        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    // Compound AND guard
    group.bench_function("compound_and_guard", |b| {
        let g1 = Guard::predicate(Predicate::Equal, 0, 42);
        let g2 = Guard::resource(ResourceType::Cpu, 50);
        let g3 = Guard::resource(ResourceType::Memory, 500);
        let guard = Guard::and(vec![g1, g2, g3]);

        b.iter(|| {
            let result = guard.evaluate(black_box(&context));
            black_box(result)
        });
    });

    group.finish();
}

fn bench_receipt_generation(c: &mut Criterion) {
    use knhk_kernel::receipt::{ReceiptBuilder, ReceiptStatus};

    let mut group = c.benchmark_group("receipt_generation");

    group.bench_function("receipt_build", |b| {
        b.iter(|| {
            let receipt = ReceiptBuilder::new(1, 100)
                .with_budget(8)
                .with_inputs(&[1, 2, 3, 4, 5])
                .with_outputs(&[10, 20, 30])
                .with_result(ReceiptStatus::Success, 5)
                .add_guard(1, true, 1)
                .add_guard(2, false, 2)
                .build();

            black_box(receipt)
        });
    });

    group.bench_function("receipt_verify", |b| {
        let receipt = ReceiptBuilder::new(1, 100)
            .with_inputs(&[1, 2, 3, 4, 5])
            .with_outputs(&[10, 20, 30])
            .with_result(ReceiptStatus::Success, 5)
            .build();

        b.iter(|| {
            let valid = receipt.verify();
            black_box(valid)
        });
    });

    group.finish();
}

fn bench_tick_measurement(c: &mut Criterion) {
    let mut group = c.benchmark_group("tick_measurement");

    // Benchmark RDTSC overhead
    group.bench_function("read_tsc", |b| {
        b.iter(|| {
            let ticks = read_tsc();
            black_box(ticks)
        });
    });

    // Benchmark timer creation and measurement
    group.bench_function("timer_elapsed", |b| {
        b.iter(|| {
            let timer = HotPathTimer::start();
            // Minimal work
            let x = black_box(42);
            let y = black_box(x + 1);
            let elapsed = timer.elapsed_ticks();
            black_box((y, elapsed))
        });
    });

    // Benchmark tick budget tracking
    group.bench_function("tick_budget", |b| {
        b.iter(|| {
            let mut budget = TickBudget::new();
            budget.charge("op1", 2).unwrap();
            budget.charge("op2", 3).unwrap();
            let remaining = budget.remaining();
            black_box(remaining)
        });
    });

    group.finish();
}

fn bench_state_transitions(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_transitions");

    group.bench_function("atomic_transition", |b| {
        let task = Task::new(1, 1);

        b.iter(|| {
            let old = task.transition(TaskState::Ready);
            task.transition(TaskState::Running);
            task.transition(TaskState::Completed);
            task.transition(TaskState::Created);
            black_box(old)
        });
    });

    group.finish();
}

fn bench_stratum_routing(c: &mut Criterion) {
    setup_kernel();

    let mut group = c.benchmark_group("stratum_routing");

    let hot_path = HotPath::new();

    group.bench_function("submit_hot", |b| {
        b.iter(|| {
            let mut task = Box::new(Task::new(1, 1));
            task.observation_count = 2; // Small = hot stratum
            task.transition(TaskState::Ready);

            let result = hot_path.submit(black_box(task));
            black_box(result)
        });
    });

    group.bench_function("submit_warm", |b| {
        b.iter(|| {
            let mut task = Box::new(Task::new(1, 1));
            task.observation_count = 8; // Medium = warm stratum
            task.transition(TaskState::Ready);

            let result = hot_path.submit(black_box(task));
            black_box(result)
        });
    });

    group.finish();
}

fn bench_descriptor_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("descriptor_operations");

    // Build a descriptor with multiple patterns
    let mut builder = DescriptorBuilder::new().with_tick_budget(8);
    for i in 1..=10 {
        let pattern = PatternEntry::new(
            PatternType::Sequence,
            i,
            i * 10,
            knhk_kernel::pattern::PatternConfig::default(),
        );
        builder = builder.add_pattern(pattern);
    }
    let desc = builder.build();

    group.bench_function("pattern_lookup", |b| {
        b.iter(|| {
            let pattern = desc.get_pattern(black_box(5));
            black_box(pattern)
        });
    });

    group.bench_function("descriptor_validate", |b| {
        b.iter(|| {
            let result = desc.validate();
            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_hot_path_execution,
    bench_pattern_dispatch,
    bench_guard_evaluation,
    bench_receipt_generation,
    bench_tick_measurement,
    bench_state_transitions,
    bench_stratum_routing,
    bench_descriptor_operations
);

criterion_main!(benches);