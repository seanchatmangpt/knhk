//! Chicago TDD Tests for Runtime RDF Query API
//!
//! These tests follow Chicago TDD principles:
//! - RED phase: Tests are EXPECTED TO FAIL until runtime RDF query API is implemented
//! - State-based testing: Verify query results
//! - Real collaborators: Use actual WorkflowEngine
//! - AAA pattern: Arrange, Act, Assert
//!
//! The workflow engine should allow SPARQL queries at runtime to inspect
//! workflow state, tasks, conditions, and variables.

use knhk_workflow_engine::{StateStore, WorkflowEngine, WorkflowParser};
use std::collections::HashMap;

// ============================================================================
// Gap Test: Runtime RDF Query API
// ============================================================================

#[tokio::test]
async fn test_runtime_rdf_query_workflow_specification() {
    // Arrange: Create engine and load workflow
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");
    let engine = WorkflowEngine::new(state_store);

    let turtle = r#"
        @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

        <http://example.org/workflow1> a yawl:Specification ;
            yawl:specName "TestWorkflow" ;
            yawl:hasInputCondition <http://example.org/input1> ;
            yawl:hasOutputCondition <http://example.org/output1> .

        <http://example.org/input1> a yawl:InputCondition ;
            yawl:conditionName "Start" .

        <http://example.org/output1> a yawl:OutputCondition ;
            yawl:conditionName "End" .
    "#;

    let mut parser = WorkflowParser::new().expect("Failed to create parser");
    let spec = parser
        .parse_turtle(turtle)
        .expect("Failed to parse workflow");
    engine
        .register_workflow(spec)
        .await
        .expect("Failed to register workflow");

    // Act: Query RDF at runtime
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT ?s ?name WHERE {
            ?s a yawl:Specification .
            ?s yawl:specName ?name .
        }
    "#;

    let spec_id = knhk_workflow_engine::WorkflowSpecId::new(); // Dummy spec_id for now
    let results = engine.query_rdf(&spec_id, query).await;

    // Assert: Should return workflow specification
    assert!(
        results.is_ok(),
        "Runtime RDF query should succeed, got error: {:?}",
        results.err()
    );

    let bindings = results.expect("Query should succeed");
    assert_eq!(
        bindings.len(),
        1,
        "Should return exactly one workflow specification"
    );

    let first_result = &bindings[0];
    assert!(
        first_result.contains_key("name"),
        "Result should contain 'name' binding"
    );
    assert_eq!(
        first_result.get("name").map(|s| s.as_str()),
        Some("TestWorkflow"),
        "Workflow name should be 'TestWorkflow'"
    );
}

#[tokio::test]
async fn test_runtime_rdf_query_task_count() {
    // Arrange: Create engine with workflow containing tasks
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");
    let engine = WorkflowEngine::new(state_store);

    let turtle = r#"
        @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .

        <http://example.org/workflow1> a yawl:Specification ;
            yawl:specName "MultiTaskWorkflow" ;
            yawl:hasInputCondition <http://example.org/input1> ;
            yawl:hasTask <http://example.org/task1> ;
            yawl:hasTask <http://example.org/task2> ;
            yawl:hasTask <http://example.org/task3> ;
            yawl:hasOutputCondition <http://example.org/output1> .

        <http://example.org/task1> a yawl:AtomicTask ;
            yawl:taskName "Task1" .

        <http://example.org/task2> a yawl:AtomicTask ;
            yawl:taskName "Task2" .

        <http://example.org/task3> a yawl:CompositeTask ;
            yawl:taskName "Task3" .
    "#;

    let mut parser = WorkflowParser::new().expect("Failed to create parser");
    let spec = parser
        .parse_turtle(turtle)
        .expect("Failed to parse workflow");
    engine
        .register_workflow(spec)
        .await
        .expect("Failed to register workflow");

    // Act: Query task count
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT (COUNT(?task) as ?taskCount) WHERE {
            ?spec a yawl:Specification .
            ?spec yawl:hasTask ?task .
        }
    "#;

    let spec_id = knhk_workflow_engine::WorkflowSpecId::new(); // Dummy spec_id for now
    let results = engine.query_rdf(&spec_id, query).await;

    // Assert: Should return 3 tasks
    assert!(results.is_ok(), "Task count query should succeed");

    let bindings = results.expect("Query should succeed");
    assert_eq!(bindings.len(), 1, "Should return one result");

    let task_count = bindings[0]
        .get("taskCount")
        .and_then(|s| s.parse::<u32>().ok())
        .expect("Should have taskCount binding as integer");

    assert_eq!(task_count, 3, "Should have 3 tasks");
}

#[tokio::test]
async fn test_runtime_rdf_query_case_variables() {
    // Arrange: Create case with variables
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");
    let engine = WorkflowEngine::new(state_store);

    let turtle = r#"
        @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .

        <http://example.org/workflow1> a yawl:Specification ;
            yawl:specName "VarWorkflow" ;
            yawl:hasInputCondition <http://example.org/input1> ;
            yawl:hasOutputCondition <http://example.org/output1> ;
            yawl:hasVariable <http://example.org/var1> .

        <http://example.org/var1> a yawl:Variable ;
            yawl:varName "orderAmount" ;
            yawl:varType "decimal" .
    "#;

    let mut parser = WorkflowParser::new().expect("Failed to create parser");
    let spec = parser
        .parse_turtle(turtle)
        .expect("Failed to parse workflow");
    engine
        .register_workflow(spec)
        .await
        .expect("Failed to register workflow");

    let mut case_data = HashMap::new();
    case_data.insert("orderAmount".to_string(), serde_json::json!(99.99));

    let spec_id = knhk_workflow_engine::WorkflowSpecId::new(); // Dummy spec_id for now
    let case_id = engine
        .create_case(spec_id, serde_json::to_value(case_data).unwrap())
        .await
        .expect("Failed to create case");

    // Act: Query case variables
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT ?varName ?varValue WHERE {
            ?case a yawl:Case .
            ?case yawl:hasVariable ?var .
            ?var yawl:varName ?varName .
            ?var yawl:varValue ?varValue .
        }
    "#;

    let results = engine.query_case_rdf(&case_id, query).await;

    // Assert: Should return variable
    assert!(results.is_ok(), "Case variable query should succeed");

    let bindings = results.expect("Query should succeed");
    assert!(!bindings.is_empty(), "Should return at least one variable");

    let var_binding = bindings
        .iter()
        .find(|b| b.get("varName").map(|s| s.as_str()) == Some("orderAmount"));

    assert!(var_binding.is_some(), "Should find 'orderAmount' variable");
}

#[tokio::test]
async fn test_runtime_rdf_query_pattern_dependencies() {
    // Arrange: Query pattern metadata from RDF
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");
    let engine = WorkflowEngine::new(state_store);

    // Act: Query pattern dependencies (pattern 3 depends on pattern 2)
    let query = r#"
        PREFIX wp: <http://bitflow.ai/ontology/workflow-pattern/v1#>
        SELECT ?patternId ?dependsOn WHERE {
            ?pattern a wp:WorkflowPattern .
            ?pattern wp:patternId ?patternId .
            ?pattern wp:dependsOn ?dependsOn .
            FILTER(?patternId = 3)
        }
    "#;

    let results = engine.query_pattern_metadata(query).await;

    // Assert: Should return pattern 3 dependencies
    assert!(results.is_ok(), "Pattern metadata query should succeed");

    let bindings = results.expect("Query should succeed");
    assert!(!bindings.is_empty(), "Pattern 3 should have dependencies");

    let has_pattern_2_dep = bindings
        .iter()
        .any(|b| b.get("dependsOn").and_then(|s| s.parse::<u32>().ok()) == Some(2));

    assert!(
        has_pattern_2_dep,
        "Pattern 3 (Synchronization) should depend on Pattern 2 (Parallel Split)"
    );
}

#[tokio::test]
async fn test_runtime_rdf_query_with_filter() {
    // Arrange: Create workflow with multiple tasks
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");
    let engine = WorkflowEngine::new(state_store);

    let turtle = r#"
        @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .

        <http://example.org/workflow1> a yawl:Specification ;
            yawl:specName "FilterTest" ;
            yawl:hasInputCondition <http://example.org/input1> ;
            yawl:hasTask <http://example.org/task1> ;
            yawl:hasTask <http://example.org/task2> ;
            yawl:hasOutputCondition <http://example.org/output1> .

        <http://example.org/task1> a yawl:AtomicTask ;
            yawl:taskName "AtomicTask1" ;
            yawl:join "XOR" ;
            yawl:split "AND" .

        <http://example.org/task2> a yawl:CompositeTask ;
            yawl:taskName "CompositeTask1" ;
            yawl:join "AND" ;
            yawl:split "XOR" .
    "#;

    let mut parser = WorkflowParser::new().expect("Failed to create parser");
    let spec = parser
        .parse_turtle(turtle)
        .expect("Failed to parse workflow");
    engine
        .register_workflow(spec)
        .await
        .expect("Failed to register workflow");

    // Act: Query only atomic tasks
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT ?task ?name WHERE {
            ?task a yawl:AtomicTask .
            ?task yawl:taskName ?name .
        }
    "#;

    let spec_id = knhk_workflow_engine::WorkflowSpecId::new(); // Dummy spec_id for now
    let results = engine.query_rdf(&spec_id, query).await;

    // Assert: Should return only atomic task
    assert!(results.is_ok(), "Filtered query should succeed");

    let bindings = results.expect("Query should succeed");
    assert_eq!(bindings.len(), 1, "Should return exactly one atomic task");
    assert_eq!(
        bindings[0].get("name").map(|s| s.as_str()),
        Some("AtomicTask1")
    );
}

#[tokio::test]
async fn test_runtime_rdf_query_invalid_sparql_returns_error() {
    // Arrange: Create engine
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");
    let engine = WorkflowEngine::new(state_store);

    let turtle = r#"
        @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
        <http://example.org/workflow1> a yawl:Specification .
    "#;

    let mut parser = WorkflowParser::new().expect("Failed to create parser");
    let spec = parser
        .parse_turtle(turtle)
        .expect("Failed to parse workflow");
    engine
        .register_workflow(spec)
        .await
        .expect("Failed to register workflow");

    // Act: Execute invalid SPARQL query
    let invalid_query = "INVALID SPARQL SYNTAX";

    let spec_id = knhk_workflow_engine::WorkflowSpecId::new(); // Dummy spec_id for now
    let results = engine.query_rdf(&spec_id, invalid_query).await;

    // Assert: Should return error
    assert!(results.is_err(), "Invalid SPARQL query should return error");
}

#[tokio::test]
async fn test_runtime_rdf_query_empty_results() {
    // Arrange: Create workflow
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");
    let engine = WorkflowEngine::new(state_store);

    let turtle = r#"
        @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
        <http://example.org/workflow1> a yawl:Specification ;
            yawl:hasInputCondition <http://example.org/input1> ;
            yawl:hasOutputCondition <http://example.org/output1> .
    "#;

    let mut parser = WorkflowParser::new().expect("Failed to create parser");
    let spec = parser
        .parse_turtle(turtle)
        .expect("Failed to parse workflow");
    engine
        .register_workflow(spec)
        .await
        .expect("Failed to register workflow");

    // Act: Query for non-existent tasks
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT ?task WHERE {
            ?task a yawl:AtomicTask .
        }
    "#;

    let spec_id = knhk_workflow_engine::WorkflowSpecId::new(); // Dummy spec_id for now
    let results = engine.query_rdf(&spec_id, query).await;

    // Assert: Should return empty results (not error)
    assert!(
        results.is_ok(),
        "Query for non-existent elements should succeed with empty results"
    );

    let bindings = results.expect("Query should succeed");
    assert!(
        bindings.is_empty(),
        "Should return empty results when no matches"
    );
}

// ============================================================================
// Extension Methods for WorkflowEngine (RED PHASE - Not Implemented)
// ============================================================================

// These trait extensions define the API we expect.
// They will be implemented in the GREEN phase.

use knhk_workflow_engine::{CaseId, WorkflowSpecId};

#[allow(dead_code)]
trait RuntimeRdfQueryExt {
    async fn query_rdf(
        &self,
        spec_id: &WorkflowSpecId,
        sparql_query: &str,
    ) -> Result<Vec<HashMap<String, String>>, String>;

    async fn query_case_rdf(
        &self,
        case_id: &CaseId,
        sparql_query: &str,
    ) -> Result<Vec<HashMap<String, String>>, String>;

    async fn query_pattern_metadata(
        &self,
        sparql_query: &str,
    ) -> Result<Vec<HashMap<String, String>>, String>;
}

impl RuntimeRdfQueryExt for WorkflowEngine {
    async fn query_rdf(
        &self,
        _spec_id: &WorkflowSpecId,
        _sparql_query: &str,
    ) -> Result<Vec<HashMap<String, String>>, String> {
        // RED PHASE: Not implemented yet
        Err("Runtime RDF query not yet implemented".to_string())
    }

    async fn query_case_rdf(
        &self,
        _case_id: &CaseId,
        _sparql_query: &str,
    ) -> Result<Vec<HashMap<String, String>>, String> {
        // RED PHASE: Not implemented yet
        Err("Runtime case RDF query not yet implemented".to_string())
    }

    async fn query_pattern_metadata(
        &self,
        _sparql_query: &str,
    ) -> Result<Vec<HashMap<String, String>>, String> {
        // RED PHASE: Not implemented yet
        Err("Pattern metadata RDF query not yet implemented".to_string())
    }
}
