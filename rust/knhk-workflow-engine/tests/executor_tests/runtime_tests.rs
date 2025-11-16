//! Tests for workflow runtime execution
//!
//! These tests validate that workflows execute according to Turtle definitions

use knhk_workflow_engine::executor::{
    WorkflowLoader, WorkflowRuntime, WorkflowState, TaskExecutor,
    TaskDefinition, TaskResult, ExecutionMode,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Simple mock task executor for testing
struct MockExecutor;

#[async_trait::async_trait]
impl TaskExecutor for MockExecutor {
    async fn execute(
        &self,
        task: &TaskDefinition,
        _input: HashMap<String, serde_json::Value>,
    ) -> knhk_workflow_engine::error::WorkflowResult<TaskResult> {
        // Simulate task execution
        Ok(TaskResult {
            task_id: task.id.clone(),
            success: true,
            output: HashMap::new(),
            error: None,
            duration: Some(std::time::Duration::from_micros(5)), // Within 8-tick limit
        })
    }
}

#[tokio::test]
async fn test_runtime_start() {
    let mut loader = WorkflowLoader::new().expect("Failed to create loader");

    let turtle = r#"
        @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        <http://example.org/workflow1> a yawl:Specification ;
            rdfs:label "Test Workflow" ;
            yawl:inputCondition <http://example.org/input> ;
            yawl:outputCondition <http://example.org/output> .

        <http://example.org/task1> a yawl:Task ;
            rdfs:label "Task 1" .

        <http://example.org/flow1> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/input> ;
            yawl:flowsInto <http://example.org/task1> .

        <http://example.org/flow2> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/task1> ;
            yawl:flowsInto <http://example.org/output> .
    "#;

    let definition = loader.load_turtle(turtle).expect("Failed to load workflow");
    let runtime = WorkflowRuntime::new(definition)
        .with_executor(Arc::new(MockExecutor));

    // Start workflow
    runtime.start().await.expect("Failed to start workflow");

    let state = runtime.get_state().await;
    assert_eq!(state.state, WorkflowState::Running);
    assert!(!state.enabled_tasks.is_empty(), "Should have enabled tasks after start");
}

#[tokio::test]
async fn test_runtime_sequence_execution() {
    let mut loader = WorkflowLoader::new().expect("Failed to create loader");

    let turtle = r#"
        @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        <http://example.org/workflow2> a yawl:Specification ;
            rdfs:label "Sequence Workflow" ;
            yawl:inputCondition <http://example.org/input> ;
            yawl:outputCondition <http://example.org/output> .

        <http://example.org/task1> a yawl:Task ;
            rdfs:label "Task 1" .

        <http://example.org/task2> a yawl:Task ;
            rdfs:label "Task 2" .

        <http://example.org/flow1> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/input> ;
            yawl:flowsInto <http://example.org/task1> .

        <http://example.org/flow2> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/task1> ;
            yawl:flowsInto <http://example.org/task2> .

        <http://example.org/flow3> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/task2> ;
            yawl:flowsInto <http://example.org/output> .
    "#;

    let definition = loader.load_turtle(turtle).expect("Failed to load workflow");
    let runtime = WorkflowRuntime::new(definition)
        .with_executor(Arc::new(MockExecutor));

    // Run workflow to completion
    let final_state = runtime.run().await.expect("Failed to run workflow");

    assert_eq!(final_state.state, WorkflowState::Completed);
    assert_eq!(final_state.completed_tasks.len(), 2);
    assert!(final_state.enabled_tasks.is_empty());
    assert!(final_state.running_tasks.is_empty());
}

#[tokio::test]
async fn test_runtime_parallel_execution() {
    let mut loader = WorkflowLoader::new().expect("Failed to create loader");

    let turtle = r#"
        @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        <http://example.org/workflow3> a yawl:Specification ;
            rdfs:label "Parallel Workflow" ;
            yawl:inputCondition <http://example.org/input> ;
            yawl:outputCondition <http://example.org/output> .

        <http://example.org/split_task> a yawl:Task ;
            rdfs:label "Split Task" ;
            yawl:split <http://www.yawlfoundation.org/yawlschema#AND> .

        <http://example.org/task_a> a yawl:Task ;
            rdfs:label "Task A" .

        <http://example.org/task_b> a yawl:Task ;
            rdfs:label "Task B" .

        <http://example.org/join_task> a yawl:Task ;
            rdfs:label "Join Task" ;
            yawl:join <http://www.yawlfoundation.org/yawlschema#AND> .

        <http://example.org/flow1> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/input> ;
            yawl:flowsInto <http://example.org/split_task> .

        <http://example.org/flow2> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/split_task> ;
            yawl:flowsInto <http://example.org/task_a> .

        <http://example.org/flow3> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/split_task> ;
            yawl:flowsInto <http://example.org/task_b> .

        <http://example.org/flow4> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/task_a> ;
            yawl:flowsInto <http://example.org/join_task> .

        <http://example.org/flow5> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/task_b> ;
            yawl:flowsInto <http://example.org/join_task> .

        <http://example.org/flow6> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/join_task> ;
            yawl:flowsInto <http://example.org/output> .
    "#;

    let definition = loader.load_turtle(turtle).expect("Failed to load workflow");
    let runtime = WorkflowRuntime::new(definition)
        .with_executor(Arc::new(MockExecutor));

    // Run workflow to completion
    let final_state = runtime.run().await.expect("Failed to run workflow");

    assert_eq!(final_state.state, WorkflowState::Completed);

    // All 4 tasks should complete
    assert_eq!(final_state.completed_tasks.len(), 4);

    // Verify both parallel branches completed
    assert!(final_state.completed_tasks.iter().any(|t| t.contains("task_a")));
    assert!(final_state.completed_tasks.iter().any(|t| t.contains("task_b")));
}

#[tokio::test]
async fn test_runtime_missing_input_condition() {
    let mut loader = WorkflowLoader::new().expect("Failed to create loader");

    // No input condition - violates Covenant 1
    let turtle = r#"
        @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        <http://example.org/workflow4> a yawl:Specification ;
            rdfs:label "Incomplete Workflow" .

        <http://example.org/task1> a yawl:Task ;
            rdfs:label "Task 1" .
    "#;

    let definition = loader.load_turtle(turtle).expect("Should load incomplete workflow");
    let runtime = WorkflowRuntime::new(definition);

    // Should fail to start (no input condition = cannot determine where to start)
    let result = runtime.start().await;
    assert!(result.is_err(), "Should fail to start workflow without input condition");
}

#[tokio::test]
async fn test_runtime_cancel() {
    let mut loader = WorkflowLoader::new().expect("Failed to create loader");

    let turtle = r#"
        @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        <http://example.org/workflow5> a yawl:Specification ;
            rdfs:label "Cancellable Workflow" ;
            yawl:inputCondition <http://example.org/input> ;
            yawl:outputCondition <http://example.org/output> .

        <http://example.org/task1> a yawl:Task ;
            rdfs:label "Task 1" .

        <http://example.org/flow1> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/input> ;
            yawl:flowsInto <http://example.org/task1> .
    "#;

    let definition = loader.load_turtle(turtle).expect("Failed to load workflow");
    let runtime = WorkflowRuntime::new(definition);

    runtime.start().await.expect("Failed to start workflow");
    runtime.cancel().await.expect("Failed to cancel workflow");

    let state = runtime.get_state().await;
    assert_eq!(state.state, WorkflowState::Cancelled);
    assert!(state.end_time.is_some());
}
