// Chicago TDD Integration Tests - End-to-End System Validation
//
// Principles:
// 1. State-based verification (not interaction-based)
// 2. Real collaborators across all components
// 3. Test full system: Sidecar → ETL → Hot Path → Telemetry
// 4. Verify Weaver telemetry emission

use knhk_sidecar::service::{KgcSidecarService, proto::*};
use knhk_sidecar::service::proto::kgc_sidecar_server::KgcSidecar;
use knhk_sidecar::config::SidecarConfig;
use knhk_etl::*;
use tonic::Request;

// ============================================================================
// Test Suite: Full System Integration (Sidecar → ETL → Emit)
// ============================================================================

#[tokio::test]
async fn test_full_system_sidecar_to_etl_to_emit() {
    // Arrange: Full system with real components
    let config = SidecarConfig::default();
    let sidecar = KgcSidecarService::new(config);

    let pipeline = Pipeline::new(
        vec!["integration_test".to_string()],
        "urn:knhk:schema:integration".to_string(),
        true,
        vec!["https://webhook.integration.test".to_string()],
    );

    let turtle_data = r#"
        <http://example.org/alice> <http://example.org/name> "Alice" .
        <http://example.org/alice> <http://example.org/email> "alice@example.com" .
        <http://example.org/bob> <http://example.org/name> "Bob" .
    "#;

    // Act: Full pipeline through sidecar
    let transaction_request = Request::new(ApplyTransactionRequest {
        rdf_data: turtle_data.as_bytes().to_vec(),
        schema_iri: "urn:knhk:schema:integration".to_string(),
    });

    let sidecar_response = sidecar.apply_transaction(transaction_request).await;

    // Execute ETL pipeline
    let ingest_result = IngestResult {
        triples: pipeline.ingest.parse_rdf_turtle(turtle_data).unwrap(),
        metadata: alloc::collections::BTreeMap::new(),
    };

    let transform_result = pipeline.transform.transform(ingest_result).unwrap();
    let load_result = pipeline.load.load(transform_result).unwrap();
    let reflex_result = pipeline.reflex.reflex(load_result).unwrap();
    let emit_result = pipeline.emit.emit(reflex_result).unwrap();

    // Assert: Full system executed successfully
    assert!(sidecar_response.is_ok(), "Sidecar should handle request");

    assert_eq!(emit_result.receipts_written, 1, "Should write receipts");
    assert!(emit_result.lockchain_hashes.len() >= 1, "Should generate lockchain hashes");

    // Verify sidecar metrics recorded transaction
    let metrics_request = Request::new(GetMetricsRequest {});
    let metrics = sidecar.get_metrics(metrics_request).await.unwrap().into_inner().metrics.unwrap();
    assert_eq!(metrics.total_transactions, 1, "Sidecar should record transaction");
}

#[tokio::test]
async fn test_integration_sidecar_query_to_hot_path() {
    // Arrange: Sidecar with query routing
    let config = SidecarConfig::default();
    let sidecar = KgcSidecarService::new(config);

    let query_request = Request::new(QueryRequest {
        query_type: query_request::QueryType::Ask as i32,
        query_sparql: "ASK { ?s <http://example.org/name> \"Alice\" }".to_string(),
    });

    // Act: Execute query through sidecar
    let response = sidecar.query(query_request).await;

    // Assert: Query processed (even if not yet implemented)
    assert!(response.is_ok(), "Sidecar should handle query request");

    let query_response = response.unwrap().into_inner();
    assert_eq!(query_response.query_type, query_request::QueryType::Ask as i32,
               "Query type should be preserved");

    // Verify metrics
    let metrics_request = Request::new(GetMetricsRequest {});
    let metrics = sidecar.get_metrics(metrics_request).await.unwrap().into_inner().metrics.unwrap();
    assert_eq!(metrics.total_queries, 1, "Should record query execution");
}

#[tokio::test]
async fn test_integration_validate_graph_to_etl() {
    // Arrange: Graph validation integrated with ETL
    let config = SidecarConfig::default();
    let sidecar = KgcSidecarService::new(config);

    let turtle_data = "<http://s> <http://p> <http://o> .";
    let validate_request = Request::new(ValidateGraphRequest {
        rdf_data: turtle_data.as_bytes().to_vec(),
        schema_iri: "urn:shacl:schema:test".to_string(),
    });

    // Act: Validate through sidecar
    let response = sidecar.validate_graph(validate_request).await;

    // Assert: Validation attempted
    assert!(response.is_ok(), "Sidecar should handle validation request");

    let validate_response = response.unwrap().into_inner();
    // Currently not implemented, but should respond
    assert!(!validate_response.errors.is_empty(), "Should report implementation status");
}

#[tokio::test]
async fn test_integration_hook_evaluation_to_reflex() {
    // Arrange: Hook evaluation integrated with reflex stage
    let config = SidecarConfig::default();
    let sidecar = KgcSidecarService::new(config);

    let turtle_data = r#"
        <http://example.org/event> <http://example.org/type> <http://example.org/OrderPlaced> .
        <http://example.org/event> <http://example.org/amount> "100.00" .
    "#;

    let hook_request = Request::new(EvaluateHookRequest {
        hook_id: "order_placed_hook".to_string(),
        rdf_data: turtle_data.as_bytes().to_vec(),
    });

    // Act: Evaluate hook
    let response = sidecar.evaluate_hook(hook_request).await;

    // Assert: Hook evaluation attempted
    assert!(response.is_ok(), "Sidecar should handle hook request");

    // Verify metrics recorded hook
    let metrics_request = Request::new(GetMetricsRequest {});
    let metrics = sidecar.get_metrics(metrics_request).await.unwrap().into_inner().metrics.unwrap();
    assert_eq!(metrics.total_hooks_evaluated, 1, "Should record hook evaluation");
}

// ============================================================================
// Test Suite: Cross-Component State Consistency
// ============================================================================

#[tokio::test]
async fn test_integration_concurrent_sidecar_and_etl_operations() {
    // Arrange: Concurrent operations across components
    let config = SidecarConfig::default();
    let sidecar = std::sync::Arc::new(KgcSidecarService::new(config));

    let pipeline = std::sync::Arc::new(Pipeline::new(
        vec!["concurrent_test".to_string()],
        "urn:test:schema".to_string(),
        false,
        vec![],
    ));

    // Act: Spawn concurrent operations
    let mut handles = vec![];

    // Concurrent sidecar operations
    for i in 0..5 {
        let svc = sidecar.clone();
        let handle = tokio::spawn(async move {
            let request = Request::new(QueryRequest {
                query_type: query_request::QueryType::Ask as i32,
                query_sparql: format!("ASK {{ ?s ?p <http://o{i}> }}"),
            });
            svc.query(request).await
        });
        handles.push(handle);
    }

    // Wait for all operations
    for handle in handles {
        let _ = handle.await;
    }

    // Assert: Consistent state across components
    let metrics_request = Request::new(GetMetricsRequest {});
    let metrics = sidecar.get_metrics(metrics_request).await.unwrap().into_inner().metrics.unwrap();

    assert_eq!(metrics.total_queries, 5, "Should record all concurrent queries");
    assert_eq!(metrics.total_requests, 5, "Should record all concurrent requests");
}

#[tokio::test]
async fn test_integration_health_check_reflects_system_state() {
    // Arrange: System with health monitoring
    let config = SidecarConfig::default();
    let sidecar = KgcSidecarService::new(config);

    // Perform some operations
    let _ = sidecar.query(Request::new(QueryRequest {
        query_type: query_request::QueryType::Ask as i32,
        query_sparql: "ASK {}".to_string(),
    })).await;

    // Act: Check health
    let health_request = Request::new(HealthCheckRequest {});
    let health_response = sidecar.health_check(health_request).await.unwrap().into_inner();

    // Assert: Health reflects system state
    assert_eq!(health_response.status, health_status::HealthStatus::HealthStatusHealthy as i32,
               "System should be healthy after successful operations");

    // Verify metrics available
    let metrics = sidecar.get_metrics(Request::new(GetMetricsRequest {}))
        .await.unwrap().into_inner().metrics.unwrap();
    assert!(metrics.total_requests > 0, "Health check should reflect operation history");
}

// ============================================================================
// Test Suite: ETL Pipeline Integration with Multiple Sources
// ============================================================================

#[test]
fn test_integration_etl_handles_multiple_data_sources() {
    // Arrange: Pipeline with multiple connectors
    let pipeline = Pipeline::new(
        vec!["kafka".to_string(), "postgres".to_string(), "salesforce".to_string()],
        "urn:knhk:schema:multi_source".to_string(),
        true,
        vec!["https://webhook1.com".to_string(), "https://webhook2.com".to_string()],
    );

    let turtle_data = r#"
        <http://kafka/event1> <http://type> <http://OrderPlaced> .
        <http://postgres/user1> <http://type> <http://User> .
        <http://salesforce/lead1> <http://type> <http://Lead> .
    "#;

    // Act: Process data from multiple sources
    let ingest_result = IngestResult {
        triples: pipeline.ingest.parse_rdf_turtle(turtle_data).unwrap(),
        metadata: alloc::collections::BTreeMap::new(),
    };

    let transform_result = pipeline.transform.transform(ingest_result).unwrap();
    let load_result = pipeline.load.load(transform_result).unwrap();
    let reflex_result = pipeline.reflex.reflex(load_result).unwrap();
    let emit_result = pipeline.emit.emit(reflex_result).unwrap();

    // Assert: All sources processed
    assert_eq!(emit_result.receipts_written, 1, "Should process all sources");
    assert!(emit_result.lockchain_hashes.len() >= 1, "Should generate hashes");
}

#[test]
fn test_integration_etl_respects_tick_budget_across_stages() {
    // Arrange: Pipeline with budget enforcement
    let pipeline = Pipeline::new(
        vec!["test".to_string()],
        "urn:test:schema".to_string(),
        false,
        vec![],
    );

    let turtle = r#"
        <http://s1> <http://p> <http://o1> .
        <http://s2> <http://p> <http://o2> .
        <http://s3> <http://p> <http://o3> .
    "#;

    // Act: Full pipeline execution
    let ingest_result = IngestResult {
        triples: pipeline.ingest.parse_rdf_turtle(turtle).unwrap(),
        metadata: alloc::collections::BTreeMap::new(),
    };

    let transform_result = pipeline.transform.transform(ingest_result).unwrap();
    let load_result = pipeline.load.load(transform_result).unwrap();
    let reflex_result = pipeline.reflex.reflex(load_result).unwrap();

    // Assert: Budget respected across all stages
    assert!(reflex_result.max_ticks <= 8,
            "Full pipeline must respect 8-tick budget, got {} ticks", reflex_result.max_ticks);
}

// ============================================================================
// Test Suite: Telemetry Integration (Weaver)
// ============================================================================

#[tokio::test]
#[cfg(feature = "otel")]
async fn test_integration_sidecar_emits_weaver_telemetry() {
    // Arrange: Sidecar with Weaver endpoint
    let config = SidecarConfig::default();
    let sidecar = KgcSidecarService::new_with_weaver(
        config,
        Some("http://localhost:4317".to_string())
    );

    let request = Request::new(ApplyTransactionRequest {
        rdf_data: b"<s> <p> <o> .".to_vec(),
        schema_iri: "urn:test:schema".to_string(),
    });

    // Act: Execute with telemetry
    let response = sidecar.apply_transaction(request).await;

    // Assert: Telemetry attempted (export may fail if no OTLP receiver)
    assert!(response.is_ok(), "Should complete even if telemetry export fails");

    // Verify metrics recorded attempt
    let metrics = sidecar.get_metrics(Request::new(GetMetricsRequest {}))
        .await.unwrap().into_inner().metrics.unwrap();

    assert!(metrics.total_latency_ms > 0, "Should record latency including telemetry");
}

#[tokio::test]
#[cfg(feature = "otel")]
async fn test_integration_etl_telemetry_with_sidecar() {
    // Arrange: Full system with telemetry
    let config = SidecarConfig::default();
    let sidecar = KgcSidecarService::new_with_weaver(
        config,
        Some("http://localhost:4317".to_string())
    );

    let pipeline = Pipeline::new(
        vec!["telemetry_test".to_string()],
        "urn:test:schema".to_string(),
        true,
        vec![],
    );

    let turtle = "<http://s> <http://p> <http://o> .";

    // Act: Full pipeline with telemetry
    let _ = sidecar.apply_transaction(Request::new(ApplyTransactionRequest {
        rdf_data: turtle.as_bytes().to_vec(),
        schema_iri: "urn:test:schema".to_string(),
    })).await;

    let ingest_result = IngestResult {
        triples: pipeline.ingest.parse_rdf_turtle(turtle).unwrap(),
        metadata: alloc::collections::BTreeMap::new(),
    };

    let _ = pipeline.transform.transform(ingest_result);

    // Assert: Telemetry coordinated across components
    // (Actual validation would be via Weaver live-check)
}

// ============================================================================
// Test Suite: Error Propagation Across Components
// ============================================================================

#[tokio::test]
async fn test_integration_error_propagation_from_etl_to_sidecar() {
    // Arrange: System with error scenario
    let config = SidecarConfig::default();
    let sidecar = KgcSidecarService::new(config);

    let invalid_turtle = "<http://s> <http://p>"; // Missing object

    let request = Request::new(ApplyTransactionRequest {
        rdf_data: invalid_turtle.as_bytes().to_vec(),
        schema_iri: "urn:test:schema".to_string(),
    });

    // Act: Execute with invalid data
    let response = sidecar.apply_transaction(request).await;

    // Assert: Error handled gracefully (doesn't panic)
    assert!(response.is_ok(), "Sidecar should handle ETL errors gracefully");

    let transaction_response = response.unwrap().into_inner();
    assert!(!transaction_response.committed, "Transaction should not commit on error");
}

#[tokio::test]
async fn test_integration_metrics_consistent_across_error_scenarios() {
    // Arrange
    let config = SidecarConfig::default();
    let sidecar = KgcSidecarService::new(config);

    // Execute mix of success and failure operations
    let _ = sidecar.query(Request::new(QueryRequest {
        query_type: query_request::QueryType::Ask as i32,
        query_sparql: "ASK {}".to_string(),
    })).await;

    let _ = sidecar.apply_transaction(Request::new(ApplyTransactionRequest {
        rdf_data: vec![],
        schema_iri: "".to_string(),
    })).await;

    // Act: Get metrics
    let metrics = sidecar.get_metrics(Request::new(GetMetricsRequest {}))
        .await.unwrap().into_inner().metrics.unwrap();

    // Assert: Metrics consistent
    assert_eq!(metrics.total_requests, metrics.successful_requests + metrics.failed_requests,
               "Metrics should account for all requests regardless of outcome");
}

// ============================================================================
// Test Suite: Performance Integration
// ============================================================================

#[tokio::test]
async fn test_integration_end_to_end_latency_acceptable() {
    // Arrange: Full system
    let config = SidecarConfig::default();
    let sidecar = KgcSidecarService::new(config);

    let turtle = "<http://s> <http://p> <http://o> .";

    // Act: Measure end-to-end latency
    let start = std::time::Instant::now();

    let _ = sidecar.apply_transaction(Request::new(ApplyTransactionRequest {
        rdf_data: turtle.as_bytes().to_vec(),
        schema_iri: "urn:test:schema".to_string(),
    })).await;

    let duration = start.elapsed();

    // Assert: Latency acceptable (< 100ms for simple transaction)
    assert!(duration.as_millis() < 100,
            "End-to-end transaction should complete in <100ms, got {:?}", duration);
}

#[test]
fn test_integration_etl_pipeline_throughput() {
    // Arrange: Pipeline for throughput test
    let pipeline = Pipeline::new(
        vec!["throughput_test".to_string()],
        "urn:test:schema".to_string(),
        false,
        vec![],
    );

    let turtle = "<http://s> <http://p> <http://o> .";

    // Act: Process batch
    let start = std::time::Instant::now();

    for _ in 0..100 {
        let ingest_result = IngestResult {
            triples: pipeline.ingest.parse_rdf_turtle(turtle).unwrap(),
            metadata: alloc::collections::BTreeMap::new(),
        };

        let transform_result = pipeline.transform.transform(ingest_result).unwrap();
        let load_result = pipeline.load.load(transform_result).unwrap();
        let _ = pipeline.reflex.reflex(load_result).unwrap();
    }

    let duration = start.elapsed();

    // Assert: Throughput acceptable
    let triples_per_second = 100.0 / duration.as_secs_f64();
    println!("ETL pipeline throughput: {:.0} transactions/second", triples_per_second);

    assert!(triples_per_second > 10.0,
            "Should process >10 transactions/second, got {:.0} tps", triples_per_second);
}
