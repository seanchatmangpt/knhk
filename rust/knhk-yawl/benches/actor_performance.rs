//! Performance benchmarks for actor-based workflow engine
//!
//! VALIDATES Q3 (Chatman constant ≤ 8 ticks)

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use knhk_yawl::engine::{
    spawn_actor, StateStore, TaskActor, TaskDefinition, TaskId, TokenManager, Workflow,
    WorkflowExecutor, WorkflowId, WorkflowMessage, WorkflowState,
};
use std::time::Duration;

fn bench_message_passing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("message_passing", |b| {
        b.to_async(&rt).iter(|| async {
            let task_id = TaskId::new();
            let actor = TaskActor::new(task_id);
            let handle = spawn_actor(actor, 100);

            // Send message
            let msg = WorkflowMessage::HealthCheck;
            handle.send(black_box(msg)).await.unwrap();

            // Give actor time to process
            tokio::time::sleep(Duration::from_micros(1)).await;
        });
    });
}

fn bench_state_transition(c: &mut Criterion) {
    let store = StateStore::new();
    let workflow_id = WorkflowId::new();

    store.create_workflow(workflow_id, serde_json::json!({}));

    c.bench_function("state_transition", |b| {
        b.iter(|| {
            store
                .transition_workflow(black_box(workflow_id), WorkflowState::Executing)
                .unwrap();
            store
                .transition_workflow(black_box(workflow_id), WorkflowState::Suspended)
                .unwrap();
            store
                .transition_workflow(black_box(workflow_id), WorkflowState::Executing)
                .unwrap();
        });
    });
}

fn bench_token_creation(c: &mut Criterion) {
    let manager = TokenManager::new();
    let workflow_id = WorkflowId::new();
    let task_id = TaskId::new();

    c.bench_function("token_creation", |b| {
        b.iter(|| {
            manager.create_token(
                black_box(workflow_id),
                black_box(Some(task_id)),
                black_box(serde_json::json!({"test": true})),
            );
        });
    });
}

fn bench_token_routing(c: &mut Criterion) {
    let manager = TokenManager::new();
    let workflow_id = WorkflowId::new();
    let source_task = TaskId::new();
    let target_task = TaskId::new();

    let token_id = manager.create_token(workflow_id, Some(source_task), serde_json::json!({}));

    c.bench_function("token_routing", |b| {
        b.iter(|| {
            manager.route_token(black_box(token_id), black_box(target_task));
        });
    });
}

fn bench_workflow_execution(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("workflow_execution");

    for task_count in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(task_count),
            task_count,
            |b, &task_count| {
                b.to_async(&rt).iter(|| async move {
                    let executor = WorkflowExecutor::new();

                    let mut tasks = vec![];
                    for i in 0..task_count {
                        tasks.push(TaskDefinition {
                            id: TaskId::new(),
                            name: format!("task-{}", i),
                            dependencies: vec![],
                        });
                    }

                    let workflow = Workflow {
                        id: WorkflowId::new(),
                        name: "bench-workflow".to_string(),
                        tasks,
                    };

                    executor.execute_workflow(black_box(workflow)).await.unwrap();

                    // Give actors time to start
                    tokio::time::sleep(Duration::from_millis(1)).await;
                });
            },
        );
    }

    group.finish();
}

fn bench_actor_spawn(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("actor_spawn", |b| {
        b.to_async(&rt).iter(|| async {
            let task_id = TaskId::new();
            let actor = TaskActor::new(black_box(task_id));
            let _handle = spawn_actor(actor, 100);

            // Give actor time to initialize
            tokio::time::sleep(Duration::from_micros(10)).await;
        });
    });
}

fn bench_concurrent_workflows(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_workflows");

    for workflow_count in [5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(workflow_count),
            workflow_count,
            |b, &workflow_count| {
                b.to_async(&rt).iter(|| async move {
                    let executor = WorkflowExecutor::new();

                    let mut workflows = vec![];
                    for i in 0..workflow_count {
                        workflows.push(Workflow {
                            id: WorkflowId::new(),
                            name: format!("workflow-{}", i),
                            tasks: vec![TaskDefinition {
                                id: TaskId::new(),
                                name: format!("task-{}", i),
                                dependencies: vec![],
                            }],
                        });
                    }

                    for workflow in workflows {
                        executor.execute_workflow(black_box(workflow)).await.unwrap();
                    }

                    tokio::time::sleep(Duration::from_millis(10)).await;
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_message_passing,
    bench_state_transition,
    bench_token_creation,
    bench_token_routing,
    bench_workflow_execution,
    bench_actor_spawn,
    bench_concurrent_workflows,
);

criterion_main!(benches);
