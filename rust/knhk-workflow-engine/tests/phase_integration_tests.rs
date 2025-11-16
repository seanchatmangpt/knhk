//! Integration Tests for Advanced Phase System

use knhk_workflow_engine::{
    parser::{WorkflowParser, WorkflowSpecId},
    patterns::PatternId,
    state::StateStore,
    task::{Task, TaskId},
    validation::{
        ConformanceMetricsPhase, FormalSoundnessPhase, LoadTestingPhase, PatternSemanticsPhase,
        Phase, PhaseContext, PhaseExecutor, PhaseStatus,
    },
    WorkflowEngine, WorkflowSpec,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Helper: Create a simple sequential workflow
fn create_simple_workflow() -> WorkflowSpec {
    WorkflowSpec {
        id: WorkflowSpecId::default(),
        name: "simple_workflow".to_string(),
        description: Some("Simple sequential workflow".to_string()),
        version: "1.0.0".to_string(),
        tasks: vec![
            Task {
                id: TaskId::parse_str("task1").unwrap(),
                name: "Task 1".to_string(),
                description: None,
                pattern: PatternId::parse_str("sequence").unwrap(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                successors: vec![TaskId::parse_str("task2").unwrap()],
                guards: Vec::new(),
            },
            Task {
                id: TaskId::parse_str("task2").unwrap(),
                name: "Task 2".to_string(),
                description: None,
                pattern: PatternId::parse_str("sequence").unwrap(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                successors: Vec::new(),
                guards: Vec::new(),
            },
        ],
        metadata: HashMap::new(),
    }
}

#[tokio::test]
async fn test_formal_soundness_phase() {
    let spec = create_simple_workflow();
    let state_store = StateStore::new_in_memory();
    let engine = Arc::new(WorkflowEngine::new(state_store));
    let spec_id = spec.id;

    engine.register_spec(spec).await.unwrap();

    let ctx = PhaseContext::new(engine, spec_id);
    let executor = PhaseExecutor::new();
    let phase = FormalSoundnessPhase::new();

    let result = executor.execute_phase(&phase, ctx).await.unwrap();

    assert_eq!(result.status, PhaseStatus::Pass);
    assert!(result.data.option_to_complete);
    assert!(result.data.proper_completion);
    assert!(result.data.no_dead_tasks);
    assert_eq!(result.data.dead_tasks.len(), 0);
}

#[tokio::test]
async fn test_conformance_metrics_phase() {
    let spec = create_simple_workflow();
    let state_store = StateStore::new_in_memory();
    let engine = Arc::new(WorkflowEngine::new(state_store));
    let spec_id = spec.id;

    engine.register_spec(spec).await.unwrap();

    let ctx = PhaseContext::new(engine, spec_id);
    let executor = PhaseExecutor::new();
    let phase = ConformanceMetricsPhase::new();

    let result = executor.execute_phase(&phase, ctx).await.unwrap();

    // Conformance metrics should be calculated (not hardcoded)
    assert!(result.data.fitness >= 0.0 && result.data.fitness <= 1.0);
    assert!(result.data.precision >= 0.0 && result.data.precision <= 1.0);
    assert!(result.data.f_measure >= 0.0 && result.data.f_measure <= 1.0);
    assert!(result.data.traces_analyzed > 0);
}

#[tokio::test]
async fn test_pattern_semantics_phase() {
    let spec = create_simple_workflow();
    let state_store = StateStore::new_in_memory();
    let engine = Arc::new(WorkflowEngine::new(state_store));
    let spec_id = spec.id;

    engine.register_spec(spec).await.unwrap();

    let ctx = PhaseContext::new(engine, spec_id);
    let executor = PhaseExecutor::new();
    let phase = PatternSemanticsPhase::new();

    let result = executor.execute_phase(&phase, ctx).await.unwrap();

    assert_eq!(result.status, PhaseStatus::Pass);
    assert!(result.data.total_patterns > 0);
    assert_eq!(result.data.invalid_patterns, 0);
}

#[tokio::test]
async fn test_load_testing_phase() {
    let spec = create_simple_workflow();
    let state_store = StateStore::new_in_memory();
    let engine = Arc::new(WorkflowEngine::new(state_store));
    let spec_id = spec.id;

    engine.register_spec(spec).await.unwrap();

    let ctx = PhaseContext::new(engine, spec_id);
    let executor = PhaseExecutor::new();
    let phase = LoadTestingPhase::new().with_num_cases(10); // Small load for test

    let result = executor.execute_phase(&phase, ctx).await.unwrap();

    assert_eq!(result.data.cases_created, 10);
    assert!(result.data.successful_cases > 0);
    assert!(result.data.avg_latency_ms >= 0.0);
    assert!(result.data.throughput >= 0.0);
}

#[tokio::test]
async fn test_parallel_phase_execution() {
    let spec = create_simple_workflow();
    let state_store = StateStore::new_in_memory();
    let engine = Arc::new(WorkflowEngine::new(state_store));
    let spec_id = spec.id;

    engine.register_spec(spec).await.unwrap();

    let ctx = PhaseContext::new(engine, spec_id);
    let executor = PhaseExecutor::new().with_parallel(true);

    // Execute multiple phases
    let phase1 = FormalSoundnessPhase::new();
    let result1 = executor.execute_phase(&phase1, ctx.clone()).await.unwrap();

    let phase2 = PatternSemanticsPhase::new();
    let result2 = executor.execute_phase(&phase2, ctx.clone()).await.unwrap();

    assert_eq!(result1.status, PhaseStatus::Pass);
    assert_eq!(result2.status, PhaseStatus::Pass);
}

#[tokio::test]
async fn test_phase_telemetry() {
    let spec = create_simple_workflow();
    let state_store = StateStore::new_in_memory();
    let engine = Arc::new(WorkflowEngine::new(state_store));
    let spec_id = spec.id;

    engine.register_spec(spec).await.unwrap();

    let ctx = PhaseContext::new(engine, spec_id);
    let executor = PhaseExecutor::new();
    let phase = FormalSoundnessPhase::new();

    let result = executor.execute_phase(&phase, ctx).await.unwrap();

    // Verify metrics are collected
    assert!(!result.metrics.is_empty());
    assert!(result.metrics.contains_key("reachable_tasks"));
    assert!(result.metrics.contains_key("dead_tasks"));
    assert!(result.duration.as_millis() > 0);
}

#[tokio::test]
async fn test_phase_with_invalid_workflow() {
    // Create a workflow with dead tasks
    let spec = WorkflowSpec {
        id: WorkflowSpecId::default(),
        name: "invalid_workflow".to_string(),
        description: None,
        version: "1.0.0".to_string(),
        tasks: vec![
            Task {
                id: TaskId::parse_str("task1").unwrap(),
                name: "Task 1".to_string(),
                description: None,
                pattern: PatternId::parse_str("sequence").unwrap(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                successors: Vec::new(),
                guards: Vec::new(),
            },
            Task {
                id: TaskId::parse_str("task2").unwrap(),
                name: "Task 2 (Dead)".to_string(),
                description: None,
                pattern: PatternId::parse_str("sequence").unwrap(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                successors: Vec::new(),
                guards: Vec::new(),
            },
        ],
        metadata: HashMap::new(),
    };

    let state_store = StateStore::new_in_memory();
    let engine = Arc::new(WorkflowEngine::new(state_store));
    let spec_id = spec.id;

    engine.register_spec(spec).await.unwrap();

    let ctx = PhaseContext::new(engine, spec_id);
    let executor = PhaseExecutor::new();
    let phase = FormalSoundnessPhase::new();

    let result = executor.execute_phase(&phase, ctx).await.unwrap();

    // Should detect dead task (task2 is unreachable)
    assert_eq!(result.status, PhaseStatus::Fail);
    assert!(!result.data.no_dead_tasks);
}

#[tokio::test]
async fn test_phase_composition() {
    use knhk_workflow_engine::validation::phases::core::ComposedPhase;

    let spec = create_simple_workflow();
    let state_store = StateStore::new_in_memory();
    let engine = Arc::new(WorkflowEngine::new(state_store));
    let spec_id = spec.id;

    engine.register_spec(spec).await.unwrap();

    let ctx = PhaseContext::new(engine, spec_id);
    let executor = PhaseExecutor::new();

    // Compose two phases
    let phase1 = FormalSoundnessPhase::new();
    let phase2 = PatternSemanticsPhase::new();
    let composed = ComposedPhase::new(phase1, phase2);

    let result = executor.execute_phase(&composed, ctx).await.unwrap();

    assert_eq!(result.status, PhaseStatus::Pass);
    assert!(result.data.first.option_to_complete);
    assert!(result.data.second.total_patterns > 0);
}
