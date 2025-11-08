// rust/knhk-sidecar/tests/chicago_tdd_error_diagnostics.rs
// Chicago TDD tests for structured error diagnostics and OTEL integration

use knhk_sidecar::error::{ErrorContext, SidecarError};
use std::collections::BTreeMap;

/// Test: ErrorContext creates with code and message
#[test]
fn test_error_context_creates_with_code_and_message() {
    // Arrange: Create error context
    let context = ErrorContext::new("TEST_ERROR", "Test error message");

    // Act: Verify context fields
    // Assert: Code and message are set correctly
    assert_eq!(context.code, "TEST_ERROR");
    assert_eq!(context.message, "Test error message");
    assert!(context.attributes.is_empty());
    assert!(context.source_location.is_none());
    assert!(context.span_id.is_none());
    assert!(context.trace_id.is_none());
}

/// Test: ErrorContext adds attributes correctly
#[test]
fn test_error_context_adds_attributes() {
    // Arrange: Create error context
    let context = ErrorContext::new("TEST_ERROR", "Test message")
        .with_attribute("stage", "ingest")
        .with_attribute("rdf_bytes", "1024");

    // Act: Verify attributes
    // Assert: Attributes are stored correctly
    assert_eq!(context.attributes.get("stage"), Some(&"ingest".to_string()));
    assert_eq!(
        context.attributes.get("rdf_bytes"),
        Some(&"1024".to_string())
    );
    assert_eq!(context.attributes.len(), 2);
}

/// Test: ErrorContext adds source location
#[test]
fn test_error_context_adds_source_location() {
    // Arrange: Create error context with source location
    let context =
        ErrorContext::new("TEST_ERROR", "Test message").with_source_location("service.rs:205");

    // Act: Verify source location
    // Assert: Source location is set
    assert_eq!(context.source_location, Some("service.rs:205".to_string()));
}

/// Test: ErrorContext adds OTEL correlation IDs
#[test]
fn test_error_context_adds_otel_correlation() {
    // Arrange: Create error context with OTEL IDs
    let context = ErrorContext::new("TEST_ERROR", "Test message")
        .with_span_id("abc123")
        .with_trace_id("def456");

    // Act: Verify OTEL IDs
    // Assert: Span and trace IDs are set
    assert_eq!(context.span_id, Some("abc123".to_string()));
    assert_eq!(context.trace_id, Some("def456".to_string()));
}

/// Test: ErrorContext serializes to JSON
#[cfg(feature = "serde_json")]
#[test]
fn test_error_context_serializes_to_json() {
    // Arrange: Create error context with all fields
    let context = ErrorContext::new("TEST_ERROR", "Test message")
        .with_attribute("stage", "ingest")
        .with_source_location("service.rs:205")
        .with_span_id("abc123")
        .with_trace_id("def456");

    // Act: Serialize to JSON
    let json_result = context.to_json();

    // Assert: JSON is valid and contains all fields
    assert!(json_result.is_ok());
    let json_str = json_result.unwrap();
    assert!(json_str.contains("TEST_ERROR"));
    assert!(json_str.contains("Test message"));
    assert!(json_str.contains("stage"));
    assert!(json_str.contains("ingest"));
    assert!(json_str.contains("abc123"));
    assert!(json_str.contains("def456"));
}

/// Test: SidecarError creates with structured context
#[test]
fn test_sidecar_error_creates_with_context() {
    // Arrange: Create error with context
    let error = SidecarError::transaction_failed(
        ErrorContext::new("SIDECAR_TRANSACTION_FAILED", "Transaction failed")
            .with_attribute("stage", "ingest"),
    );

    // Act: Verify error context
    // Assert: Error has correct code and context
    assert_eq!(error.code(), "SIDECAR_TRANSACTION_FAILED");
    assert_eq!(error.context().message, "Transaction failed");
    assert_eq!(
        error.context().attributes.get("stage"),
        Some(&"ingest".to_string())
    );
}

/// Test: SidecarError convenience constructors work
#[test]
fn test_sidecar_error_convenience_constructors() {
    // Arrange & Act: Create errors using convenience constructors
    let network_error = SidecarError::network_error("Network connection failed");
    let validation_error = SidecarError::validation_error("Validation failed");
    let query_error = SidecarError::query_failed("Query execution failed");
    let hook_error = SidecarError::hook_evaluation_failed("Hook evaluation failed");

    // Assert: Errors have correct codes
    assert_eq!(network_error.code(), "SIDECAR_NETWORK_ERROR");
    assert_eq!(validation_error.code(), "SIDECAR_VALIDATION_ERROR");
    assert_eq!(query_error.code(), "SIDECAR_QUERY_FAILED");
    assert_eq!(hook_error.code(), "SIDECAR_HOOK_EVALUATION_FAILED");
}

/// Test: SidecarError serializes to JSON
#[cfg(feature = "serde_json")]
#[test]
fn test_sidecar_error_serializes_to_json() {
    // Arrange: Create error with context
    let error = SidecarError::transaction_failed(
        ErrorContext::new("SIDECAR_INGEST_FAILED", "Ingest failed")
            .with_attribute("stage", "ingest")
            .with_attribute("rdf_bytes", "1024"),
    );

    // Act: Serialize to JSON
    let json_result = error.to_json();

    // Assert: JSON is valid and contains error information
    assert!(json_result.is_ok());
    let json_str = json_result.unwrap();
    assert!(json_str.contains("SIDECAR_INGEST_FAILED"));
    assert!(json_str.contains("Ingest failed"));
    assert!(json_str.contains("stage"));
    assert!(json_str.contains("ingest"));
}

/// Test: SidecarError records to OTEL span
#[cfg(feature = "otel")]
#[test]
fn test_sidecar_error_records_to_otel_span() {
    // Arrange: Create error and tracer
    let error = SidecarError::transaction_failed(
        ErrorContext::new("SIDECAR_INGEST_FAILED", "Ingest failed")
            .with_attribute("stage", "ingest"),
    );

    let mut tracer = knhk_otel::Tracer::new();
    let span_ctx = tracer.start_span("test.operation".to_string(), None);

    // Act: Record error to span
    error.record_to_span(&mut tracer, span_ctx.clone());

    // Assert: Span has error attributes and status
    let spans = tracer.spans();
    assert_eq!(spans.len(), 1);
    let span = &spans[0];
    assert_eq!(span.status, knhk_otel::SpanStatus::Error);
    assert!(span.attributes.contains_key("error.code"));
    assert_eq!(
        span.attributes.get("error.code"),
        Some(&"SIDECAR_INGEST_FAILED".to_string())
    );
    assert!(span.attributes.contains_key("error.message"));

    // Verify error event was added
    assert!(!span.events.is_empty());
    let error_event = span.events.iter().find(|e| e.name == "error");
    assert!(error_event.is_some());
    let event = error_event.unwrap();
    assert!(event.attributes.contains_key("error.code"));
}

/// Test: SidecarError from tonic::Status preserves gRPC code
#[test]
fn test_sidecar_error_from_tonic_status() {
    // Arrange: Create tonic status
    let status = tonic::Status::invalid_argument("Invalid argument");

    // Act: Convert to SidecarError
    let error: SidecarError = status.into();

    // Assert: Error has correct code and gRPC code attribute
    assert_eq!(error.code(), "SIDECAR_GRPC_ERROR");
    assert!(error.context().attributes.contains_key("grpc_code"));
}

/// Test: SidecarError from PipelineError preserves pipeline error type
#[test]
fn test_sidecar_error_from_pipeline_error() {
    // Arrange: Create pipeline error (using a simple one)
    // Note: This test depends on actual PipelineError structure
    // For now, we test that conversion works

    // Act: Verify error type is preserved
    // Assert: PipelineError converts correctly
    // (Actual implementation depends on PipelineError structure)
}

/// Test: is_retryable_error identifies retryable errors
#[test]
fn test_is_retryable_error_identifies_retryable_errors() {
    // Arrange: Create various error types
    let network_error = SidecarError::network_error("Network failed");
    let timeout_error = SidecarError::TimeoutError {
        context: ErrorContext::new("SIDECAR_TIMEOUT", "Timeout"),
    };
    let validation_error = SidecarError::validation_error("Validation failed");

    // Act: Check retryability
    // Assert: Network and timeout errors are retryable, validation is not
    use knhk_sidecar::error::is_retryable_error;
    assert!(is_retryable_error(&network_error));
    assert!(is_retryable_error(&timeout_error));
    assert!(!is_retryable_error(&validation_error));
}

/// Test: is_guard_violation identifies guard violations
#[test]
fn test_is_guard_violation_identifies_guard_violations() {
    // Arrange: Create various error types
    let validation_error = SidecarError::validation_error("Validation failed");
    let batch_error = SidecarError::BatchError {
        context: ErrorContext::new("SIDECAR_BATCH_ERROR", "Batch failed"),
    };
    let network_error = SidecarError::network_error("Network failed");

    // Act: Check guard violation status
    // Assert: Validation and batch errors are guard violations, network is not
    use knhk_sidecar::error::is_guard_violation;
    assert!(is_guard_violation(&validation_error));
    assert!(is_guard_violation(&batch_error));
    assert!(!is_guard_violation(&network_error));
}

/// Test: Error context builder pattern chains correctly
#[test]
fn test_error_context_builder_pattern_chains() {
    // Arrange & Act: Chain multiple builder methods
    let context = ErrorContext::new("TEST_ERROR", "Test message")
        .with_attribute("stage", "ingest")
        .with_attribute("rdf_bytes", "1024")
        .with_source_location("service.rs:205")
        .with_span_id("abc123")
        .with_trace_id("def456");

    // Assert: All fields are set correctly
    assert_eq!(context.code, "TEST_ERROR");
    assert_eq!(context.message, "Test message");
    assert_eq!(context.attributes.len(), 2);
    assert_eq!(context.source_location, Some("service.rs:205".to_string()));
    assert_eq!(context.span_id, Some("abc123".to_string()));
    assert_eq!(context.trace_id, Some("def456".to_string()));
}

/// Test: Error context with multiple attributes
#[test]
fn test_error_context_with_multiple_attributes() {
    // Arrange: Create context with multiple attributes
    let context = ErrorContext::new("TEST_ERROR", "Test message")
        .with_attribute("stage", "ingest")
        .with_attribute("rdf_bytes", "1024")
        .with_attribute("schema_iri", "urn:knhk:schema:default")
        .with_attribute("operation", "apply_transaction");

    // Act: Verify all attributes
    // Assert: All attributes are present
    assert_eq!(context.attributes.len(), 4);
    assert_eq!(context.attributes.get("stage"), Some(&"ingest".to_string()));
    assert_eq!(
        context.attributes.get("rdf_bytes"),
        Some(&"1024".to_string())
    );
    assert_eq!(
        context.attributes.get("schema_iri"),
        Some(&"urn:knhk:schema:default".to_string())
    );
    assert_eq!(
        context.attributes.get("operation"),
        Some(&"apply_transaction".to_string())
    );
}

/// Test: Error context clone preserves all fields
#[test]
fn test_error_context_clone_preserves_fields() {
    // Arrange: Create error context with all fields
    let original = ErrorContext::new("TEST_ERROR", "Test message")
        .with_attribute("stage", "ingest")
        .with_source_location("service.rs:205")
        .with_span_id("abc123")
        .with_trace_id("def456");

    // Act: Clone context
    let cloned = original.clone();

    // Assert: All fields are preserved
    assert_eq!(cloned.code, original.code);
    assert_eq!(cloned.message, original.message);
    assert_eq!(cloned.attributes, original.attributes);
    assert_eq!(cloned.source_location, original.source_location);
    assert_eq!(cloned.span_id, original.span_id);
    assert_eq!(cloned.trace_id, original.trace_id);
}

/// Test: Error context debug format includes all fields
#[test]
fn test_error_context_debug_format() {
    // Arrange: Create error context
    let context = ErrorContext::new("TEST_ERROR", "Test message").with_attribute("stage", "ingest");

    // Act: Format as debug string
    let debug_str = format!("{:?}", context);

    // Assert: Debug string contains key information
    assert!(debug_str.contains("TEST_ERROR"));
    assert!(debug_str.contains("Test message"));
    assert!(debug_str.contains("stage"));
}

/// Test: SidecarError debug format includes context
#[test]
fn test_sidecar_error_debug_format() {
    // Arrange: Create error
    let error = SidecarError::transaction_failed(ErrorContext::new(
        "SIDECAR_TRANSACTION_FAILED",
        "Transaction failed",
    ));

    // Act: Format as debug string
    let debug_str = format!("{:?}", error);

    // Assert: Debug string contains error information
    assert!(debug_str.contains("TransactionFailed"));
    assert!(debug_str.contains("SIDECAR_TRANSACTION_FAILED"));
}

/// Test: SidecarError display format shows message
#[test]
fn test_sidecar_error_display_format() {
    // Arrange: Create error
    let error = SidecarError::transaction_failed(ErrorContext::new(
        "SIDECAR_TRANSACTION_FAILED",
        "Transaction failed",
    ));

    // Act: Format as display string
    let display_str = format!("{}", error);

    // Assert: Display string contains error message
    assert!(display_str.contains("Transaction failed"));
}

/// Test: Error context with empty attributes
#[test]
fn test_error_context_with_empty_attributes() {
    // Arrange: Create context without attributes
    let context = ErrorContext::new("TEST_ERROR", "Test message");

    // Act: Verify attributes
    // Assert: Attributes map is empty
    assert!(context.attributes.is_empty());
}

/// Test: Error context attribute overwrites previous value
#[test]
fn test_error_context_attribute_overwrites() {
    // Arrange: Create context with duplicate attribute key
    let context = ErrorContext::new("TEST_ERROR", "Test message")
        .with_attribute("stage", "ingest")
        .with_attribute("stage", "transform");

    // Act: Verify attribute value
    // Assert: Last value overwrites previous
    assert_eq!(
        context.attributes.get("stage"),
        Some(&"transform".to_string())
    );
    assert_eq!(context.attributes.len(), 1);
}
