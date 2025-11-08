// rust/knhk-connectors/tests/error_diagnostics_test.rs
// Chicago TDD tests for enhanced ConnectorError diagnostics
// Tests focus on behavior: error codes, messages, retryability

use knhk_connectors::ConnectorError;

#[test]
fn test_error_code_extraction() {
    // Arrange: Create various error types
    let validation_error = ConnectorError::ValidationFailed("Invalid input".to_string());
    let network_error = ConnectorError::NetworkError("Connection timeout".to_string());
    let rate_limit_error = ConnectorError::RateLimitError {
        message: "Rate limit exceeded".to_string(),
        retry_after_ms: Some(5000),
    };

    // Act: Extract error codes
    let validation_code = validation_error.code();
    let network_code = network_error.code();
    let rate_limit_code = rate_limit_error.code();

    // Assert: Error codes match expected values
    assert_eq!(validation_code, "CONNECTOR_VALIDATION_FAILED");
    assert_eq!(network_code, "CONNECTOR_NETWORK_ERROR");
    assert_eq!(rate_limit_code, "CONNECTOR_RATE_LIMIT_ERROR");
}

#[test]
fn test_error_message_extraction() {
    // Arrange: Create errors with specific messages
    let error_msg = "Schema validation failed: invalid IRI format";
    let error = ConnectorError::SchemaMismatch(error_msg.to_string());

    // Act: Extract error message
    let extracted_msg = error.message();

    // Assert: Message matches original
    assert_eq!(extracted_msg, error_msg);
}

#[test]
fn test_rate_limit_error_message_extraction() {
    // Arrange: Create rate limit error
    let error = ConnectorError::RateLimitError {
        message: "API rate limit exceeded".to_string(),
        retry_after_ms: Some(10000),
    };

    // Act: Extract message
    let msg = error.message();

    // Assert: Message extracted correctly
    assert_eq!(msg, "API rate limit exceeded");
}

#[test]
fn test_retryable_errors() {
    // Arrange: Create retryable and non-retryable errors
    let network_error = ConnectorError::NetworkError("Connection failed".to_string());
    let io_error = ConnectorError::IoError("File read failed".to_string());
    let rate_limit_error = ConnectorError::RateLimitError {
        message: "Rate limited".to_string(),
        retry_after_ms: Some(5000),
    };
    let validation_error = ConnectorError::ValidationFailed("Invalid input".to_string());
    let guard_error = ConnectorError::GuardViolation("max_run_len > 8".to_string());

    // Act: Check retryability
    let network_retryable = network_error.is_retryable();
    let io_retryable = io_error.is_retryable();
    let rate_limit_retryable = rate_limit_error.is_retryable();
    let validation_retryable = validation_error.is_retryable();
    let guard_retryable = guard_error.is_retryable();

    // Assert: Retryable errors identified correctly
    assert!(network_retryable, "Network errors should be retryable");
    assert!(io_retryable, "IO errors should be retryable");
    assert!(
        rate_limit_retryable,
        "Rate limit errors should be retryable"
    );
    assert!(
        !validation_retryable,
        "Validation errors should not be retryable"
    );
    assert!(!guard_retryable, "Guard violations should not be retryable");
}

#[test]
fn test_error_code_consistency() {
    // Arrange: Create multiple instances of same error type
    let error1 = ConnectorError::NetworkError("Timeout".to_string());
    let error2 = ConnectorError::NetworkError("Connection refused".to_string());

    // Act: Extract codes
    let code1 = error1.code();
    let code2 = error2.code();

    // Assert: Same error type produces same code
    assert_eq!(code1, code2, "Same error type should produce same code");
    assert_eq!(code1, "CONNECTOR_NETWORK_ERROR");
}

#[test]
fn test_all_error_types_have_codes() {
    // Arrange: Create one instance of each error type
    let errors = vec![
        ConnectorError::ValidationFailed("test".to_string()),
        ConnectorError::SchemaMismatch("test".to_string()),
        ConnectorError::GuardViolation("test".to_string()),
        ConnectorError::ParseError("test".to_string()),
        ConnectorError::IoError("test".to_string()),
        ConnectorError::NetworkError("test".to_string()),
        ConnectorError::AuthenticationError("test".to_string()),
        ConnectorError::RateLimitError {
            message: "test".to_string(),
            retry_after_ms: None,
        },
    ];

    // Act & Assert: All errors have non-empty codes
    for error in errors {
        let code = error.code();
        assert!(!code.is_empty(), "All errors should have non-empty codes");
        assert!(
            code.starts_with("CONNECTOR_"),
            "Error codes should be prefixed with CONNECTOR_"
        );
    }
}

#[test]
fn test_all_error_types_have_messages() {
    // Arrange: Create one instance of each error type
    let errors = vec![
        ConnectorError::ValidationFailed("msg1".to_string()),
        ConnectorError::SchemaMismatch("msg2".to_string()),
        ConnectorError::GuardViolation("msg3".to_string()),
        ConnectorError::ParseError("msg4".to_string()),
        ConnectorError::IoError("msg5".to_string()),
        ConnectorError::NetworkError("msg6".to_string()),
        ConnectorError::AuthenticationError("msg7".to_string()),
        ConnectorError::RateLimitError {
            message: "msg8".to_string(),
            retry_after_ms: None,
        },
    ];

    // Act & Assert: All errors have non-empty messages
    for error in errors {
        let msg = error.message();
        assert!(!msg.is_empty(), "All errors should have non-empty messages");
    }
}

#[test]
fn test_rate_limit_error_with_retry_after() {
    // Arrange: Create rate limit error with retry_after_ms
    let error = ConnectorError::RateLimitError {
        message: "Rate limit exceeded".to_string(),
        retry_after_ms: Some(10000),
    };

    // Act: Extract message and check retryability
    let msg = error.message();
    let retryable = error.is_retryable();

    // Assert: Message extracted and error is retryable
    assert_eq!(msg, "Rate limit exceeded");
    assert!(retryable, "Rate limit errors should be retryable");
}

#[test]
fn test_rate_limit_error_without_retry_after() {
    // Arrange: Create rate limit error without retry_after_ms
    let error = ConnectorError::RateLimitError {
        message: "Rate limit exceeded".to_string(),
        retry_after_ms: None,
    };

    // Act: Extract message and check retryability
    let msg = error.message();
    let retryable = error.is_retryable();

    // Assert: Message extracted and error is still retryable
    assert_eq!(msg, "Rate limit exceeded");
    assert!(
        retryable,
        "Rate limit errors should be retryable even without retry_after_ms"
    );
}
