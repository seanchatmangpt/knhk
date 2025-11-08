//! Performance tests for warm path queries
//! Test query execution time, cache hit rates, path selection accuracy

use knhk_etl::path_selector::{select_path, QueryPath};
use knhk_warm::{execute_ask, execute_select, WarmPathGraph};
use std::time::Instant;

#[test]
fn test_query_execution_time() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    let test_data = r#"
        <s1> <p1> <o1> .
        <s2> <p1> <o2> .
        <s3> <p2> <o1> .
    "#;
    graph
        .load_from_turtle(test_data)
        .expect("Failed to load data");

    let query = "SELECT ?s ?o WHERE { ?s <p1> ?o }";

    let start = Instant::now();
    let result = execute_select(&graph, query);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Query should succeed");
    assert!(
        duration.as_millis() < 500,
        "Query should complete in <500ms, took {}ms",
        duration.as_millis()
    );
}

#[test]
fn test_cache_hit_rate() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    let test_data = r#"
        <s1> <p1> <o1> .
        <s2> <p1> <o2> .
    "#;
    graph
        .load_from_turtle(test_data)
        .expect("Failed to load data");

    let query = "SELECT ?s WHERE { ?s <p1> ?o }";

    // Execute query multiple times
    for _ in 0..10 {
        let result = execute_select(&graph, query);
        assert!(result.is_ok(), "Query should succeed");
    }

    // Check cache metrics
    let metrics = graph.get_metrics();
    assert!(metrics.total_queries > 0, "Should have executed queries");
    assert!(metrics.cache_hits > 0, "Should have cache hits");

    // Cache hit rate should be > 0 after multiple identical queries
    assert!(
        metrics.cache_hit_rate > 0.0,
        "Cache hit rate should be > 0: {}",
        metrics.cache_hit_rate
    );
}

#[test]
fn test_path_selection_accuracy() {
    // Test hot path selection
    let hot_query = "ASK { <s> <p> <o> }";
    let path = select_path(hot_query, 5);
    assert_eq!(
        path,
        QueryPath::Hot,
        "Simple ASK with small data should route to hot path"
    );

    // Test warm path selection
    let warm_query = "SELECT ?s ?p ?o WHERE { ?s ?p ?o }";
    let path = select_path(warm_query, 100);
    assert_eq!(
        path,
        QueryPath::Warm,
        "SELECT query should route to warm path"
    );

    // Test warm path with larger data
    let path = select_path(warm_query, 10000);
    assert_eq!(
        path,
        QueryPath::Warm,
        "Should still route to warm path for ≤10K triples"
    );

    // Test cold path selection (UPDATE query)
    let cold_query = "INSERT { <s> <p> <o> } WHERE {}";
    let path = select_path(cold_query, 100);
    assert_eq!(
        path,
        QueryPath::Cold,
        "UPDATE query should route to cold path"
    );

    // Test cold path selection (data size > 10K)
    let path = select_path(warm_query, 10001);
    assert_eq!(
        path,
        QueryPath::Cold,
        "Large dataset should route to cold path"
    );
}

#[test]
fn test_path_selection_hot_path_constraints() {
    // Test that queries with FILTER don't route to hot path
    let query_with_filter = "ASK { <s> <p> <o> FILTER (?o > 10) }";
    let path = select_path(query_with_filter, 5);
    assert_ne!(
        path,
        QueryPath::Hot,
        "Queries with FILTER should not route to hot path"
    );

    // Test that queries with OPTIONAL don't route to hot path
    let query_with_optional = "ASK { <s> <p> <o> OPTIONAL { <s> <p2> <o2> } }";
    let path = select_path(query_with_optional, 5);
    assert_ne!(
        path,
        QueryPath::Hot,
        "Queries with OPTIONAL should not route to hot path"
    );
}

#[test]
fn test_path_selection_warm_path_constraints() {
    // Test that UPDATE queries don't route to warm path
    let update_query = "INSERT { <s> <p> <o> } WHERE {}";
    let path = select_path(update_query, 100);
    assert_ne!(
        path,
        QueryPath::Warm,
        "UPDATE queries should not route to warm path"
    );

    // Test that SHACL queries don't route to warm path
    let shacl_query = "ASK { <s> sh:hasValue <o> }";
    let path = select_path(shacl_query, 100);
    assert_ne!(
        path,
        QueryPath::Warm,
        "SHACL queries should not route to warm path"
    );
}

#[test]
fn test_concurrent_query_execution() {
    use std::sync::Arc;
    use std::thread;

    let graph = Arc::new(WarmPathGraph::new().expect("Failed to create graph"));

    let test_data = r#"
        <s1> <p1> <o1> .
        <s2> <p1> <o2> .
    "#;
    graph
        .load_from_turtle(test_data)
        .expect("Failed to load data");

    let query = "SELECT ?s WHERE { ?s <p1> ?o }";

    // Execute queries concurrently
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let graph_clone = Arc::clone(&graph);
            let query_clone = query.to_string();
            thread::spawn(move || execute_select(&*graph_clone, &query_clone))
        })
        .collect();

    // Wait for all queries to complete
    for handle in handles {
        let result = handle.join().expect("Thread should complete");
        assert!(result.is_ok(), "Concurrent query should succeed");
    }

    // Check metrics
    let metrics = graph.get_metrics();
    assert!(
        metrics.total_queries >= 10,
        "Should have executed at least 10 queries"
    );
}

#[test]
fn test_performance_target_validation() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    // Load dataset of moderate size
    let mut test_data = String::new();
    for i in 0..500 {
        test_data.push_str(&format!("<s{}> <p1> <o{}> .\n", i, i));
    }
    graph
        .load_from_turtle(&test_data)
        .expect("Failed to load data");

    let query = "SELECT ?s ?o WHERE { ?s <p1> ?o } LIMIT 50";

    let start = Instant::now();
    let result = execute_select(&graph, query);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Query should succeed");
    assert!(
        duration.as_millis() <= 500,
        "Query should complete within 500ms target, took {}ms",
        duration.as_millis()
    );
}
