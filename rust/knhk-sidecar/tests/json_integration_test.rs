// rust/knhk-sidecar/tests/json_integration_test.rs
// Integration tests for JSON endpoints in sidecar service

use knhk_sidecar::config::SidecarConfig;
use knhk_sidecar::service::{proto, KgcSidecarService};
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
        ..Default::default()
    };
    KgcSidecarService::new(config)
}

#[tokio::test]
async fn test_apply_transaction_with_json() {
    // Arrange: Create service and prepare JSON data
    let service = create_test_service();
    let json_data = r#"{
        "additions": [
            {"s": "http://example.org/s", "p": "http://example.org/p", "o": "http://example.org/o"}
        ],
        "removals": []
    }"#;

    let request = Request::new(proto::ApplyTransactionRequest {
        delta: None,
        json_data: json_data.as_bytes().to_vec(),
        actor: "test".to_string(),
        options: None,
    });

    // Act: Execute transaction
    let response = service.apply_transaction(request).await;

    // Assert: Transaction succeeded
    assert!(response.is_ok(), "JSON transaction should succeed");
    let response = response.unwrap().into_inner();
    assert!(response.committed, "Transaction should be committed");
    assert!(
        !response.transaction_id.is_empty(),
        "Transaction ID should be generated"
    );
    assert!(response.receipt.is_some(), "Receipt should be present");
}

#[tokio::test]
async fn test_query_with_json_data() {
    // Arrange: Create service and prepare JSON query data
    let service = create_test_service();
    let json_data = r#"[
        {"s": "http://example.org/s", "p": "http://example.org/p", "o": "http://example.org/o"}
    ]"#;

    let request = Request::new(proto::QueryRequest {
        query_type: 1, // SELECT
        query: "SELECT ?s WHERE { ?s ?p ?o }".to_string(),
        data_format: "json".to_string(),
        data: vec![],
        json_data: json_data.as_bytes().to_vec(),
    });

    // Act: Execute query
    let response = service.query(request).await;

    // Assert: Query succeeded
    assert!(response.is_ok(), "JSON query should succeed");
    let response = response.unwrap().into_inner();
    assert!(response.success, "Query should succeed");
}

#[tokio::test]
async fn test_validate_graph_with_json() {
    // Arrange: Create service and prepare JSON data
    let service = create_test_service();
    let json_data = r#"[
        {"s": "http://example.org/s", "p": "http://example.org/p", "o": "http://example.org/o"}
    ]"#;

    let request = Request::new(proto::ValidateGraphRequest {
        rdf_data: vec![],
        json_data: json_data.as_bytes().to_vec(),
        data_format: "json".to_string(),
        schema_iri: "urn:knhk:schema:default".to_string(),
    });

    // Act: Validate graph
    let response = service.validate_graph(request).await;

    // Assert: Validation succeeded (or failed gracefully)
    assert!(response.is_ok(), "JSON validation should succeed");
    let response = response.unwrap().into_inner();
    // Note: Validation may fail if schema doesn't match, but should not error
}

#[tokio::test]
async fn test_evaluate_hook_with_json() {
    // Arrange: Create service and prepare JSON data
    let service = create_test_service();
    let json_data = r#"[
        {"s": "http://example.org/s", "p": "http://example.org/p", "o": "http://example.org/o"}
    ]"#;

    let request = Request::new(proto::EvaluateHookRequest {
        hook_id: "test-hook".to_string(),
        rdf_data: vec![],
        json_data: json_data.as_bytes().to_vec(),
        data_format: "json".to_string(),
    });

    // Act: Evaluate hook
    let response = service.evaluate_hook(request).await;

    // Assert: Hook evaluation succeeded (or failed gracefully)
    assert!(response.is_ok(), "JSON hook evaluation should succeed");
    let response = response.unwrap().into_inner();
    // Note: Hook may not fire if conditions not met, but should not error
}

#[tokio::test]
async fn test_backward_compatibility_protobuf() {
    // Arrange: Test that protobuf format still works
    let service = create_test_service();
    let delta = Some(proto::Delta {
        additions: vec![proto::Triple {
            subject: "http://example.org/s".to_string(),
            predicate: "http://example.org/p".to_string(),
            object: "http://example.org/o".to_string(),
            graph: "".to_string(),
        }],
        removals: vec![],
    });

    let request = Request::new(proto::ApplyTransactionRequest {
        delta,
        json_data: vec![],
        actor: "test".to_string(),
        options: None,
    });

    // Act: Execute transaction
    let response = service.apply_transaction(request).await;

    // Assert: Protobuf transaction still works
    assert!(response.is_ok(), "Protobuf transaction should still work");
    let response = response.unwrap().into_inner();
    assert!(response.committed, "Transaction should be committed");
}
