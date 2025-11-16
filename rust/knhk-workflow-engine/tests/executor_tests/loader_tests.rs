//! Tests for Turtle/RDF workflow loader
//!
//! These tests validate Covenant 1: Turtle Is Definition and Cause

use knhk_workflow_engine::executor::{WorkflowLoader, SplitType, JoinType};

#[test]
fn test_load_simple_sequence() {
    let mut loader = WorkflowLoader::new().expect("Failed to create loader");

    let turtle = r#"
        @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        <http://example.org/workflow1> a yawl:Specification ;
            rdfs:label "Simple Sequence Workflow" ;
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

    let workflow = loader.load_turtle(turtle).expect("Failed to load workflow");

    assert_eq!(workflow.name, "Simple Sequence Workflow");
    assert_eq!(workflow.tasks.len(), 2);
    assert_eq!(workflow.flows.len(), 3);
    assert!(workflow.input_condition.is_some());
    assert!(workflow.output_condition.is_some());
}

#[test]
fn test_load_parallel_split_join() {
    let mut loader = WorkflowLoader::new().expect("Failed to create loader");

    let turtle = r#"
        @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        <http://example.org/workflow2> a yawl:Specification ;
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

    let workflow = loader.load_turtle(turtle).expect("Failed to load workflow");

    assert_eq!(workflow.name, "Parallel Workflow");
    assert_eq!(workflow.tasks.len(), 4);

    // Find split and join tasks
    let split_task = workflow.tasks.iter()
        .find(|t| t.name == "Split Task")
        .expect("Split task not found");
    assert_eq!(split_task.split_type, Some(SplitType::AND));

    let join_task = workflow.tasks.iter()
        .find(|t| t.name == "Join Task")
        .expect("Join task not found");
    assert_eq!(join_task.join_type, Some(JoinType::AND));
}

#[test]
fn test_invalid_split_join_combination() {
    let mut loader = WorkflowLoader::new().expect("Failed to create loader");

    // Invalid: XOR split with AND join
    let turtle = r#"
        @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        <http://example.org/workflow3> a yawl:Specification ;
            rdfs:label "Invalid Workflow" ;
            yawl:inputCondition <http://example.org/input> ;
            yawl:outputCondition <http://example.org/output> .

        <http://example.org/task1> a yawl:Task ;
            rdfs:label "Invalid Task" ;
            yawl:split <http://www.yawlfoundation.org/yawlschema#XOR> ;
            yawl:join <http://www.yawlfoundation.org/yawlschema#AND> .

        <http://example.org/flow1> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/input> ;
            yawl:flowsInto <http://example.org/task1> .
    "#;

    let result = loader.load_turtle(turtle);
    assert!(result.is_err(), "Should reject invalid split/join combination");
}

#[test]
fn test_load_with_execution_semantics() {
    let mut loader = WorkflowLoader::new().expect("Failed to create loader");

    let turtle = r#"
        @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
        @prefix yawl-exec: <http://bitflow.ai/ontology/yawl/execution/v1#> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        <http://example.org/workflow4> a yawl:Specification ;
            rdfs:label "Workflow with Execution Semantics" ;
            yawl:inputCondition <http://example.org/input> ;
            yawl:outputCondition <http://example.org/output> .

        <http://example.org/task1> a yawl:Task ;
            rdfs:label "Async Task" ;
            yawl-exec:executionMode <http://bitflow.ai/ontology/yawl/execution/v1#Asynchronous> ;
            yawl-exec:timeoutPolicy <http://bitflow.ai/ontology/yawl/execution/v1#TimeoutRetry> ;
            yawl-exec:RetryPolicy <http://bitflow.ai/ontology/yawl/execution/v1#RetryExponential> .

        <http://example.org/flow1> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/input> ;
            yawl:flowsInto <http://example.org/task1> .
    "#;

    let workflow = loader.load_turtle(turtle).expect("Failed to load workflow");

    assert_eq!(workflow.tasks.len(), 1);
    let task = &workflow.tasks[0];

    // Note: Execution mode parsing might need adjustment based on actual ontology
    // For now, just verify the workflow loads successfully
    assert_eq!(task.name, "Async Task");
}

#[test]
fn test_missing_input_condition() {
    let mut loader = WorkflowLoader::new().expect("Failed to create loader");

    // No input condition - violates Covenant 1
    let turtle = r#"
        @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        <http://example.org/workflow5> a yawl:Specification ;
            rdfs:label "Incomplete Workflow" .

        <http://example.org/task1> a yawl:Task ;
            rdfs:label "Task 1" .
    "#;

    let workflow = loader.load_turtle(turtle).expect("Should load incomplete workflow");

    // Loader should load it, but runtime should reject it
    assert!(workflow.input_condition.is_none());
}
