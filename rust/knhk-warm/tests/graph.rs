//! Chicago TDD tests for WarmPathGraph operations
//! Tests load_from_file(), insert_triple(), insert_quads(), cache invalidation

use knhk_warm::query::execute_select;
use knhk_warm::WarmPathGraph;
use oxigraph::model::{GraphName, NamedNode, Quad, Term};
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_load_from_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_data.ttl");

    // Create test file
    let mut file = File::create(&file_path).expect("Failed to create file");
    writeln!(file, "<s1> <p1> <o1> .").expect("Failed to write");
    writeln!(file, "<s2> <p1> <o2> .").expect("Failed to write");
    drop(file);

    let graph = WarmPathGraph::new().expect("Failed to create graph");
    let result = graph.load_from_file(&file_path);

    assert!(result.is_ok(), "Should load from file successfully");
    assert_eq!(graph.size(), 2, "Graph should contain 2 triples");
}

#[test]
fn test_load_from_file_nonexistent() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    let nonexistent_path = PathBuf::from("/nonexistent/path/to/file.ttl");

    let result = graph.load_from_file(&nonexistent_path);
    assert!(result.is_err(), "Should fail for nonexistent file");

    let error_msg = result.unwrap_err();
    assert!(
        error_msg.contains("Failed to open") || error_msg.contains("No such file"),
        "Error should mention file issue: {}",
        error_msg
    );
}

#[test]
fn test_insert_triple() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    assert_eq!(graph.size(), 0, "Initial graph should be empty");

    let result = graph.insert_triple(
        "http://example.org/s1",
        "http://example.org/p1",
        "http://example.org/o1",
    );

    assert!(result.is_ok(), "Should insert triple successfully");
    assert_eq!(graph.size(), 1, "Graph should contain 1 triple");

    // Verify triple was inserted
    let query = "ASK { <http://example.org/s1> <http://example.org/p1> <http://example.org/o1> }";
    let ask_result = graph.query(query).expect("Query should succeed");

    match ask_result {
        oxigraph::sparql::QueryResults::Boolean(true) => {}
        _ => panic!("ASK query should return true for inserted triple"),
    }
}

#[test]
fn test_insert_triple_invalid_iri() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    // Invalid IRI (contains spaces)
    let result = graph.insert_triple(
        "invalid iri with spaces",
        "http://example.org/p1",
        "http://example.org/o1",
    );

    assert!(result.is_err(), "Should fail for invalid IRI");
    let error_msg = result.unwrap_err();
    assert!(
        error_msg.contains("Invalid") || error_msg.contains("IRI"),
        "Error should mention IRI issue: {}",
        error_msg
    );
}

#[test]
fn test_insert_triple_cache_invalidation() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    // Load initial data
    graph
        .load_from_turtle("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    // Execute query (cache it)
    let query = "SELECT ?s WHERE { ?s <p1> ?o }";
    let result1 = execute_select(&graph, query).expect("Query should succeed");
    let initial_size = result1.bindings.len();

    // Insert new triple
    graph
        .insert_triple("<s2>", "<p1>", "<o2>")
        .expect("Failed to insert");

    // Query should reflect new data (cache should be invalidated)
    let result2 = execute_select(&graph, query).expect("Query should succeed");
    assert!(
        result2.bindings.len() > initial_size,
        "Result should reflect new triple: {} vs {}",
        result2.bindings.len(),
        initial_size
    );
}

#[test]
fn test_insert_quads() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    let s1 = NamedNode::new("http://example.org/s1").expect("Valid IRI");
    let p1 = NamedNode::new("http://example.org/p1").expect("Valid IRI");
    let o1 = NamedNode::new("http://example.org/o1").expect("Valid IRI");

    let s2 = NamedNode::new("http://example.org/s2").expect("Valid IRI");
    let p2 = NamedNode::new("http://example.org/p2").expect("Valid IRI");
    let o2 = NamedNode::new("http://example.org/o2").expect("Valid IRI");

    let quads = vec![
        Quad::new(s1, p1, Term::NamedNode(o1), GraphName::DefaultGraph),
        Quad::new(s2, p2, Term::NamedNode(o2), GraphName::DefaultGraph),
    ];

    let result = graph.insert_quads(&quads);
    assert!(result.is_ok(), "Should insert quads successfully");
    assert_eq!(graph.size(), 2, "Graph should contain 2 quads");
}

#[test]
fn test_insert_quads_empty() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    let quads = vec![];
    let result = graph.insert_quads(&quads);

    assert!(result.is_ok(), "Should handle empty quads");
    assert_eq!(graph.size(), 0, "Graph should remain empty");
}

#[test]
fn test_bump_epoch_cache_invalidation() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    graph
        .load_from_turtle("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    let query = "SELECT ?s WHERE { ?s <p1> ?o }";

    // Execute query (cache it)
    let _result1 = execute_select(&graph, query).expect("Query should succeed");

    // Get initial metrics
    let metrics_before = graph.get_metrics();
    let cache_hits_before = metrics_before.cache_hits;

    // Bump epoch (should invalidate cache)
    graph.bump_epoch();

    // Execute same query again
    let _result2 = execute_select(&graph, query).expect("Query should succeed");

    // Cache should be invalidated, so this should be a cache miss
    let metrics_after = graph.get_metrics();

    // Total queries should increase
    assert!(
        metrics_after.total_queries > metrics_before.total_queries,
        "Total queries should increase: {} vs {}",
        metrics_after.total_queries,
        metrics_before.total_queries
    );
}

#[test]
fn test_load_from_turtle_and_insert() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    // Load initial data
    graph
        .load_from_turtle("<s1> <p1> <o1> .")
        .expect("Failed to load");
    assert_eq!(graph.size(), 1, "Should have 1 triple");

    // Insert additional triple
    graph
        .insert_triple("<s2>", "<p1>", "<o2>")
        .expect("Failed to insert");
    assert_eq!(graph.size(), 2, "Should have 2 triples");

    // Verify both triples exist
    let query = "SELECT ?s WHERE { ?s <p1> ?o }";
    let result = execute_select(&graph, query).expect("Query should succeed");
    assert_eq!(result.bindings.len(), 2, "Should find both triples");
}

#[test]
fn test_graph_size_consistency() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    assert_eq!(graph.size(), 0, "Empty graph should have size 0");

    graph
        .load_from_turtle("<s1> <p1> <o1> .")
        .expect("Failed to load");
    assert_eq!(graph.size(), 1, "Should have size 1");

    graph
        .insert_triple("<s2>", "<p1>", "<o2>")
        .expect("Failed to insert");
    assert_eq!(graph.size(), 2, "Should have size 2");

    graph
        .insert_triple("<s3>", "<p1>", "<o3>")
        .expect("Failed to insert");
    assert_eq!(graph.size(), 3, "Should have size 3");
}

#[test]
fn test_load_from_file_empty_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("empty.ttl");

    // Create empty file
    File::create(&file_path).expect("Failed to create file");

    let graph = WarmPathGraph::new().expect("Failed to create graph");
    let result = graph.load_from_file(&file_path);

    // Empty file should be handled gracefully
    assert!(result.is_ok(), "Should handle empty file");
    assert_eq!(graph.size(), 0, "Empty file should result in empty graph");
}

#[test]
fn test_load_from_file_invalid_turtle() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("invalid.ttl");

    // Create file with invalid Turtle syntax
    let mut file = File::create(&file_path).expect("Failed to create file");
    writeln!(file, "This is not valid Turtle syntax").expect("Failed to write");
    drop(file);

    let graph = WarmPathGraph::new().expect("Failed to create graph");
    let result = graph.load_from_file(&file_path);

    assert!(result.is_err(), "Should fail for invalid Turtle");
    let error_msg = result.unwrap_err();
    assert!(
        error_msg.contains("Failed to load") || error_msg.contains("parse"),
        "Error should mention parsing issue: {}",
        error_msg
    );
}
