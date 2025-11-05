//! Chicago TDD tests for query module
//! Tests execute_construct(), execute_describe(), result conversion functions

use knhk_warm::{WarmPathGraph, execute_select, execute_ask, execute_construct, execute_describe};
use knhk_warm::query::{select_to_json, ask_to_json, construct_to_json, QueryError};

#[test]
fn test_execute_construct() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    let test_data = r#"
        <s1> <p1> <o1> .
        <s2> <p1> <o2> .
    "#;
    graph.load_from_turtle(test_data).expect("Failed to load data");
    
    let query = "CONSTRUCT { ?s <p1> <o1> } WHERE { ?s <p1> ?o }";
    let result = execute_construct(&graph, query);
    
    assert!(result.is_ok(), "CONSTRUCT query should succeed");
    
    let construct_result = result.unwrap();
    assert!(!construct_result.triples.is_empty(), "Should construct triples");
    assert!(construct_result.triples.len() >= 1, "Should have at least 1 triple");
}

#[test]
fn test_execute_construct_empty_result() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    // Query that doesn't match
    let query = "CONSTRUCT { ?s <p2> <o2> } WHERE { ?s <p2> <o2> }";
    let result = execute_construct(&graph, query);
    
    assert!(result.is_ok(), "CONSTRUCT query should succeed even with no matches");
    
    let construct_result = result.unwrap();
    assert_eq!(construct_result.triples.len(), 0, "Should have no triples for non-matching query");
}

#[test]
fn test_execute_describe() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    let test_data = r#"
        <s1> <p1> <o1> .
        <s1> <p2> <o2> .
        <s2> <p1> <o3> .
    "#;
    graph.load_from_turtle(test_data).expect("Failed to load data");
    
    let query = "DESCRIBE <s1>";
    let result = execute_describe(&graph, query);
    
    assert!(result.is_ok(), "DESCRIBE query should succeed");
    
    let describe_result = result.unwrap();
    assert!(!describe_result.triples.is_empty(), "Should describe resource");
    assert!(describe_result.triples.len() >= 2, "Should have at least 2 triples describing s1");
}

#[test]
fn test_execute_describe_nonexistent() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    // Describe resource that doesn't exist
    let query = "DESCRIBE <s999>";
    let result = execute_describe(&graph, query);
    
    assert!(result.is_ok(), "DESCRIBE should succeed even for nonexistent resource");
    
    let describe_result = result.unwrap();
    assert_eq!(describe_result.triples.len(), 0, "Should have no triples for nonexistent resource");
}

#[test]
fn test_execute_query_parse_error() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    let invalid_query = "INVALID SPARQL SYNTAX {";
    let result = execute_select(&graph, invalid_query);
    
    assert!(result.is_err(), "Invalid query should return error");
    
    match result.unwrap_err() {
        QueryError::ParseError(_) => {}
        QueryError::ExecutionError(_) => {}
        e => panic!("Expected ParseError or ExecutionError, got: {:?}", e),
    }
}

#[test]
fn test_select_to_json() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    let query = "SELECT ?s ?o WHERE { ?s <p1> ?o }";
    let result = execute_select(&graph, query).expect("Query should succeed");
    
    let json = select_to_json(&result);
    
    assert!(json.is_array(), "JSON should be an array");
    
    if let serde_json::Value::Array(arr) = json {
        assert!(!arr.is_empty(), "Array should not be empty");
        
        if let Some(first) = arr.first() {
            assert!(first.is_object(), "Each element should be an object");
        }
    }
}

#[test]
fn test_ask_to_json() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    let query = "ASK { <s1> <p1> <o1> }";
    let result = execute_ask(&graph, query).expect("Query should succeed");
    
    let json = ask_to_json(&result);
    
    assert!(json.is_boolean(), "JSON should be a boolean");
    
    if let serde_json::Value::Bool(b) = json {
        assert!(b, "ASK query should return true");
    }
}

#[test]
fn test_construct_to_json() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    let query = "CONSTRUCT { ?s <p1> <o1> } WHERE { ?s <p1> ?o }";
    let result = execute_construct(&graph, query).expect("Query should succeed");
    
    let json = construct_to_json(&result);
    
    assert!(json.is_array(), "JSON should be an array");
    
    if let serde_json::Value::Array(arr) = json {
        assert!(!arr.is_empty(), "Array should contain triples");
        
        for item in arr {
            assert!(item.is_string(), "Each triple should be a string");
        }
    }
}

#[test]
fn test_execute_select_empty_result() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    // Query that doesn't match
    let query = "SELECT ?s WHERE { ?s <p999> ?o }";
    let result = execute_select(&graph, query);
    
    assert!(result.is_ok(), "Query should succeed even with no matches");
    
    let select_result = result.unwrap();
    assert_eq!(select_result.bindings.len(), 0, "Should have no bindings for non-matching query");
    assert!(!select_result.variables.is_empty(), "Should still have variables defined");
}

#[test]
fn test_execute_ask_false() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    // Query that doesn't match
    let query = "ASK { <s1> <p999> <o999> }";
    let result = execute_ask(&graph, query);
    
    assert!(result.is_ok(), "ASK query should succeed");
    
    let ask_result = result.unwrap();
    assert!(!ask_result.result, "ASK query should return false for non-matching pattern");
}

#[test]
fn test_select_result_variables() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    let query = "SELECT ?s ?p ?o WHERE { ?s ?p ?o }";
    let result = execute_select(&graph, query).expect("Query should succeed");
    
    assert_eq!(result.variables.len(), 3, "Should have 3 variables");
    assert!(result.variables.contains(&"s".to_string()), "Should have variable 's'");
    assert!(result.variables.contains(&"p".to_string()), "Should have variable 'p'");
    assert!(result.variables.contains(&"o".to_string()), "Should have variable 'o'");
}

