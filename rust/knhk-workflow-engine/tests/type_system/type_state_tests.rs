//! Type-state pattern integration tests

use knhk_workflow_engine::builders::{CaseBuilder, TaskExecution, WorkflowBuilder};
use knhk_workflow_engine::case::CaseState;
use knhk_workflow_engine::parser::WorkflowSpecId;

#[test]
fn test_case_builder_type_state_flow() {
    let spec_id = WorkflowSpecId::new("test-workflow".to_string());

    let case = CaseBuilder::new()
        .with_spec_id(spec_id.clone())
        .with_data(serde_json::json!({"input": "test"}))
        .build();

    assert_eq!(case.state, CaseState::Created);
    assert_eq!(case.spec_id, spec_id);
}

#[test]
fn test_case_builder_with_empty_data() {
    let spec_id = WorkflowSpecId::new("test-workflow".to_string());

    let case = CaseBuilder::new()
        .with_spec_id(spec_id)
        .with_empty_data()
        .build();

    assert_eq!(case.state, CaseState::Created);
    assert_eq!(case.data, serde_json::json!({}));
}

// This test demonstrates compile-time safety
// Uncommenting the following would cause a compile error:
/*
#[test]
fn test_case_builder_cannot_build_without_data() {
    let spec_id = WorkflowSpecId::new("test".to_string());

    // ERROR: No method named `build` found for struct `CaseBuilder<HasSpecId>`
    let case = CaseBuilder::new()
        .with_spec_id(spec_id)
        .build();  // ← This won't compile!
}
*/

#[test]
fn test_workflow_builder_state_transitions() {
    let spec_id = WorkflowSpecId::new("workflow-123".to_string());

    let result = WorkflowBuilder::new()
        .with_spec_id(spec_id.clone())
        .add_pattern(1)
        .add_pattern(2)
        .configure()
        .and_then(|b| b.validate())
        .map(|b| b.build());

    assert!(result.is_ok());
    let (built_spec_id, _definition, patterns) = result.unwrap();
    assert_eq!(built_spec_id, spec_id);
    assert_eq!(patterns.len(), 2);
}

#[test]
fn test_workflow_builder_validation_failure() {
    let spec_id = WorkflowSpecId::new("workflow-123".to_string());

    // Missing patterns - should fail validation
    let result = WorkflowBuilder::new()
        .with_spec_id(spec_id)
        .configure()
        .and_then(|b| b.validate());

    assert!(result.is_err());
}

#[test]
fn test_workflow_builder_missing_spec_id() {
    let result = WorkflowBuilder::new()
        .add_pattern(1)
        .configure();

    assert!(result.is_err());
}

#[test]
fn test_task_execution_state_flow() {
    let task = TaskExecution::new("task-1".to_string());

    // Start the task
    let executing = task.start();
    assert_eq!(executing.task_id(), "task-1");

    // Complete the task
    std::thread::sleep(std::time::Duration::from_millis(10));
    let completed = executing.complete(serde_json::json!({"status": "success"}));

    assert_eq!(completed.result(), &serde_json::json!({"status": "success"}));

    // Duration should be positive
    let duration = completed.duration();
    assert!(duration.is_some());
    assert!(duration.unwrap().num_milliseconds() >= 0);
}

// This test demonstrates compile-time safety
// Uncommenting the following would cause a compile error:
/*
#[test]
fn test_task_cannot_complete_without_start() {
    let task = TaskExecution::new("task-1".to_string());

    // ERROR: No method named `complete` found for struct `TaskExecution<TaskPending>`
    let completed = task.complete(serde_json::json!({}));  // ← Won't compile!
}
*/

#[test]
fn test_multiple_workflows_with_builder() {
    let workflows: Vec<_> = (0..3)
        .map(|i| {
            WorkflowBuilder::new()
                .with_spec_id(WorkflowSpecId::new(format!("workflow-{}", i)))
                .add_pattern(1)
                .configure()
                .and_then(|b| b.validate())
                .map(|b| b.build())
        })
        .collect();

    assert_eq!(workflows.len(), 3);
    assert!(workflows.iter().all(|w| w.is_ok()));
}

#[test]
fn test_builder_default_constructors() {
    // Test default implementations
    let case_builder = CaseBuilder::default();
    let workflow_builder = WorkflowBuilder::default();

    // These should compile without errors
    let _ = std::mem::size_of_val(&case_builder);
    let _ = std::mem::size_of_val(&workflow_builder);
}
