//! Chicago TDD tests for edge cases
//! Tests empty queries, invalid SPARQL, large queries, concurrent operations, cache eviction

use knhk_warm::query::QueryError;
use knhk_warm::{execute_ask, execute_select, WarmPathGraph};
use std::sync::Arc;
use std::thread;

#[test]
fn test_empty_query_string() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    graph
        .load_from_turtle("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    let empty_query = "";
    let result = execute_select(&graph, empty_query);

    assert!(result.is_err(), "Empty query should return error");

    match result.unwrap_err() {
        QueryError::ParseError(_) | QueryError::ExecutionError(_) => {}
        e => panic!("Expected ParseError or ExecutionError, got: {:?}", e),
    }
}

#[test]
fn test_whitespace_only_query() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    graph
        .load_from_turtle("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    let whitespace_query = "   \n\t  ";
    let result = execute_select(&graph, whitespace_query);

    assert!(result.is_err(), "Whitespace-only query should return error");
}

#[test]
fn test_invalid_sparql_syntax() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    graph
        .load_from_turtle("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    let invalid_queries = vec![
        "SELECT ?s WHERE {",
        "SELECT ?s WHERE { ?s <p1> ?o } FILTER",
        "SELECT ?s WHERE { ?s <p1> ?o FILTER ?o >",
        "INVALID KEYWORD { ?s <p1> ?o }",
        "SELECT ?s WHERE { ?s <p1> ?o } ORDER BY",
    ];

    for invalid_query in invalid_queries {
        let result = execute_select(&graph, invalid_query);
        assert!(
            result.is_err(),
            "Invalid query should return error: {}",
            invalid_query
        );
    }
}

#[test]
fn test_very_large_query() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    // Build large dataset
    let mut large_data = String::new();
    for i in 0..1000 {
        large_data.push_str(&format!("<s{}> <p1> <o{}> .\n", i, i));
    }
    graph
        .load_from_turtle(&large_data)
        .expect("Failed to load data");

    // Large query (should still work)
    let query = "SELECT ?s ?o WHERE { ?s <p1> ?o } LIMIT 100";
    let result = execute_select(&graph, query);

    assert!(result.is_ok(), "Large query should succeed");

    let select_result = result.unwrap();
    assert_eq!(
        select_result.bindings.len(),
        100,
        "Should return 100 results"
    );
}

#[test]
fn test_concurrent_query_execution() {
    let graph = Arc::new(WarmPathGraph::new().expect("Failed to create graph"));

    graph
        .load_from_turtle("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    let query = "SELECT ?s WHERE { ?s <p1> ?o }";

    // Execute queries concurrently from multiple threads
    let handles: Vec<_> = (0..20)
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

        let select_result = result.unwrap();
        assert_eq!(select_result.bindings.len(), 1, "Should return 1 binding");
    }
}

#[test]
fn test_concurrent_data_modification() {
    let graph = Arc::new(WarmPathGraph::new().expect("Failed to create graph"));

    graph
        .load_from_turtle("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    let query = "SELECT ?s WHERE { ?s <p1> ?o }";

    // Spawn threads that modify data and query concurrently
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let graph_clone: Arc<WarmPathGraph> = Arc::clone(&graph);
            let query_clone = query.to_string();
            thread::spawn(move || {
                // Insert triple
                graph_clone
                    .insert_triple(&format!("<s{}>", i + 10), "<p1>", &format!("<o{}>", i + 10))
                    .expect("Failed to insert");

                // Execute query
                execute_select(&*graph_clone, &query_clone)
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        let result = handle.join().expect("Thread should complete");
        assert!(
            result.is_ok(),
            "Query should succeed even with concurrent modifications"
        );
    }

    // Verify final state
    assert!(graph.size() >= 11, "Graph should have at least 11 triples");
}

#[test]
fn test_cache_eviction_behavior() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    graph
        .load_from_turtle("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    // Execute many different queries to fill cache
    // Cache size is 1000, so we'll execute 1001 queries to trigger eviction
    for i in 0..1001 {
        let query = format!("SELECT ?s WHERE {{ ?s <p{}> ?o }}", i);
        let _ = execute_select(&graph, &query);
    }

    // Verify cache is still functional
    let query = "SELECT ?s WHERE { ?s <p1> ?o }";
    let result = execute_select(&graph, query);

    assert!(result.is_ok(), "Cache should still work after eviction");

    // Execute first query again - should be a cache miss (was evicted)
    let first_query = "SELECT ?s WHERE { ?s <p0> ?o }";
    let result = execute_select(&graph, first_query);
    assert!(
        result.is_ok(),
        "Query should succeed even if evicted from cache"
    );
}

#[test]
fn test_query_plan_cache_behavior() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    graph
        .load_from_turtle("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    let query = "SELECT ?s WHERE { ?s <p1> ?o }";

    // First execution - should parse and cache query plan
    let start1 = std::time::Instant::now();
    let _result1 = execute_select(&graph, query).expect("Query should succeed");
    let duration1 = start1.elapsed();

    // Second execution - should use cached query plan
    let start2 = std::time::Instant::now();
    let _result2 = execute_select(&graph, query).expect("Query should succeed");
    let duration2 = start2.elapsed();

    // Second execution should be faster (cached query plan)
    assert!(
        duration2 <= duration1 + std::time::Duration::from_millis(10),
        "Cached query plan should be faster or similar: {:?} vs {:?}",
        duration1,
        duration2
    );
}

#[test]
fn test_large_result_set() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    // Build large dataset
    let mut large_data = String::new();
    for i in 0..5000 {
        large_data.push_str(&format!("<s{}> <p1> <o{}> .\n", i, i));
    }
    graph
        .load_from_turtle(&large_data)
        .expect("Failed to load data");

    // Query that returns large result set
    let query = "SELECT ?s ?o WHERE { ?s <p1> ?o }";
    let result = execute_select(&graph, query);

    assert!(result.is_ok(), "Large result set query should succeed");

    let select_result = result.unwrap();
    assert_eq!(
        select_result.bindings.len(),
        5000,
        "Should return all 5000 bindings"
    );
}

#[test]
fn test_query_with_prefixes() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    graph
        .load_from_turtle(
            r#"
        @prefix ex: <http://example.org/> .
        ex:s1 ex:p1 ex:o1 .
    "#,
        )
        .expect("Failed to load data");

    // Query with prefix
    let query = r#"
        PREFIX ex: <http://example.org/>
        SELECT ?s WHERE { ?s ex:p1 ?o }
    "#;
    let result = execute_select(&graph, query);

    assert!(result.is_ok(), "Query with prefix should succeed");

    let select_result = result.unwrap();
    assert_eq!(select_result.bindings.len(), 1, "Should return 1 binding");
}

#[test]
fn test_multiple_concurrent_graphs() {
    // Test that multiple graphs can be used concurrently
    let graph1 = Arc::new(WarmPathGraph::new().expect("Failed to create graph"));
    let graph2 = Arc::new(WarmPathGraph::new().expect("Failed to create graph"));

    graph1
        .load_from_turtle("<s1> <p1> <o1> .")
        .expect("Failed to load");
    graph2
        .load_from_turtle("<s2> <p1> <o2> .")
        .expect("Failed to load");

    let query = "SELECT ?s WHERE { ?s <p1> ?o }";

    let handle1 = thread::spawn({
        let graph = Arc::clone(&graph1);
        let query = query.to_string();
        move || execute_select(&*graph, &query)
    });

    let handle2 = thread::spawn({
        let graph = Arc::clone(&graph2);
        let query = query.to_string();
        move || execute_select(&*graph, &query)
    });

    let result1 = handle1.join().expect("Thread 1 should complete");
    let result2 = handle2.join().expect("Thread 2 should complete");

    assert!(result1.is_ok(), "Graph 1 query should succeed");
    assert!(result2.is_ok(), "Graph 2 query should succeed");

    let select1 = result1.unwrap();
    let select2 = result2.unwrap();

    assert_eq!(select1.bindings.len(), 1, "Graph 1 should return 1 binding");
    assert_eq!(select2.bindings.len(), 1, "Graph 2 should return 1 binding");

    // Bindings should be different (different graphs)
    assert_ne!(
        select1.bindings, select2.bindings,
        "Graphs should have different data"
    );
}

#[test]
fn test_query_performance_under_load() {
    let graph = Arc::new(WarmPathGraph::new().expect("Failed to create graph"));

    // Load moderate dataset
    let mut data = String::new();
    for i in 0..100 {
        data.push_str(&format!("<s{}> <p1> <o{}> .\n", i, i));
    }
    graph.load_from_turtle(&data).expect("Failed to load data");

    let query = "SELECT ?s WHERE { ?s <p1> ?o }";

    // Execute many queries concurrently
    let start = std::time::Instant::now();
    let handles: Vec<_> = (0..100)
        .map(|_| {
            let graph_clone = Arc::clone(&graph);
            let query_clone = query.to_string();
            thread::spawn(move || execute_select(&*graph_clone, &query_clone))
        })
        .collect();

    // Wait for all queries
    for handle in handles {
        let result = handle.join().expect("Thread should complete");
        assert!(result.is_ok(), "Query should succeed under load");
    }

    let duration = start.elapsed();

    // All queries should complete within reasonable time
    assert!(
        duration.as_millis() < 5000,
        "100 concurrent queries should complete in <5s: {}ms",
        duration.as_millis()
    );
}

#[test]
fn test_cache_consistency_under_concurrent_load() {
    let graph = Arc::new(WarmPathGraph::new().expect("Failed to create graph"));

    graph
        .load_from_turtle("<s1> <p1> <o1> .")
        .expect("Failed to load data");

    let query = "SELECT ?s WHERE { ?s <p1> ?o }";

    // Execute queries concurrently
    let handles: Vec<_> = (0..50)
        .map(|_| {
            let graph_clone = Arc::clone(&graph);
            let query_clone = query.to_string();
            thread::spawn(move || execute_select(&*graph_clone, &query_clone))
        })
        .collect();

    // Collect results
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.join().expect("Thread should complete");
        assert!(result.is_ok(), "Query should succeed");
        results.push(result.unwrap());
    }

    // All results should be consistent
    for result in results {
        assert_eq!(
            result.bindings.len(),
            1,
            "All queries should return same result"
        );
    }

    // Check metrics
    let metrics = graph.get_metrics();
    assert!(
        metrics.total_queries >= 50,
        "Should have executed at least 50 queries"
    );
}
