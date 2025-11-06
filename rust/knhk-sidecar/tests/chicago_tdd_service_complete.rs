// Chicago TDD Tests for KGC Sidecar Service - Complete State-Based Validation
//
// Principles:
// 1. State-based verification (not interaction-based)
// 2. Real collaborators (no mocks)
// 3. Test actual behavior using AAA pattern
// 4. Verify telemetry emission and metrics

use knhk_sidecar::config::SidecarConfig;
use knhk_sidecar::service::{KgcSidecarService, proto::*};
use knhk_sidecar::service::proto::kgc_sidecar_server::KgcSidecar;
use tonic::Request;

// ============================================================================
// Test Suite: ApplyTransaction End-to-End
// ============================================================================

#[tokio::test]
async fn test_apply_transaction_records_metrics() {
    // Arrange: Real service with default config
    let config = SidecarConfig::default();
    let service = KgcSidecarService::new(config);

    let request = Request::new(ApplyTransactionRequest {
        rdf_data: vec![],
        schema_iri: "urn:test:schema".to_string(),
    });

    // Act: Call actual method
    let response = service.apply_transaction(request).await;

    // Assert: Verify response structure and metrics updated
    assert!(response.is_ok(), "Service should handle request without panic");
    let response = response.unwrap().into_inner();

    // Verify metrics were recorded
    let metrics_request = Request::new(GetMetricsRequest {});
    let metrics_response = service.get_metrics(metrics_request).await.unwrap().into_inner();
    let metrics = metrics_response.metrics.unwrap();

    assert_eq!(metrics.total_requests, 1, "Should record 1 request");
    assert_eq!(metrics.total_transactions, 1, "Should record 1 transaction");
    assert!(metrics.last_request_time_ms > 0, "Should record timestamp");
}

#[tokio::test]
async fn test_apply_transaction_returns_not_implemented_error() {
    // Arrange: Service with real ETL integration pending
    let config = SidecarConfig::default();
    let service = KgcSidecarService::new(config);

    let turtle_data = "<http://example.org/s> <http://example.org/p> <http://example.org/o> .";
    let request = Request::new(ApplyTransactionRequest {
        rdf_data: turtle_data.as_bytes().to_vec(),
        schema_iri: "urn:test:schema".to_string(),
    });

    // Act: Execute actual transaction
    let response = service.apply_transaction(request).await.unwrap().into_inner();

    // Assert: Currently returns not implemented (as documented)
    assert!(!response.committed, "Transaction should not commit until ETL integrated");
    assert!(!response.errors.is_empty(), "Should report ETL integration pending");
    assert!(response.errors[0].contains("ETL") || response.errors[0].contains("pending"),
            "Error should mention ETL integration status");
}

#[tokio::test]
async fn test_apply_transaction_with_weaver_telemetry() {
    // Arrange: Service with Weaver endpoint configured
    let config = SidecarConfig::default();
    #[cfg(feature = "otel")]
    let service = KgcSidecarService::new_with_weaver(
        config,
        Some("http://localhost:4317".to_string())
    );
    #[cfg(not(feature = "otel"))]
    let service = KgcSidecarService::new(config);

    let request = Request::new(ApplyTransactionRequest {
        rdf_data: b"<s> <p> <o> .".to_vec(),
        schema_iri: "urn:test:schema".to_string(),
    });

    // Act: Execute with telemetry
    let response = service.apply_transaction(request).await;

    // Assert: Request completed and telemetry attempted
    assert!(response.is_ok(), "Should complete request even if telemetry export fails");

    // Verify metrics captured telemetry attempt
    let metrics_req = Request::new(GetMetricsRequest {});
    let metrics = service.get_metrics(metrics_req).await.unwrap().into_inner().metrics.unwrap();
    assert!(metrics.total_latency_ms > 0, "Should record latency for telemetry");
}

// ============================================================================
// Test Suite: Query Operations State-Based
// ============================================================================

#[tokio::test]
async fn test_query_ask_operation_state() {
    // Arrange: Service ready for queries
    let config = SidecarConfig::default();
    let service = KgcSidecarService::new(config);

    let request = Request::new(QueryRequest {
        query_type: query_request::QueryType::Ask as i32,
        query_sparql: "ASK { ?s ?p ?o }".to_string(),
    });

    // Act: Execute query
    let response = service.query(request).await.unwrap().into_inner();

    // Assert: Query attempted and state recorded
    assert_eq!(response.query_type, query_request::QueryType::Ask as i32,
               "Should preserve query type");

    // Verify query metrics updated
    let metrics_req = Request::new(GetMetricsRequest {});
    let metrics = service.get_metrics(metrics_req).await.unwrap().into_inner().metrics.unwrap();
    assert_eq!(metrics.total_queries, 1, "Should record 1 query");
}

#[tokio::test]
async fn test_query_select_operation_state() {
    // Arrange
    let config = SidecarConfig::default();
    let service = KgcSidecarService::new(config);

    let request = Request::new(QueryRequest {
        query_type: query_request::QueryType::Select as i32,
        query_sparql: "SELECT * WHERE { ?s ?p ?o }".to_string(),
    });

    // Act
    let response = service.query(request).await.unwrap().into_inner();

    // Assert: Query type preserved
    assert_eq!(response.query_type, query_request::QueryType::Select as i32);
}

#[tokio::test]
async fn test_query_construct_operation_state() {
    // Arrange
    let config = SidecarConfig::default();
    let service = KgcSidecarService::new(config);

    let request = Request::new(QueryRequest {
        query_type: query_request::QueryType::Construct as i32,
        query_sparql: "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }".to_string(),
    });

    // Act
    let response = service.query(request).await.unwrap().into_inner();

    // Assert
    assert_eq!(response.query_type, query_request::QueryType::Construct as i32);
}

// ============================================================================
// Test Suite: ValidateGraph State-Based
// ============================================================================

#[tokio::test]
async fn test_validate_graph_with_shacl_schema() {
    // Arrange: Real validation request
    let config = SidecarConfig::default();
    let service = KgcSidecarService::new(config);

    let request = Request::new(ValidateGraphRequest {
        rdf_data: b"<s> <p> <o> .".to_vec(),
        schema_iri: "urn:shacl:schema:test".to_string(),
    });

    // Act: Execute validation
    let response = service.validate_graph(request).await.unwrap().into_inner();

    // Assert: Validation attempted (currently returns not implemented)
    assert!(!response.valid, "Validation should indicate not yet implemented");
    assert!(!response.errors.is_empty(), "Should report implementation status");
}

#[tokio::test]
async fn test_validate_graph_with_empty_data() {
    // Arrange: Edge case - empty RDF data
    let config = SidecarConfig::default();
    let service = KgcSidecarService::new(config);

    let request = Request::new(ValidateGraphRequest {
        rdf_data: vec![],
        schema_iri: "urn:shacl:schema:test".to_string(),
    });

    // Act
    let response = service.validate_graph(request).await.unwrap().into_inner();

    // Assert: Service handles gracefully
    assert!(!response.errors.is_empty(), "Should handle empty data gracefully");
}

// ============================================================================
// Test Suite: EvaluateHook State-Based
// ============================================================================

#[tokio::test]
async fn test_evaluate_hook_with_valid_turtle() {
    // Arrange: Hook evaluation request
    let config = SidecarConfig::default();
    let service = KgcSidecarService::new(config);

    let turtle = "<http://ex.org/s> <http://ex.org/p> <http://ex.org/o> .";
    let request = Request::new(EvaluateHookRequest {
        hook_id: "test_hook_001".to_string(),
        rdf_data: turtle.as_bytes().to_vec(),
    });

    // Act: Evaluate hook
    let response = service.evaluate_hook(request).await.unwrap().into_inner();

    // Assert: Hook evaluation attempted
    assert!(!response.errors.is_empty(), "Should report implementation status");

    // Verify metrics recorded hook evaluation
    let metrics_req = Request::new(GetMetricsRequest {});
    let metrics = service.get_metrics(metrics_req).await.unwrap().into_inner().metrics.unwrap();
    assert_eq!(metrics.total_hooks_evaluated, 1, "Should record hook evaluation");
}

#[tokio::test]
async fn test_evaluate_hook_with_invalid_utf8() {
    // Arrange: Invalid UTF-8 data
    let config = SidecarConfig::default();
    let service = KgcSidecarService::new(config);

    let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
    let request = Request::new(EvaluateHookRequest {
        hook_id: "test_hook_002".to_string(),
        rdf_data: invalid_utf8,
    });

    // Act: Should fail gracefully
    let response = service.evaluate_hook(request).await;

    // Assert: Error handled properly
    assert!(response.is_err(), "Should return error for invalid UTF-8");
    let status = response.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument,
               "Should return InvalidArgument status");
}

// ============================================================================
// Test Suite: Health Check State Verification
// ============================================================================

#[tokio::test]
async fn test_health_check_initial_healthy_state() {
    // Arrange: New service instance
    let config = SidecarConfig::default();
    let service = KgcSidecarService::new(config);

    let request = Request::new(HealthCheckRequest {});

    // Act: Check health
    let response = service.health_check(request).await.unwrap().into_inner();

    // Assert: Initially healthy
    assert_eq!(response.status, health_status::HealthStatus::HealthStatusHealthy as i32,
               "Service should start in healthy state");
    assert!(response.message.contains("healthy") || response.message.contains("Healthy"),
            "Health message should indicate healthy state");
    assert!(response.timestamp_ms > 0, "Should include timestamp");
}

#[tokio::test]
async fn test_health_check_returns_current_timestamp() {
    // Arrange
    let config = SidecarConfig::default();
    let service = KgcSidecarService::new(config);

    let before = chrono::Utc::now().timestamp_millis() as u64;

    // Act
    let request = Request::new(HealthCheckRequest {});
    let response = service.health_check(request).await.unwrap().into_inner();

    let after = chrono::Utc::now().timestamp_millis() as u64;

    // Assert: Timestamp within reasonable range
    assert!(response.timestamp_ms >= before && response.timestamp_ms <= after,
            "Timestamp should be current time");
}

// ============================================================================
// Test Suite: GetMetrics State Aggregation
// ============================================================================

#[tokio::test]
async fn test_get_metrics_aggregates_all_operations() {
    // Arrange: Service with multiple operations
    let config = SidecarConfig::default();
    let service = KgcSidecarService::new(config);

    // Perform various operations
    let _ = service.apply_transaction(Request::new(ApplyTransactionRequest {
        rdf_data: vec![],
        schema_iri: "urn:test".to_string(),
    })).await;

    let _ = service.query(Request::new(QueryRequest {
        query_type: query_request::QueryType::Ask as i32,
        query_sparql: "ASK { }".to_string(),
    })).await;

    let _ = service.evaluate_hook(Request::new(EvaluateHookRequest {
        hook_id: "hook1".to_string(),
        rdf_data: b"data".to_vec(),
    })).await;

    // Act: Get aggregated metrics
    let request = Request::new(GetMetricsRequest {});
    let response = service.get_metrics(request).await.unwrap().into_inner();
    let metrics = response.metrics.unwrap();

    // Assert: All operations recorded
    assert_eq!(metrics.total_requests, 3, "Should count all requests");
    assert_eq!(metrics.total_transactions, 1, "Should count transactions");
    assert_eq!(metrics.total_queries, 1, "Should count queries");
    assert_eq!(metrics.total_hooks_evaluated, 1, "Should count hooks");
    assert!(metrics.average_latency_ms >= 0.0, "Should calculate average latency");
}

#[tokio::test]
async fn test_get_metrics_calculates_success_failure_ratio() {
    // Arrange
    let config = SidecarConfig::default();
    let service = KgcSidecarService::new(config);

    // Mix of operations (all currently fail due to not implemented)
    let _ = service.apply_transaction(Request::new(ApplyTransactionRequest {
        rdf_data: vec![],
        schema_iri: "".to_string(),
    })).await;

    let _ = service.query(Request::new(QueryRequest {
        query_type: 0,
        query_sparql: "".to_string(),
    })).await;

    // Act: Get metrics
    let request = Request::new(GetMetricsRequest {});
    let response = service.get_metrics(request).await.unwrap().into_inner();
    let metrics = response.metrics.unwrap();

    // Assert: Success and failure counts recorded
    assert_eq!(metrics.total_requests, metrics.successful_requests + metrics.failed_requests,
               "Total should equal success + failure");
}

// ============================================================================
// Test Suite: Concurrent Operations State Consistency
// ============================================================================

#[tokio::test]
async fn test_concurrent_requests_maintain_consistent_metrics() {
    // Arrange: Service handling concurrent requests
    let config = SidecarConfig::default();
    let service = std::sync::Arc::new(KgcSidecarService::new(config));

    // Act: Spawn concurrent operations
    let mut handles = vec![];
    for i in 0..10 {
        let svc = service.clone();
        let handle = tokio::spawn(async move {
            let request = Request::new(QueryRequest {
                query_type: query_request::QueryType::Ask as i32,
                query_sparql: format!("ASK {{ ?s ?p <http://example.org/{i}> }}"),
            });
            svc.query(request).await
        });
        handles.push(handle);
    }

    // Wait for all operations
    for handle in handles {
        let _ = handle.await;
    }

    // Assert: Metrics consistent
    let request = Request::new(GetMetricsRequest {});
    let metrics = service.get_metrics(request).await.unwrap().into_inner().metrics.unwrap();
    assert_eq!(metrics.total_queries, 10, "Should record all concurrent queries");
    assert_eq!(metrics.total_requests, 10, "Should record all concurrent requests");
}

// ============================================================================
// Test Suite: Error State Verification
// ============================================================================

#[tokio::test]
async fn test_service_handles_malformed_requests_gracefully() {
    // Arrange
    let config = SidecarConfig::default();
    let service = KgcSidecarService::new(config);

    // Act: Invalid query type
    let request = Request::new(QueryRequest {
        query_type: 9999, // Invalid
        query_sparql: "".to_string(),
    });
    let response = service.query(request).await;

    // Assert: Service handles gracefully (doesn't panic)
    assert!(response.is_ok(), "Service should handle invalid input gracefully");
}

#[tokio::test]
async fn test_service_state_persists_across_requests() {
    // Arrange
    let config = SidecarConfig::default();
    let service = std::sync::Arc::new(KgcSidecarService::new(config));

    // Act: First request
    let _ = service.query(Request::new(QueryRequest {
        query_type: query_request::QueryType::Ask as i32,
        query_sparql: "ASK {}".to_string(),
    })).await;

    // Second request
    let _ = service.apply_transaction(Request::new(ApplyTransactionRequest {
        rdf_data: vec![],
        schema_iri: "".to_string(),
    })).await;

    // Assert: Cumulative state maintained
    let metrics = service.get_metrics(Request::new(GetMetricsRequest {}))
        .await.unwrap().into_inner().metrics.unwrap();

    assert_eq!(metrics.total_requests, 2, "State should accumulate across requests");
    assert_eq!(metrics.total_queries, 1, "Query count should persist");
    assert_eq!(metrics.total_transactions, 1, "Transaction count should persist");
}
