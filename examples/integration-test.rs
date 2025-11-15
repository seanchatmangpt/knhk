// Integration Test Example
// Demonstrates full-feature integration testing
//
// Key Concepts:
// - Multiple components working together
// - Real async/await execution
// - Telemetry verification
// - End-to-end workflow
// - Resource cleanup

use std::collections::HashMap;
use std::time::Instant;

// ============================================================================
// Components
// ============================================================================

/// Query executor (simulated)
struct QueryExecutor {
    execution_count: usize,
}

impl QueryExecutor {
    fn new() -> Self {
        Self {
            execution_count: 0,
        }
    }

    async fn execute_ask(&mut self, sparql: &str) -> Result<bool, String> {
        // Simulate async execution
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        self.execution_count += 1;

        if sparql.is_empty() {
            return Err("Empty query".to_string());
        }

        Ok(true)
    }

    fn get_execution_count(&self) -> usize {
        self.execution_count
    }
}

/// Cache (simulated)
struct Cache {
    store: HashMap<String, String>,
    hits: usize,
    misses: usize,
}

impl Cache {
    fn new() -> Self {
        Self {
            store: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    fn get(&mut self, key: &str) -> Option<&String> {
        match self.store.get(key) {
            Some(value) => {
                self.hits += 1;
                Some(value)
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }

    fn set(&mut self, key: String, value: String) {
        self.store.insert(key, value);
    }

    fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

/// Telemetry collector (simulated)
struct TelemetryCollector {
    spans: Vec<Span>,
    metrics: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
struct Span {
    name: String,
    duration_ms: u128,
    success: bool,
}

impl TelemetryCollector {
    fn new() -> Self {
        Self {
            spans: Vec::new(),
            metrics: HashMap::new(),
        }
    }

    fn record_span(&mut self, name: String, duration_ms: u128, success: bool) {
        self.spans.push(Span {
            name,
            duration_ms,
            success,
        });
    }

    fn record_metric(&mut self, name: String, value: u64) {
        *self.metrics.entry(name).or_insert(0) += value;
    }

    fn get_span_count(&self) -> usize {
        self.spans.len()
    }

    fn get_successful_spans(&self) -> usize {
        self.spans.iter().filter(|s| s.success).count()
    }

    fn get_metric(&self, name: &str) -> u64 {
        *self.metrics.get(name).unwrap_or(&0)
    }
}

// ============================================================================
// Integrated System
// ============================================================================

struct QueryService {
    executor: QueryExecutor,
    cache: Cache,
    telemetry: TelemetryCollector,
}

impl QueryService {
    fn new() -> Self {
        Self {
            executor: QueryExecutor::new(),
            cache: Cache::new(),
            telemetry: TelemetryCollector::new(),
        }
    }

    async fn execute_query_with_cache(&mut self, query_id: &str, sparql: &str) -> Result<bool, String> {
        let start = Instant::now();

        // Check cache
        if let Some(cached_result) = self.cache.get(query_id) {
            let duration = start.elapsed().as_millis();
            self.telemetry.record_span(
                "query.cached".to_string(),
                duration,
                true,
            );
            self.telemetry.record_metric("cache.hits".to_string(), 1);
            return Ok(cached_result == "true");
        }

        // Cache miss - execute query
        self.telemetry.record_metric("cache.misses".to_string(), 1);

        let result = self.executor.execute_ask(sparql).await;

        let duration = start.elapsed().as_millis();
        let success = result.is_ok();

        self.telemetry.record_span(
            "query.execute".to_string(),
            duration,
            success,
        );

        if let Ok(res) = &result {
            // Cache result
            self.cache.set(query_id.to_string(), res.to_string());
        }

        result
    }

    fn get_stats(&self) -> ServiceStats {
        ServiceStats {
            total_queries: self.executor.get_execution_count(),
            cache_hit_rate: self.cache.hit_rate(),
            total_spans: self.telemetry.get_span_count(),
            successful_spans: self.telemetry.get_successful_spans(),
            cache_hits: self.telemetry.get_metric("cache.hits"),
            cache_misses: self.telemetry.get_metric("cache.misses"),
        }
    }
}

#[derive(Debug)]
struct ServiceStats {
    total_queries: usize,
    cache_hit_rate: f64,
    total_spans: usize,
    successful_spans: usize,
    cache_hits: u64,
    cache_misses: u64,
}

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_query_pipeline() {
        // Arrange: Create service with all components
        let mut service = QueryService::new();

        // Act: Execute query (cache miss)
        let result1 = service
            .execute_query_with_cache("query_1", "ASK { ?s ?p ?o }")
            .await;

        // Assert: Query succeeded
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), true);

        // Assert: Telemetry recorded
        let stats = service.get_stats();
        assert_eq!(stats.total_queries, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.total_spans, 1);
    }

    #[tokio::test]
    async fn test_cache_hit_path() {
        // Arrange: Create service and execute query once
        let mut service = QueryService::new();
        service
            .execute_query_with_cache("query_1", "ASK { ?s ?p ?o }")
            .await
            .expect("First execution");

        // Act: Execute same query again (cache hit)
        let result2 = service
            .execute_query_with_cache("query_1", "ASK { ?s ?p ?o }")
            .await;

        // Assert: Query succeeded without execution
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), true);

        // Assert: Executor called only once (first time)
        let stats = service.get_stats();
        assert_eq!(stats.total_queries, 1, "Executor should be called once");
        assert_eq!(stats.cache_hits, 1, "Should have 1 cache hit");
        assert_eq!(stats.cache_hit_rate, 50.0, "50% hit rate (1 miss, 1 hit)");
    }

    #[tokio::test]
    async fn test_error_handling_integration() {
        // Arrange: Create service
        let mut service = QueryService::new();

        // Act: Execute invalid query (empty)
        let result = service.execute_query_with_cache("query_error", "").await;

        // Assert: Query failed
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Empty query");

        // Assert: Telemetry recorded error
        let stats = service.get_stats();
        assert_eq!(stats.total_spans, 1);
        assert_eq!(stats.successful_spans, 0, "Error span should not be successful");
    }

    #[tokio::test]
    async fn test_concurrent_queries() {
        // Arrange: Create service
        let mut service = QueryService::new();

        // Act: Execute multiple queries concurrently
        let mut handles = vec![];

        for i in 0..10 {
            let query_id = format!("query_{}", i);
            let sparql = format!("ASK {{ ?s{} ?p{} ?o{} }}", i, i, i);

            // Execute sequentially for this example (tokio test doesn't support concurrent mutable access)
            service.execute_query_with_cache(&query_id, &sparql).await.expect("Query execution");
        }

        // Assert: All queries executed
        let stats = service.get_stats();
        assert_eq!(stats.total_queries, 10);
        assert_eq!(stats.cache_misses, 10);
        assert_eq!(stats.total_spans, 10);
    }

    #[tokio::test]
    async fn test_end_to_end_workflow() {
        // Arrange: Create service
        let mut service = QueryService::new();

        // Act: Execute complete workflow
        // Step 1: Execute query A
        let result_a = service
            .execute_query_with_cache("query_a", "ASK { ?user a :Person }")
            .await
            .expect("Query A");

        // Step 2: Execute query B
        let result_b = service
            .execute_query_with_cache("query_b", "ASK { ?user :hasEmail ?email }")
            .await
            .expect("Query B");

        // Step 3: Re-execute query A (cache hit)
        let result_a2 = service
            .execute_query_with_cache("query_a", "ASK { ?user a :Person }")
            .await
            .expect("Query A (cached)");

        // Assert: All queries succeeded
        assert!(result_a);
        assert!(result_b);
        assert!(result_a2);

        // Assert: Service state correct
        let stats = service.get_stats();
        assert_eq!(stats.total_queries, 2, "Executor called twice (A and B)");
        assert_eq!(stats.cache_hits, 1, "One cache hit (A repeated)");
        assert_eq!(stats.cache_misses, 2, "Two cache misses (A and B first time)");
        assert_eq!(stats.cache_hit_rate, 33.33, "33% hit rate (1/3)");
        assert_eq!(stats.total_spans, 3, "Three spans (A, B, A cached)");
        assert_eq!(stats.successful_spans, 3, "All spans successful");
    }
}

// ============================================================================
// Main: Run Integration Tests
// ============================================================================

#[tokio::main]
async fn main() {
    println!("=== Integration Test Example ===\n");

    // Create service
    let mut service = QueryService::new();

    println!("--- Running End-to-End Integration ---");

    // Execute workflow
    println!("Step 1: Execute query A (cache miss)");
    service
        .execute_query_with_cache("query_a", "ASK { ?s ?p ?o }")
        .await
        .expect("Query A");

    println!("Step 2: Execute query B (cache miss)");
    service
        .execute_query_with_cache("query_b", "SELECT ?s WHERE { ?s ?p ?o }")
        .await
        .expect("Query B");

    println!("Step 3: Re-execute query A (cache hit)");
    service
        .execute_query_with_cache("query_a", "ASK { ?s ?p ?o }")
        .await
        .expect("Query A (cached)");

    println!("Step 4: Execute query C (cache miss)");
    service
        .execute_query_with_cache("query_c", "ASK { ?x ?y ?z }")
        .await
        .expect("Query C");

    println!("\n--- Service Statistics ---");
    let stats = service.get_stats();
    println!("Total queries executed: {}", stats.total_queries);
    println!("Cache hits: {}", stats.cache_hits);
    println!("Cache misses: {}", stats.cache_misses);
    println!("Cache hit rate: {:.1}%", stats.cache_hit_rate);
    println!("Total telemetry spans: {}", stats.total_spans);
    println!("Successful spans: {}", stats.successful_spans);
    println!();

    println!("=== Integration Testing Principles ===");
    println!("1. ✅ Multiple components: Query executor + Cache + Telemetry");
    println!("2. ✅ Real async execution: tokio::test for async tests");
    println!("3. ✅ End-to-end workflow: Complete user scenario tested");
    println!("4. ✅ State verification: Check all component states");
    println!("5. ✅ Real collaborators: No mocks for core logic");
    println!("6. ✅ Telemetry validation: Verify observability works");
    println!("7. ✅ Resource cleanup: Tests are independent");
    println!();

    println!("=== Run Tests ===");
    println!("cargo test --test integration_test");
}

// Key Takeaways:
//
// 1. **Multiple Components**: Test integration, not isolation
//    - QueryExecutor + Cache + Telemetry
//    - Real collaborators (no mocks)
//    - Proves components work together
//
// 2. **Async Execution**: Use tokio::test
//    - Real async/await code
//    - Tests async behavior
//    - Production-like execution
//
// 3. **End-to-End Workflow**: Complete scenarios
//    - Execute → Cache → Re-execute
//    - Multi-step workflows
//    - User-facing behavior
//
// 4. **State Verification**: Check all components
//    - Executor call count
//    - Cache hit/miss rates
//    - Telemetry spans recorded
//
// 5. **Telemetry Validation**: Verify observability
//    - Spans recorded for operations
//    - Metrics tracked correctly
//    - Success/failure recorded
//
// 6. **Resource Cleanup**: Independent tests
//    - Each test creates fresh service
//    - No shared state between tests
//    - Deterministic execution
//
// Integration test vs unit test:
// - Unit: Test single component in isolation
// - Integration: Test multiple components together
// - Integration proves real system works
//
// See also:
// - /home/user/knhk/rust/tests/integration_complete.rs
// - /home/user/knhk/docs/reference/cards/TESTING_CHECKLIST.md
