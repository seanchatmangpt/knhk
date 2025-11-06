// rust/knhk-sidecar/tests/service_implementation_test.rs
// Chicago TDD tests for sidecar service implementation
// Tests actual behavior through state verification (no mocks)

use knhk_sidecar::config::SidecarConfig;
use knhk_sidecar::service::{KgcSidecarService, proto};
use proto::kgc_sidecar_server::KgcSidecar;
use tonic::Request;

/// Helper to create test service instance
fn create_test_service() -> KgcSidecarService {
    let config = SidecarConfig {
        port: 50051,
        circuit_breaker_failure_threshold: 5,
        circuit_breaker_reset_timeout_ms: 60000,
        retry_max_attempts: 3,
        retry_initial_delay_ms: 100,
        retry_max_delay_ms: 5000,
    };
    KgcSidecarService::new(config)
}

#[tokio::test]
async fn test_apply_transaction_with_valid_rdf() {
    // Arrange: Create service and prepare valid RDF data
    let service = create_test_service();
    let rdf_data = b"<http://example.org/alice> <http://example.org/knows> <http://example.org/bob> .".to_vec();

    let request = Request::new(proto::ApplyTransactionRequest {
        rdf_data,
        transaction_metadata: std::collections::HashMap::new(),
    });

    // Act: Execute transaction
    let response = service.apply_transaction(request).await;

    // Assert: Transaction succeeded with receipt
    assert!(response.is_ok(), "Transaction should succeed");
    let response = response.unwrap().into_inner();
    assert!(response.committed, "Transaction should be committed");
    assert!(!response.transaction_id.is_empty(), "Transaction ID should be generated");
    assert!(response.receipt.is_some(), "Receipt should be present");
    assert!(response.errors.is_empty(), "No errors should be present");

    // Assert: Receipt has valid data
    let receipt = response.receipt.unwrap();
    assert!(!receipt.receipt_id.is_empty(), "Receipt ID should be present");
    assert!(receipt.ticks > 0, "Receipt should have tick count");
    assert!(receipt.span_id > 0, "Receipt should have span ID");
}

#[tokio::test]
async fn test_apply_transaction_with_invalid_utf8() {
    // Arrange: Create service with invalid UTF-8 data
    let service = create_test_service();
    let rdf_data = vec![0xFF, 0xFE, 0xFD]; // Invalid UTF-8

    let request = Request::new(proto::ApplyTransactionRequest {
        rdf_data,
        transaction_metadata: std::collections::HashMap::new(),
    });

    // Act: Execute transaction
    let response = service.apply_transaction(request).await;

    // Assert: Transaction failed with proper error
    assert!(response.is_err(), "Transaction should fail with invalid UTF-8");
    let err = response.err().unwrap();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
    assert!(err.message().contains("Invalid UTF-8"), "Error message should mention UTF-8");
}

#[tokio::test]
async fn test_apply_transaction_with_invalid_rdf_syntax() {
    // Arrange: Create service with syntactically invalid RDF
    let service = create_test_service();
    let rdf_data = b"<incomplete triple".to_vec();

    let request = Request::new(proto::ApplyTransactionRequest {
        rdf_data,
        transaction_metadata: std::collections::HashMap::new(),
    });

    // Act: Execute transaction
    let response = service.apply_transaction(request).await;

    // Assert: Transaction failed but response is Ok (error in response body)
    assert!(response.is_ok(), "Response should be Ok even if transaction failed");
    let response = response.unwrap().into_inner();
    assert!(!response.committed, "Transaction should not be committed");
    assert!(!response.errors.is_empty(), "Errors should be present");
    assert!(response.errors[0].contains("Ingest failed"), "Error should mention ingest failure");
}

#[tokio::test]
async fn test_apply_transaction_with_multiple_triples() {
    // Arrange: Create service with multiple triples
    let service = create_test_service();
    let rdf_data = b"
        <http://example.org/alice> <http://example.org/knows> <http://example.org/bob> .
        <http://example.org/alice> <http://example.org/age> \"30\" .
        <http://example.org/bob> <http://example.org/age> \"32\" .
    ".to_vec();

    let request = Request::new(proto::ApplyTransactionRequest {
        rdf_data,
        transaction_metadata: std::collections::HashMap::new(),
    });

    // Act: Execute transaction
    let response = service.apply_transaction(request).await;

    // Assert: Transaction succeeded
    assert!(response.is_ok(), "Transaction should succeed with multiple triples");
    let response = response.unwrap().into_inner();
    assert!(response.committed, "Transaction should be committed");
    assert!(response.receipt.is_some(), "Receipt should be present");
}

#[tokio::test]
async fn test_query_ask_returns_boolean() {
    // Arrange: Create service and ASK query
    let service = create_test_service();
    let request = Request::new(proto::QueryRequest {
        query_type: 0, // ASK
        query: "ASK { ?s ?p ?o }".to_string(),
        rdf_data: vec![],
        limit: 0,
    });

    // Act: Execute query
    let response = service.query(request).await;

    // Assert: Query succeeded with boolean result
    assert!(response.is_ok(), "Query should succeed");
    let response = response.unwrap().into_inner();
    assert!(response.success, "Query should be successful");
    assert!(response.result.is_some(), "Result should be present");

    let result = response.result.unwrap();
    assert!(!result.bindings.is_empty(), "Should have result bindings");
    assert!(result.bindings[0].value == "true" || result.bindings[0].value == "false",
            "ASK query should return boolean");
}

#[tokio::test]
async fn test_query_select_with_rdf_data() {
    // Arrange: Create service with RDF data for SELECT query
    let service = create_test_service();
    let rdf_data = b"<http://example.org/alice> <http://example.org/name> \"Alice\" .".to_vec();

    let request = Request::new(proto::QueryRequest {
        query_type: 1, // SELECT
        query: "SELECT ?s WHERE { ?s ?p ?o }".to_string(),
        rdf_data,
        limit: 10,
    });

    // Act: Execute query
    let response = service.query(request).await;

    // Assert: Query succeeded with results
    assert!(response.is_ok(), "SELECT query should succeed");
    let response = response.unwrap().into_inner();
    assert!(response.success, "Query should be successful");
    assert!(response.result.is_some(), "Result should be present");

    let result = response.result.unwrap();
    assert!(!result.bindings.is_empty(), "Should have result bindings");
    assert!(result.bindings[0].value.contains("example.org"), "Should contain subject IRI");
}

#[tokio::test]
async fn test_query_construct_returns_triples() {
    // Arrange: Create service with RDF data for CONSTRUCT query
    let service = create_test_service();
    let rdf_data = b"<http://example.org/alice> <http://example.org/knows> <http://example.org/bob> .".to_vec();

    let request = Request::new(proto::QueryRequest {
        query_type: 2, // CONSTRUCT
        query: "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }".to_string(),
        rdf_data,
        limit: 10,
    });

    // Act: Execute query
    let response = service.query(request).await;

    // Assert: Query succeeded with constructed triples
    assert!(response.is_ok(), "CONSTRUCT query should succeed");
    let response = response.unwrap().into_inner();
    assert!(response.success, "Query should be successful");
    assert!(response.result.is_some(), "Result should be present");

    let result = response.result.unwrap();
    assert!(!result.bindings.is_empty(), "Should have constructed triples");
    assert!(result.bindings[0].value.contains("<http"), "Should be N-Triples format");
}

#[tokio::test]
async fn test_query_with_empty_data_returns_empty_results() {
    // Arrange: Create service with empty RDF data
    let service = create_test_service();
    let request = Request::new(proto::QueryRequest {
        query_type: 1, // SELECT
        query: "SELECT ?s WHERE { ?s ?p ?o }".to_string(),
        rdf_data: vec![],
        limit: 10,
    });

    // Act: Execute query
    let response = service.query(request).await;

    // Assert: Query succeeded but no results
    assert!(response.is_ok(), "Query should succeed");
    let response = response.unwrap().into_inner();
    assert!(response.success, "Query should be successful");
    assert!(response.result.is_some(), "Result should be present");

    let result = response.result.unwrap();
    assert!(result.bindings.is_empty(), "Should have no results with empty data");
}

#[tokio::test]
async fn test_validate_graph_with_valid_rdf() {
    // Arrange: Create service with valid RDF
    let service = create_test_service();
    let rdf_data = b"<http://example.org/alice> <http://example.org/age> \"30\" .".to_vec();

    let request = Request::new(proto::ValidateGraphRequest {
        rdf_data,
        schema_iri: "urn:knhk:schema:test".to_string(),
    });

    // Act: Validate graph
    let response = service.validate_graph(request).await;

    // Assert: Validation succeeded
    assert!(response.is_ok(), "Validation should succeed");
    let response = response.unwrap().into_inner();
    assert!(response.valid, "Graph should be valid");
    assert!(response.errors.is_empty(), "No errors should be present");
    assert!(response.warnings.is_empty(), "No warnings should be present");
}

#[tokio::test]
async fn test_validate_graph_with_invalid_utf8() {
    // Arrange: Create service with invalid UTF-8
    let service = create_test_service();
    let rdf_data = vec![0xFF, 0xFE, 0xFD];

    let request = Request::new(proto::ValidateGraphRequest {
        rdf_data,
        schema_iri: "urn:knhk:schema:test".to_string(),
    });

    // Act: Validate graph
    let response = service.validate_graph(request).await;

    // Assert: Validation failed
    assert!(response.is_ok(), "Response should be Ok even if validation failed");
    let response = response.unwrap().into_inner();
    assert!(!response.valid, "Graph should be invalid");
    assert!(!response.errors.is_empty(), "Errors should be present");
    assert!(response.errors[0].contains("Invalid UTF-8"), "Error should mention UTF-8");
}

#[tokio::test]
async fn test_validate_graph_with_invalid_rdf_syntax() {
    // Arrange: Create service with syntactically invalid RDF
    let service = create_test_service();
    let rdf_data = b"<incomplete".to_vec();

    let request = Request::new(proto::ValidateGraphRequest {
        rdf_data,
        schema_iri: "urn:knhk:schema:test".to_string(),
    });

    // Act: Validate graph
    let response = service.validate_graph(request).await;

    // Assert: Validation failed
    assert!(response.is_ok(), "Response should be Ok even if validation failed");
    let response = response.unwrap().into_inner();
    assert!(!response.valid, "Graph should be invalid");
    assert!(!response.errors.is_empty(), "Errors should be present");
    assert!(response.errors[0].contains("Parse failed"), "Error should mention parse failure");
}

#[tokio::test]
async fn test_evaluate_hook_with_valid_rdf() {
    // Arrange: Create service with valid RDF for hook
    let service = create_test_service();
    let rdf_data = b"<http://example.org/event> <http://example.org/type> \"test\" .".to_vec();

    let request = Request::new(proto::EvaluateHookRequest {
        hook_id: "test_hook".to_string(),
        rdf_data,
    });

    // Act: Evaluate hook
    let response = service.evaluate_hook(request).await;

    // Assert: Hook evaluation succeeded
    assert!(response.is_ok(), "Hook evaluation should succeed");
    let response = response.unwrap().into_inner();
    assert!(response.fired, "Hook should fire with valid data");
    assert!(response.result.is_some(), "Hook result should be present");
    assert!(response.receipt.is_some(), "Receipt should be present");
    assert!(response.errors.is_empty(), "No errors should be present");

    // Assert: Receipt has valid data
    let receipt = response.receipt.unwrap();
    assert!(!receipt.receipt_id.is_empty(), "Receipt ID should be present");
    assert!(receipt.ticks > 0, "Receipt should have tick count");
}

#[tokio::test]
async fn test_evaluate_hook_with_invalid_utf8() {
    // Arrange: Create service with invalid UTF-8
    let service = create_test_service();
    let rdf_data = vec![0xFF, 0xFE, 0xFD];

    let request = Request::new(proto::EvaluateHookRequest {
        hook_id: "test_hook".to_string(),
        rdf_data,
    });

    // Act: Evaluate hook
    let response = service.evaluate_hook(request).await;

    // Assert: Hook evaluation failed
    assert!(response.is_err(), "Hook evaluation should fail with invalid UTF-8");
    let err = response.err().unwrap();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
    assert!(err.message().contains("Invalid UTF-8"), "Error message should mention UTF-8");
}

#[tokio::test]
async fn test_evaluate_hook_with_empty_rdf() {
    // Arrange: Create service with empty RDF
    let service = create_test_service();
    let rdf_data = vec![];

    let request = Request::new(proto::EvaluateHookRequest {
        hook_id: "test_hook".to_string(),
        rdf_data,
    });

    // Act: Evaluate hook
    let response = service.evaluate_hook(request).await;

    // Assert: Hook evaluation succeeded but hook did not fire
    assert!(response.is_ok(), "Hook evaluation should succeed");
    let response = response.unwrap().into_inner();
    assert!(!response.fired, "Hook should not fire with empty data");
}

#[tokio::test]
async fn test_evaluate_hook_with_multiple_triples() {
    // Arrange: Create service with multiple triples
    let service = create_test_service();
    let rdf_data = b"
        <http://example.org/event1> <http://example.org/type> \"test\" .
        <http://example.org/event2> <http://example.org/type> \"test\" .
    ".to_vec();

    let request = Request::new(proto::EvaluateHookRequest {
        hook_id: "test_hook".to_string(),
        rdf_data,
    });

    // Act: Evaluate hook
    let response = service.evaluate_hook(request).await;

    // Assert: Hook evaluation succeeded
    assert!(response.is_ok(), "Hook evaluation should succeed with multiple triples");
    let response = response.unwrap().into_inner();
    assert!(response.fired, "Hook should fire");
    assert!(response.receipt.is_some(), "Receipt should be present");
}

#[tokio::test]
async fn test_health_check_returns_healthy() {
    // Arrange: Create service
    let service = create_test_service();
    let request = Request::new(proto::HealthCheckRequest {});

    // Act: Check health
    let response = service.health_check(request).await;

    // Assert: Service is healthy
    assert!(response.is_ok(), "Health check should succeed");
    let response = response.unwrap().into_inner();
    assert_eq!(response.status, 1, "Status should be HEALTHY (1)");
    assert!(response.message.contains("healthy"), "Message should indicate healthy state");
    assert!(response.timestamp_ms > 0, "Timestamp should be present");
}

#[tokio::test]
async fn test_get_metrics_returns_valid_metrics() {
    // Arrange: Create service and execute some operations
    let service = create_test_service();

    // Execute a transaction to generate metrics
    let rdf_data = b"<http://example.org/s> <http://example.org/p> <http://example.org/o> .".to_vec();
    let _ = service.apply_transaction(Request::new(proto::ApplyTransactionRequest {
        rdf_data,
        transaction_metadata: std::collections::HashMap::new(),
    })).await;

    // Act: Get metrics
    let request = Request::new(proto::GetMetricsRequest {});
    let response = service.get_metrics(request).await;

    // Assert: Metrics are present
    assert!(response.is_ok(), "Get metrics should succeed");
    let response = response.unwrap().into_inner();
    assert!(response.metrics.is_some(), "Metrics should be present");

    let metrics = response.metrics.unwrap();
    assert!(metrics.total_requests > 0, "Total requests should be > 0");
    assert!(metrics.total_transactions > 0, "Total transactions should be > 0");
    assert!(metrics.last_request_time_ms > 0, "Last request time should be present");
}

#[tokio::test]
async fn test_concurrent_transactions() {
    // Arrange: Create service and prepare multiple transactions
    let service = std::sync::Arc::new(create_test_service());
    let mut handles = vec![];

    // Act: Execute 5 concurrent transactions
    for i in 0..5 {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            let rdf_data = format!(
                "<http://example.org/tx{}> <http://example.org/type> \"transaction\" .",
                i
            ).into_bytes();

            let request = Request::new(proto::ApplyTransactionRequest {
                rdf_data,
                transaction_metadata: std::collections::HashMap::new(),
            });

            service_clone.apply_transaction(request).await
        });
        handles.push(handle);
    }

    // Wait for all transactions to complete
    let results = futures::future::join_all(handles).await;

    // Assert: All transactions succeeded
    for result in results {
        assert!(result.is_ok(), "Task should not panic");
        let response = result.unwrap();
        assert!(response.is_ok(), "Transaction should succeed");
        let response = response.unwrap().into_inner();
        assert!(response.committed, "Transaction should be committed");
    }
}
