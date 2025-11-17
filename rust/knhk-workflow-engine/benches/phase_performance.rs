//! Phase Performance Benchmarks
//!
//! Benchmarks for the advanced phase system to ensure optimal performance.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use knhk_workflow_engine::{
    parser::{WorkflowParser, WorkflowSpecId},
    patterns::PatternId,
    state::StateStore,
    task::{Task, TaskId},
    validation::{
        ConformanceMetricsPhase, FormalSoundnessPhase, LoadTestingPhase, PatternSemanticsPhase,
        Phase, PhaseContext, PhaseExecutor,
    },
    WorkflowEngine, WorkflowSpec,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Create a test workflow with N tasks
fn create_test_workflow(num_tasks: usize) -> WorkflowSpec {
    let mut tasks = Vec::new();

    for i in 0..num_tasks {
        let task_id = TaskId::parse_str(&format!("task_{}", i)).unwrap();
        let successor = if i < num_tasks - 1 {
            vec![TaskId::parse_str(&format!("task_{}", i + 1)).unwrap()]
        } else {
            Vec::new()
        };

        tasks.push(Task {
            id: task_id,
            name: format!("Task {}", i),
            description: None,
            pattern: PatternId::parse_str("sequence").unwrap(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            successors: successor,
            guards: Vec::new(),
        });
    }

    WorkflowSpec {
        id: WorkflowSpecId::default(),
        name: format!("benchmark_workflow_{}", num_tasks),
        description: Some("Benchmark workflow".to_string()),
        version: "1.0.0".to_string(),
        tasks,
        metadata: HashMap::new(),
    }
}

/// Benchmark formal soundness validation
fn bench_formal_soundness(c: &mut Criterion) {
    let mut group = c.benchmark_group("formal_soundness");

    for num_tasks in [5, 10, 20, 50].iter() {
        let spec = create_test_workflow(*num_tasks);
        let state_store = StateStore::new_in_memory();
        let engine = Arc::new(WorkflowEngine::new(state_store));
        let spec_id = spec.id;

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            engine.register_spec(spec).await.unwrap();
        });

        group.throughput(Throughput::Elements(*num_tasks as u64));
        group.bench_with_input(BenchmarkId::from_parameter(num_tasks), num_tasks, |b, _| {
            b.to_async(&rt).iter(|| async {
                let ctx = PhaseContext::new(engine.clone(), spec_id);
                let executor = PhaseExecutor::new();
                let phase = FormalSoundnessPhase::new();
                let result = executor.execute_phase(&phase, ctx).await.unwrap();
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark conformance metrics calculation
fn bench_conformance_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("conformance_metrics");

    for num_tasks in [5, 10, 20, 50].iter() {
        let spec = create_test_workflow(*num_tasks);
        let state_store = StateStore::new_in_memory();
        let engine = Arc::new(WorkflowEngine::new(state_store));
        let spec_id = spec.id;

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            engine.register_spec(spec).await.unwrap();
        });

        group.throughput(Throughput::Elements(*num_tasks as u64));
        group.bench_with_input(BenchmarkId::from_parameter(num_tasks), num_tasks, |b, _| {
            b.to_async(&rt).iter(|| async {
                let ctx = PhaseContext::new(engine.clone(), spec_id);
                let executor = PhaseExecutor::new();
                let phase = ConformanceMetricsPhase::new();
                let result = executor.execute_phase(&phase, ctx).await.unwrap();
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark pattern semantics validation
fn bench_pattern_semantics(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_semantics");

    for num_tasks in [5, 10, 20, 50].iter() {
        let spec = create_test_workflow(*num_tasks);
        let state_store = StateStore::new_in_memory();
        let engine = Arc::new(WorkflowEngine::new(state_store));
        let spec_id = spec.id;

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            engine.register_spec(spec).await.unwrap();
        });

        group.throughput(Throughput::Elements(*num_tasks as u64));
        group.bench_with_input(BenchmarkId::from_parameter(num_tasks), num_tasks, |b, _| {
            b.to_async(&rt).iter(|| async {
                let ctx = PhaseContext::new(engine.clone(), spec_id);
                let executor = PhaseExecutor::new();
                let phase = PatternSemanticsPhase::new();
                let result = executor.execute_phase(&phase, ctx).await.unwrap();
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark load testing
fn bench_load_testing(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_testing");
    group.sample_size(10); // Reduced sample size for expensive benchmark

    for num_cases in [10, 25, 50].iter() {
        let spec = create_test_workflow(5);
        let state_store = StateStore::new_in_memory();
        let engine = Arc::new(WorkflowEngine::new(state_store));
        let spec_id = spec.id;

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            engine.register_spec(spec).await.unwrap();
        });

        group.throughput(Throughput::Elements(*num_cases as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_cases),
            num_cases,
            |b, &num_cases| {
                b.to_async(&rt).iter(|| async {
                    let ctx = PhaseContext::new(engine.clone(), spec_id);
                    let executor = PhaseExecutor::new();
                    let phase = LoadTestingPhase::new().with_num_cases(num_cases);
                    let result = executor.execute_phase(&phase, ctx).await.unwrap();
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark parallel phase execution
fn bench_parallel_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_execution");

    let spec = create_test_workflow(10);
    let state_store = StateStore::new_in_memory();
    let engine = Arc::new(WorkflowEngine::new(state_store));
    let spec_id = spec.id;

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        engine.register_spec(spec).await.unwrap();
    });

    group.bench_function("sequential", |b| {
        b.to_async(&rt).iter(|| async {
            let ctx = PhaseContext::new(engine.clone(), spec_id);
            let executor = PhaseExecutor::new().with_parallel(false);

            let phase1 = FormalSoundnessPhase::new();
            let _r1 = executor.execute_phase(&phase1, ctx.clone()).await.unwrap();

            let phase2 = PatternSemanticsPhase::new();
            let _r2 = executor.execute_phase(&phase2, ctx.clone()).await.unwrap();

            let phase3 = ConformanceMetricsPhase::new();
            let _r3 = executor.execute_phase(&phase3, ctx).await.unwrap();
        });
    });

    group.bench_function("parallel", |b| {
        b.to_async(&rt).iter(|| async {
            let ctx = PhaseContext::new(engine.clone(), spec_id);
            let executor = PhaseExecutor::new().with_parallel(true);

            // Note: In real parallel execution, we'd use execute_phases
            // This benchmark just shows the setup overhead
            let phase1 = FormalSoundnessPhase::new();
            let _r1 = executor.execute_phase(&phase1, ctx.clone()).await.unwrap();

            let phase2 = PatternSemanticsPhase::new();
            let _r2 = executor.execute_phase(&phase2, ctx.clone()).await.unwrap();

            let phase3 = ConformanceMetricsPhase::new();
            let _r3 = executor.execute_phase(&phase3, ctx).await.unwrap();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_formal_soundness,
    bench_conformance_metrics,
    bench_pattern_semantics,
    bench_load_testing,
    bench_parallel_execution,
);
criterion_main!(benches);
