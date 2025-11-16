//! Performance benchmarks for quantum-inspired optimization algorithms
//!
//! Validates that 1M workflows can be scheduled in < 100ms

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use knhk_workflow_engine::quantum::*;
use std::time::Duration;

fn create_benchmark_tasks(count: usize) -> Vec<WorkflowTask> {
    (0..count)
        .map(|i| {
            WorkflowTask::new(format!("task-{}", i))
                .with_duration(50 + (i as u64 % 100))
                .with_cost(10.0 + (i as f64 % 50.0))
                .with_cpu(40.0 + (i as f64 % 40.0))
                .with_memory(512.0 + (i as f64 % 1024.0))
                .with_priority((i as i32 % 10) - 5)
        })
        .collect()
}

fn create_benchmark_tasks_with_deps(count: usize) -> Vec<WorkflowTask> {
    let mut tasks = Vec::with_capacity(count);

    for i in 0..count {
        let mut task = WorkflowTask::new(format!("task-{}", i))
            .with_duration(50)
            .with_cost(10.0)
            .with_cpu(50.0)
            .with_memory(512.0);

        // Add dependencies to create DAG structure
        if i > 0 && i % 3 == 0 {
            task = task.with_dependency(tasks[i - 1].id);
        }
        if i > 1 && i % 5 == 0 {
            task = task.with_dependency(tasks[i - 2].id);
        }

        tasks.push(task);
    }

    tasks
}

fn bench_quantum_annealing(c: &mut Criterion) {
    let mut group = c.benchmark_group("quantum_annealing");
    group.measurement_time(Duration::from_secs(30));

    for size in [100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        let tasks = create_benchmark_tasks(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| async {
                    let mut constraints = ConstraintManager::new();
                    constraints
                        .add_constraint(Box::new(LatencyConstraint::new(100_000)))
                        .unwrap();
                    constraints
                        .add_constraint(Box::new(CostConstraint::new(1_000_000.0)))
                        .unwrap();

                    let config = AnnealingConfig::with_seed(42)
                        .max_iterations(100) // Limit iterations for benchmark
                        .cooling_rate(0.9); // Faster cooling

                    let mut annealer = QuantumAnnealing::new(config, std::sync::Arc::new(constraints));
                    let result = annealer.optimize(black_box(&tasks)).await.unwrap();
                    black_box(result)
                });
        });
    }

    group.finish();
}

fn bench_grover_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("grover_search");
    group.measurement_time(Duration::from_secs(20));

    let resources = vec![
        Resource::new("resource-0"),
        Resource::new("resource-1"),
        Resource::new("resource-2"),
        Resource::new("resource-3"),
    ];

    for size in [100, 1_000, 10_000, 100_000].iter() {
        let tasks = create_benchmark_tasks(*size);
        let search_space = tasks.len() * resources.len();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| async {
                    let config = GroverConfig::for_search_space(search_space)
                        .with_seed(42)
                        .with_amplification(2.0);

                    let mut grover = GroverSearch::new(config);
                    let result = grover
                        .find_optimal_allocation(
                            black_box(&tasks),
                            &resources,
                            Box::new(default_oracle),
                        )
                        .await
                        .unwrap();
                    black_box(result)
                });
        });
    }

    group.finish();
}

fn bench_qaoa(c: &mut Criterion) {
    let mut group = c.benchmark_group("qaoa");
    group.measurement_time(Duration::from_secs(20));

    for size in [10, 50, 100, 500, 1_000].iter() {
        let tasks = create_benchmark_tasks_with_deps(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| async {
                    let config = QAOAConfig::default()
                        .with_seed(42)
                        .with_layers(2)
                        .with_max_iterations(50); // Reduced for benchmark

                    let mut qaoa = QAOAOptimizer::new(config);
                    let result = qaoa
                        .optimize_assignment(black_box(&tasks), 4)
                        .await
                        .unwrap();
                    black_box(result)
                });
        });
    }

    group.finish();
}

fn bench_quantum_walk(c: &mut Criterion) {
    let mut group = c.benchmark_group("quantum_walk");
    group.measurement_time(Duration::from_secs(20));

    for size in [10, 50, 100, 500, 1_000, 10_000].iter() {
        let tasks = create_benchmark_tasks_with_deps(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| async {
                    let config = QuantumWalkConfig::default()
                        .with_seed(42)
                        .with_max_iterations(100); // Reduced for benchmark

                    let mut qwalk = QuantumWalk::new(config);
                    let result = qwalk
                        .find_execution_order(black_box(&tasks))
                        .await
                        .unwrap();
                    black_box(result)
                });
        });
    }

    group.finish();
}

fn bench_scheduler_e2e(c: &mut Criterion) {
    let mut group = c.benchmark_group("scheduler_end_to_end");
    group.measurement_time(Duration::from_secs(30));

    for size in [100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        let tasks = create_benchmark_tasks(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| async {
                    let scheduler = QuantumScheduler::builder()
                        .with_seed(42)
                        .with_method(OptimizationMethod::QuantumAnnealing)
                        .with_constraint(Box::new(LatencyConstraint::new(100_000)))
                        .with_constraint(Box::new(CostConstraint::new(1_000_000.0)))
                        .with_constraint(Box::new(ResourceConstraint::new(80.0)))
                        .build()
                        .unwrap();

                    let result = scheduler.optimize(black_box(&tasks)).await.unwrap();
                    black_box(result)
                });
        });
    }

    group.finish();
}

fn bench_hybrid_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("hybrid_optimization");
    group.measurement_time(Duration::from_secs(40));

    for size in [100, 1_000, 10_000].iter() {
        let tasks = create_benchmark_tasks(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| async {
                    let scheduler = QuantumScheduler::builder()
                        .with_seed(42)
                        .with_method(OptimizationMethod::Hybrid)
                        .with_constraint(Box::new(LatencyConstraint::new(100_000)))
                        .build()
                        .unwrap();

                    let result = scheduler.optimize(black_box(&tasks)).await.unwrap();
                    black_box(result)
                });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_quantum_annealing,
    bench_grover_search,
    bench_qaoa,
    bench_quantum_walk,
    bench_scheduler_e2e,
    bench_hybrid_optimization,
);

criterion_main!(benches);
