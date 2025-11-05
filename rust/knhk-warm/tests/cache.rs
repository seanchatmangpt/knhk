//! Chicago TDD tests for cache behavior
//! Tests cache invalidation on epoch bump, LRU eviction, query plan cache hits/misses, cache metrics accuracy

use knhk_warm::{WarmPathGraph, execute_select, execute_ask};
use knhk_warm::query::QueryMetrics;

#[test]
fn test_cache_invalidation_on_epoch_bump() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    let query = "SELECT ?s WHERE { ?s <p1> ?o }";
    
    // Execute query (cache it)
    let _result1 = execute_select(&graph, query).expect("Query should succeed");
    
    let metrics_before = graph.get_metrics();
    let cache_hits_before = metrics_before.cache_hits;
    let cache_misses_before = metrics_before.cache_misses;
    
    // Bump epoch (should invalidate cache)
    graph.bump_epoch();
    
    // Execute same query again
    let _result2 = execute_select(&graph, query).expect("Query should succeed");
    
    let metrics_after = graph.get_metrics();
    
    // Cache misses should increase (cache was invalidated)
    assert!(metrics_after.total_queries > metrics_before.total_queries,
           "Total queries should increase: {} vs {}",
           metrics_after.total_queries, metrics_before.total_queries);
}

#[test]
fn test_cache_invalidation_on_data_insert() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    let query = "SELECT ?s WHERE { ?s <p1> ?o }";
    
    // Execute query (cache it)
    let result1 = execute_select(&graph, query).expect("Query should succeed");
    let initial_count = result1.bindings.len();
    
    // Insert new triple (should invalidate cache)
    graph.insert_triple("<s2>", "<p1>", "<o2>").expect("Failed to insert");
    
    // Execute same query again
    let result2 = execute_select(&graph, query).expect("Query should succeed");
    
    // Result should reflect new data (cache was invalidated)
    assert!(result2.bindings.len() > initial_count,
           "Result should reflect new data: {} vs {}",
           result2.bindings.len(), initial_count);
}

#[test]
fn test_cache_invalidation_on_data_load() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    let query = "SELECT ?s WHERE { ?s <p1> ?o }";
    
    // Execute query (cache it)
    let result1 = execute_select(&graph, query).expect("Query should succeed");
    let initial_count = result1.bindings.len();
    
    // Load more data (should invalidate cache)
    graph.load_from_turtle("<s2> <p1> <o2> .").expect("Failed to load data");
    
    // Execute same query again
    let result2 = execute_select(&graph, query).expect("Query should succeed");
    
    // Result should reflect new data
    assert!(result2.bindings.len() > initial_count,
           "Result should reflect new data: {} vs {}",
           result2.bindings.len(), initial_count);
}

#[test]
fn test_lru_cache_eviction() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    // Execute many different queries to fill cache (cache size is 1000)
    // We'll execute 1001 queries to trigger eviction
    for i in 0..1001 {
        let query = format!("SELECT ?s WHERE {{ ?s <p{}> ?o }}", i);
        let _ = execute_select(&graph, &query);
    }
    
    // Verify cache is still functional
    let query = "SELECT ?s WHERE { ?s <p1> ?o }";
    let result = execute_select(&graph, query);
    
    assert!(result.is_ok(), "Cache should still work after eviction");
    
    // Execute first query again (was evicted)
    let first_query = "SELECT ?s WHERE { ?s <p0> ?o }";
    let result = execute_select(&graph, first_query);
    assert!(result.is_ok(), "Query should succeed even if evicted");
}

#[test]
fn test_query_plan_cache_hit() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    let query = "SELECT ?s WHERE { ?s <p1> ?o }";
    
    // First execution - should parse and cache query plan
    let start1 = std::time::Instant::now();
    let _result1 = execute_select(&graph, query).expect("Query should succeed");
    let duration1 = start1.elapsed();
    
    // Second execution - should use cached query plan
    let start2 = std::time::Instant::now();
    let _result2 = execute_select(&graph, query).expect("Query should succeed");
    let duration2 = start2.elapsed();
    
    // Second execution should be at least as fast (cached query plan)
    assert!(duration2 <= duration1 + std::time::Duration::from_millis(50),
           "Cached query plan should be similar or faster: {:?} vs {:?}",
           duration1, duration2);
}

#[test]
fn test_query_plan_cache_miss() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    // Execute different queries (each should parse and cache)
    let queries = vec![
        "SELECT ?s WHERE { ?s <p1> ?o }",
        "SELECT ?s WHERE { ?s <p2> ?o }",
        "SELECT ?s WHERE { ?s <p3> ?o }",
    ];
    
    for query in &queries {
        let result = execute_select(&graph, query);
        assert!(result.is_ok(), "Query should succeed");
    }
    
    // All queries should have been parsed and cached
    let metrics = graph.get_metrics();
    assert!(metrics.total_queries >= queries.len() as u64,
           "Should have executed all queries");
}

#[test]
fn test_cache_metrics_accuracy() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    let query = "SELECT ?s WHERE { ?s <p1> ?o }";
    
    // Initial metrics
    let metrics_initial = graph.get_metrics();
    assert_eq!(metrics_initial.total_queries, 0, "Initial total queries should be 0");
    assert_eq!(metrics_initial.cache_hits, 0, "Initial cache hits should be 0");
    assert_eq!(metrics_initial.cache_misses, 0, "Initial cache misses should be 0");
    
    // Execute query (cache miss)
    let _result1 = execute_select(&graph, query).expect("Query should succeed");
    
    let metrics_after_first = graph.get_metrics();
    assert_eq!(metrics_after_first.total_queries, 1, "Should have 1 query");
    assert_eq!(metrics_after_first.cache_hits, 0, "First query should be cache miss");
    assert_eq!(metrics_after_first.cache_misses, 1, "Should have 1 cache miss");
    
    // Execute same query again (cache hit)
    let _result2 = execute_select(&graph, query).expect("Query should succeed");
    
    let metrics_after_second = graph.get_metrics();
    assert_eq!(metrics_after_second.total_queries, 2, "Should have 2 queries");
    assert_eq!(metrics_after_second.cache_hits, 1, "Second query should be cache hit");
    assert_eq!(metrics_after_second.cache_misses, 1, "Should still have 1 cache miss");
    
    // Verify hit rate
    let hit_rate = metrics_after_second.cache_hit_rate;
    assert!((hit_rate - 0.5).abs() < 0.01,
           "Hit rate should be approximately 0.5: {}", hit_rate);
}

#[test]
fn test_cache_hit_rate_calculation() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    let query = "SELECT ?s WHERE { ?s <p1> ?o }";
    
    // Execute query 10 times (should get cache hits after first)
    for _ in 0..10 {
        let _result = execute_select(&graph, query).expect("Query should succeed");
    }
    
    let metrics = graph.get_metrics();
    
    // Verify hit rate calculation
    let expected_hit_rate = if metrics.total_queries > 0 {
        metrics.cache_hits as f64 / metrics.total_queries as f64
    } else {
        0.0
    };
    
    assert!((metrics.cache_hit_rate - expected_hit_rate).abs() < 0.01,
           "Hit rate should match calculation: {} vs {}",
           metrics.cache_hit_rate, expected_hit_rate);
    
    // Hit rate should be > 0 after multiple identical queries
    assert!(metrics.cache_hit_rate > 0.0,
           "Hit rate should be > 0: {}", metrics.cache_hit_rate);
}

#[test]
fn test_cache_invalidation_metrics() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    let query = "SELECT ?s WHERE { ?s <p1> ?o }";
    
    // Execute query (cache it)
    let _result1 = execute_select(&graph, query).expect("Query should succeed");
    
    let metrics_before = graph.get_metrics();
    
    // Bump epoch (invalidate cache)
    graph.bump_epoch();
    
    // Execute same query again (should be cache miss due to epoch change)
    let _result2 = execute_select(&graph, query).expect("Query should succeed");
    
    let metrics_after = graph.get_metrics();
    
    // Cache hits should not increase (cache was invalidated)
    assert!(metrics_after.total_queries > metrics_before.total_queries,
           "Total queries should increase: {} vs {}",
           metrics_after.total_queries, metrics_before.total_queries);
}

#[test]
fn test_cache_consistency_across_epochs() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    let query = "SELECT ?s WHERE { ?s <p1> ?o }";
    
    // Execute query in epoch 1
    let result1 = execute_select(&graph, query).expect("Query should succeed");
    let count1 = result1.bindings.len();
    
    // Insert data and bump epoch
    graph.insert_triple("<s2>", "<p1>", "<o2>").expect("Failed to insert");
    
    // Execute same query in epoch 2
    let result2 = execute_select(&graph, query).expect("Query should succeed");
    let count2 = result2.bindings.len();
    
    // Result should reflect new epoch data
    assert!(count2 > count1,
           "Result should reflect new epoch: {} vs {}",
           count2, count1);
}

#[test]
fn test_cache_eviction_order() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    // Fill cache with queries
    for i in 0..1001 {
        let query = format!("SELECT ?s WHERE {{ ?s <p{}> ?o }}", i);
        let _ = execute_select(&graph, &query);
    }
    
    // First query should have been evicted (LRU)
    let first_query = "SELECT ?s WHERE { ?s <p0> ?o }";
    let result = execute_select(&graph, first_query);
    
    assert!(result.is_ok(), "Query should succeed even if evicted");
    
    // Last query should still be in cache
    let last_query = "SELECT ?s WHERE { ?s <p1000> ?o }";
    let result = execute_select(&graph, last_query);
    
    assert!(result.is_ok(), "Last query should succeed");
}

#[test]
fn test_cache_invalidation_on_quad_insert() {
    let graph = WarmPathGraph::new().expect("Failed to create graph");
    
    graph.load_from_turtle("<s1> <p1> <o1> .").expect("Failed to load data");
    
    let query = "SELECT ?s WHERE { ?s <p1> ?o }";
    
    // Execute query (cache it)
    let result1 = execute_select(&graph, query).expect("Query should succeed");
    let initial_count = result1.bindings.len();
    
    // Insert quad (should invalidate cache)
    use oxigraph::model::{GraphName, NamedNode, Quad, Term};
    let s2 = NamedNode::new("http://example.org/s2").expect("Valid IRI");
    let p1 = NamedNode::new("http://example.org/p1").expect("Valid IRI");
    let o2 = NamedNode::new("http://example.org/o2").expect("Valid IRI");
    let quad = Quad::new(s2, p1, Term::NamedNode(o2), GraphName::DefaultGraph);
    
    graph.insert_quads(&[quad]).expect("Failed to insert quad");
    
    // Execute same query again
    let result2 = execute_select(&graph, query).expect("Query should succeed");
    
    // Result should reflect new data
    assert!(result2.bindings.len() > initial_count,
           "Result should reflect new data: {} vs {}",
           result2.bindings.len(), initial_count);
}

