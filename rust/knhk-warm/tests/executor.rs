//! Chicago TDD tests for WarmPathExecutor
//! Tests automatic path selection, warm/cold path execution, unified result format

use knhk_etl::path_selector::QueryPath;
use knhk_warm::query::QueryExecutionResult;
use knhk_warm::{WarmPathExecutor, WarmPathGraph};

#[test]
fn test_executor_creation() {
    let executor = WarmPathExecutor::new();
    assert!(executor.is_ok(), "Executor should be created successfully");

    let executor = executor.unwrap();
    assert_eq!(
        executor.graph().size(),
        0,
        "New executor should have empty graph"
    );
}

#[test]
fn test_load_rdf() {
    let executor = WarmPathExecutor::new().expect("Failed to create executor");

    let turtle_data = r#"
        <s1> <p1> <o1> .
        <s2> <p1> <o2> .
    "#;

    let result = executor.load_rdf(turtle_data);
    assert!(result.is_ok(), "Should load RDF data successfully");

    assert_eq!(executor.graph().size(), 2, "Graph should contain 2 triples");
}

#[test]
fn test_execute_query_warm_path_select() {
    let executor = WarmPathExecutor::new().expect("Failed to create executor");

    let turtle_data = r#"
        <s1> <p1> <o1> .
        <s2> <p1> <o2> .
    "#;
    executor.load_rdf(turtle_data).expect("Failed to load data");

    let query = "SELECT ?s ?o WHERE { ?s <p1> ?o }";
    let result = executor.execute_query(query);

    assert!(result.is_ok(), "Query should execute successfully");

    match result.unwrap() {
        QueryExecutionResult::Select(select) => {
            assert_eq!(select.bindings.len(), 2, "Should return 2 bindings");
            assert!(
                select.variables.contains(&"s".to_string()),
                "Should have variable 's'"
            );
            assert!(
                select.variables.contains(&"o".to_string()),
                "Should have variable 'o'"
            );
        }
        _ => panic!("Expected Select result"),
    }
}

#[test]
fn test_execute_query_warm_path_ask() {
    let executor = WarmPathExecutor::new().expect("Failed to create executor");

    let turtle_data = r#"
        <s1> <p1> <o1> .
    "#;
    executor.load_rdf(turtle_data).expect("Failed to load data");

    let query = "ASK { <s1> <p1> <o1> }";
    let result = executor.execute_query(query);

    assert!(result.is_ok(), "ASK query should execute successfully");

    match result.unwrap() {
        QueryExecutionResult::Ask(ask) => {
            assert!(ask.result, "ASK query should return true");
        }
        _ => panic!("Expected Ask result"),
    }
}

#[test]
fn test_execute_query_warm_path_construct() {
    let executor = WarmPathExecutor::new().expect("Failed to create executor");

    let turtle_data = r#"
        <s1> <p1> <o1> .
        <s2> <p1> <o2> .
    "#;
    executor.load_rdf(turtle_data).expect("Failed to load data");

    let query = "CONSTRUCT { ?s <p1> <o1> } WHERE { ?s <p1> ?o }";
    let result = executor.execute_query(query);

    assert!(
        result.is_ok(),
        "CONSTRUCT query should execute successfully"
    );

    match result.unwrap() {
        QueryExecutionResult::Construct(construct) => {
            assert!(!construct.triples.is_empty(), "Should construct triples");
        }
        _ => panic!("Expected Construct result"),
    }
}

#[test]
fn test_execute_query_warm_path_describe() {
    let executor = WarmPathExecutor::new().expect("Failed to create executor");

    let turtle_data = r#"
        <s1> <p1> <o1> .
        <s1> <p2> <o2> .
    "#;
    executor.load_rdf(turtle_data).expect("Failed to load data");

    let query = "DESCRIBE <s1>";
    let result = executor.execute_query(query);

    assert!(result.is_ok(), "DESCRIBE query should execute successfully");

    match result.unwrap() {
        QueryExecutionResult::Describe(describe) => {
            assert!(!describe.triples.is_empty(), "Should describe resource");
        }
        _ => panic!("Expected Describe result"),
    }
}

#[test]
fn test_execute_query_path_selection() {
    let executor = WarmPathExecutor::new().expect("Failed to create executor");

    // Load small dataset for hot path
    let turtle_data = r#"
        <s1> <p1> <o1> .
    "#;
    executor.load_rdf(turtle_data).expect("Failed to load data");

    // Hot path query (simple ASK, data size ≤8)
    let hot_query = "ASK { <s1> <p1> <o1> }";
    let result = executor.execute_query(hot_query);
    assert!(result.is_ok(), "Hot path query should execute");

    // Warm path query (SELECT, data size ≤10K)
    let warm_query = "SELECT ?s WHERE { ?s <p1> ?o }";
    let result = executor.execute_query(warm_query);
    assert!(result.is_ok(), "Warm path query should execute");

    match result.unwrap() {
        QueryExecutionResult::Select(_) => {}
        _ => panic!("Expected Select result for warm path"),
    }
}

#[test]
fn test_execute_query_invalid_query() {
    let executor = WarmPathExecutor::new().expect("Failed to create executor");

    executor
        .load_rdf("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    let invalid_query = "INVALID SPARQL SYNTAX {";
    let result = executor.execute_query(invalid_query);

    assert!(result.is_err(), "Invalid query should return error");
    let error_msg = result.unwrap_err();
    assert!(
        error_msg.contains("parse") || error_msg.contains("failed"),
        "Error should mention parse failure: {}",
        error_msg
    );
}

#[test]
fn test_execute_query_empty_query() {
    let executor = WarmPathExecutor::new().expect("Failed to create executor");

    executor
        .load_rdf("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    let empty_query = "";
    let result = executor.execute_query(empty_query);

    assert!(result.is_err(), "Empty query should return error");
}

#[test]
fn test_graph_accessor() {
    let executor = WarmPathExecutor::new().expect("Failed to create executor");

    let graph = executor.graph();
    assert_eq!(graph.size(), 0, "Initial graph should be empty");

    executor
        .load_rdf("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    let graph_after = executor.graph();
    assert_eq!(graph_after.size(), 1, "Graph should reflect loaded data");
}

#[test]
fn test_executor_idempotence() {
    let executor = WarmPathExecutor::new().expect("Failed to create executor");

    let turtle_data = r#"
        <s1> <p1> <o1> .
    "#;
    executor.load_rdf(turtle_data).expect("Failed to load data");

    let query = "SELECT ?s WHERE { ?s <p1> ?o }";

    // Execute query multiple times - should be idempotent
    for _ in 0..5 {
        let result = executor.execute_query(query);
        assert!(result.is_ok(), "Query should succeed on repeated execution");

        match result.unwrap() {
            QueryExecutionResult::Select(select) => {
                assert_eq!(
                    select.bindings.len(),
                    1,
                    "Should consistently return 1 binding"
                );
            }
            _ => panic!("Expected Select result"),
        }
    }
}

#[test]
fn test_executor_state_consistency() {
    let executor = WarmPathExecutor::new().expect("Failed to create executor");

    // Load initial data
    executor
        .load_rdf("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    let query = "SELECT ?s WHERE { ?s <p1> ?o }";
    let result1 = executor.execute_query(query).expect("Query should succeed");

    // Load more data
    executor
        .load_rdf("<s2> <p1> <o2> .")
        .expect("Failed to load data");

    let result2 = executor.execute_query(query).expect("Query should succeed");

    // Results should reflect new data
    match (result1, result2) {
        (QueryExecutionResult::Select(select1), QueryExecutionResult::Select(select2)) => {
            assert!(
                select2.bindings.len() > select1.bindings.len(),
                "Result should reflect new data: {} vs {}",
                select1.bindings.len(),
                select2.bindings.len()
            );
        }
        _ => panic!("Both should be Select results"),
    }
}
