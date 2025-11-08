//! Chicago TDD tests for error handling
//! Tests QueryError and WarmPathError variants, invalid input handling, parse failures

use knhk_warm::query::{
    execute_ask, execute_construct, execute_describe, execute_select, QueryError,
};
use knhk_warm::WarmPathExecutor;
use knhk_warm::{WarmPathError, WarmPathGraph};

#[test]
fn test_query_error_parse_error() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    graph
        .load_from_turtle("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    let invalid_query = "SELECT ?s WHERE { INVALID SYNTAX";
    let result = execute_select(&graph, invalid_query);

    assert!(result.is_err(), "Invalid query should return error");

    match result.unwrap_err() {
        QueryError::ParseError(msg) => {
            assert!(!msg.is_empty(), "Parse error should have message");
        }
        QueryError::ExecutionError(msg) => {
            assert!(!msg.is_empty(), "Execution error should have message");
        }
        e => panic!("Expected ParseError or ExecutionError, got: {:?}", e),
    }
}

#[test]
fn test_query_error_execution_error() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    // Empty graph - query should still succeed
    let query = "SELECT ?s WHERE { ?s <p1> ?o }";
    let result = execute_select(&graph, query);

    assert!(result.is_ok(), "Query should succeed even with empty graph");

    let select_result = result.unwrap();
    assert_eq!(
        select_result.bindings.len(),
        0,
        "Should return empty result"
    );
}

#[test]
fn test_graph_error_invalid_turtle() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    let invalid_turtle = "This is not valid Turtle syntax !!!";
    let result = graph.load_from_turtle(invalid_turtle);

    assert!(result.is_err(), "Invalid Turtle should return error");

    let error_msg = result.unwrap_err();
    assert!(
        error_msg.contains("Failed to load")
            || error_msg.contains("parse")
            || error_msg.contains("Turtle"),
        "Error should mention loading or parsing issue: {}",
        error_msg
    );
}

#[test]
fn test_graph_error_invalid_iri() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    // Invalid IRI (contains spaces)
    let result = graph.insert_triple(
        "invalid iri with spaces",
        "http://example.org/p1",
        "http://example.org/o1",
    );

    assert!(result.is_err(), "Invalid IRI should return error");

    let error_msg = result.unwrap_err();
    assert!(
        error_msg.contains("Invalid") || error_msg.contains("IRI"),
        "Error should mention IRI issue: {}",
        error_msg
    );
}

#[test]
fn test_executor_error_invalid_query() {
    let executor = WarmPathExecutor::new().expect("Failed to create executor");

    executor
        .load_rdf("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    let invalid_query = "INVALID SPARQL SYNTAX {";
    let result = executor.execute_query(invalid_query);

    assert!(result.is_err(), "Invalid query should return error");

    let error_msg = result.unwrap_err();
    assert!(
        error_msg.contains("parse") || error_msg.contains("failed") || error_msg.contains("error"),
        "Error should mention failure: {}",
        error_msg
    );
}

#[test]
fn test_executor_error_empty_query() {
    let executor = WarmPathExecutor::new().expect("Failed to create executor");

    executor
        .load_rdf("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    let empty_query = "";
    let result = executor.execute_query(empty_query);

    assert!(result.is_err(), "Empty query should return error");
}

#[test]
fn test_query_error_display_implementations() {
    // Test that all QueryError variants have proper Display implementations
    let parse_error = QueryError::ParseError("Parse failed".to_string());
    let execution_error = QueryError::ExecutionError("Execution failed".to_string());
    let unsupported_error = QueryError::UnsupportedQueryType("Unsupported".to_string());

    let parse_msg = format!("{}", parse_error);
    let exec_msg = format!("{}", execution_error);
    let unsupported_msg = format!("{}", unsupported_error);

    assert!(!parse_msg.is_empty(), "ParseError should have message");
    assert!(!exec_msg.is_empty(), "ExecutionError should have message");
    assert!(
        !unsupported_msg.is_empty(),
        "UnsupportedQueryType should have message"
    );
}

#[test]
fn test_graph_error_file_not_found() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    let nonexistent_path = std::path::PathBuf::from("/nonexistent/path/to/file.ttl");
    let result = graph.load_from_file(&nonexistent_path);

    assert!(result.is_err(), "Nonexistent file should return error");

    let error_msg = result.unwrap_err();
    assert!(
        error_msg.contains("Failed to open")
            || error_msg.contains("No such file")
            || error_msg.contains("not found"),
        "Error should mention file issue: {}",
        error_msg
    );
}

#[test]
fn test_query_error_propagation() {
    // Test that errors from graph.query() propagate correctly
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    graph
        .load_from_turtle("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    // Invalid query
    let invalid_query = "SELECT ?s WHERE { INVALID";
    let result = execute_select(&graph, invalid_query);

    assert!(result.is_err(), "Error should propagate");

    // Error should be QueryError, not String
    match result.unwrap_err() {
        QueryError::ParseError(_) | QueryError::ExecutionError(_) => {}
        e => panic!("Expected QueryError variant, got: {:?}", e),
    }
}

#[test]
fn test_executor_error_context() {
    let executor = WarmPathExecutor::new().expect("Failed to create executor");

    executor
        .load_rdf("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    // Query that fails
    let invalid_query = "INVALID";
    let result = executor.execute_query(invalid_query);

    assert!(result.is_err(), "Should return error");

    let error_msg = result.unwrap_err();
    assert!(!error_msg.is_empty(), "Error should have message");
    assert!(error_msg.len() > 10, "Error message should be descriptive");
}

#[test]
fn test_query_error_unwrap_safety() {
    // Test that errors can be safely unwrapped for testing
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    graph
        .load_from_turtle("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    let invalid_query = "SELECT ?s WHERE { INVALID";
    let result = execute_select(&graph, invalid_query);

    // Should be able to match on error variants
    match result {
        Ok(_) => panic!("Should have returned error"),
        Err(QueryError::ParseError(_)) => {}
        Err(QueryError::ExecutionError(_)) => {}
        Err(QueryError::UnsupportedQueryType(_)) => {}
    }
}
