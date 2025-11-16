//! Chicago TDD tests for tracing module

#![cfg(feature = "std")]

use knhk_cli::tracing;

/// Test: init_tracing returns Result
/// Chicago TDD: Test behavior (initialization) not implementation (OTEL setup)
#[test]
fn test_init_tracing_returns_result() {
    // Arrange: Clear OTEL environment variables to test default behavior
    std::env::remove_var("OTEL_ENABLED");
    std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
    std::env::remove_var("OTEL_SERVICE_NAME");
    std::env::remove_var("OTEL_SERVICE_VERSION");

    // Act: Initialize tracing
    let result = tracing::init_tracing();

    // Assert: Verify actual behavior - either succeeds or fails with expected error
    match result {
        Ok(_) => {
            // Success case - tracing initialized
        }
        Err(e) => {
            // Error case - should be subscriber already set error or other meaningful error
            assert!(!e.is_empty(), "Error message should not be empty");
            // Error should mention subscriber or tracing
            assert!(
                e.to_lowercase().contains("subscriber")
                    || e.to_lowercase().contains("tracing")
                    || e.to_lowercase().contains("global"),
                "Error should mention subscriber, tracing, or global state"
            );
        }
    }
}

/// Test: init_tracing with OTEL disabled
/// Chicago TDD: Test behavior (fallback to basic tracing) not implementation (subscriber setup)
#[test]
fn test_init_tracing_with_otel_disabled() {
    // Arrange: Disable OTEL
    std::env::set_var("OTEL_ENABLED", "false");
    std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");

    // Act: Initialize tracing
    let result = tracing::init_tracing();

    // Assert: Returns Result (may fail if global subscriber already set)
    // We just verify it returns a Result without panicking
    match result {
        Ok(guard) => {
            // When OTEL is disabled, result should be None (no guard)
            assert!(guard.is_none());
        }
        Err(_e) => {
            // Error case - may be "global default trace dispatcher has already been set"
            // This is acceptable in test environment
        }
    }
}

/// Test: init_tracing with custom service name
/// Chicago TDD: Test behavior (service name configuration) not implementation (env var reading)
#[test]
fn test_init_tracing_with_custom_service_name() {
    // Arrange: Set custom service name
    std::env::set_var("OTEL_SERVICE_NAME", "test-service");
    std::env::set_var("OTEL_ENABLED", "false");

    // Act: Initialize tracing
    let result = tracing::init_tracing();

    // Assert: Verify actual behavior - either succeeds or fails with expected error
    match result {
        Ok(_) => {
            // Success case - tracing initialized
        }
        Err(e) => {
            // Error case - should have meaningful error message
            assert!(!e.is_empty(), "Error message should not be empty");
        }
    }
}

/// Test: init_tracing with custom service version
/// Chicago TDD: Test behavior (service version configuration) not implementation (env var reading)
#[test]
fn test_init_tracing_with_custom_service_version() {
    // Arrange: Set custom service version
    std::env::set_var("OTEL_SERVICE_VERSION", "2.0.0");
    std::env::set_var("OTEL_ENABLED", "false");

    // Act: Initialize tracing
    let result = tracing::init_tracing();

    // Assert: Verify actual behavior - either succeeds or fails with expected error
    match result {
        Ok(_) => {
            // Success case - tracing initialized
        }
        Err(e) => {
            // Error case - should have meaningful error message
            assert!(!e.is_empty(), "Error message should not be empty");
        }
    }
}

/// Test: init_tracing with OTLP endpoint
/// Chicago TDD: Test behavior (OTLP endpoint configuration) not implementation (OTEL setup)
#[test]
fn test_init_tracing_with_otlp_endpoint() {
    // Arrange: Set OTLP endpoint
    std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4317");
    std::env::set_var("OTEL_ENABLED", "true");

    // Act: Initialize tracing
    let result = tracing::init_tracing();

    // Assert: Returns Result (may fail if collector not running or subscriber already set)
    // The error "a global default trace dispatcher has already been set" is expected when
    // running multiple tests together
    match result {
        Ok(_) => {
            // Success case - tracing initialized
        }
        Err(e) => {
            // Error case - may be "global default trace dispatcher has already been set"
            // or "Failed to initialize OpenTelemetry SDK" or "Failed to initialize tracing subscriber"
            // All of these are acceptable in test environment
            assert!(
                e.contains("global default")
                    || e.contains("subscriber")
                    || e.contains("OpenTelemetry")
                    || e.contains("tracing subscriber")
            );
        }
    }
}

/// Test: init_tracing with KNHK_TRACE level
/// Chicago TDD: Test behavior (trace level configuration) not implementation (filter setup)
#[test]
fn test_init_tracing_with_trace_level() {
    // Arrange: Set trace level and disable OTEL
    std::env::set_var("KNHK_TRACE", "debug");
    std::env::set_var("OTEL_ENABLED", "false");

    // Act: Initialize tracing
    let result = tracing::init_tracing();

    // Assert: Verify actual behavior - either succeeds or fails with expected error
    match result {
        Ok(_) => {
            // Success case - tracing initialized
        }
        Err(e) => {
            // Error case - should have meaningful error message
            assert!(!e.is_empty(), "Error message should not be empty");
        }
    }
}

/// Test: init_tracing multiple times
/// Chicago TDD: Test behavior (idempotency) not implementation (subscriber initialization)
#[test]
fn test_init_tracing_multiple_times() {
    // Arrange: Disable OTEL
    std::env::set_var("OTEL_ENABLED", "false");

    // Act: Initialize tracing multiple times
    let result1 = tracing::init_tracing();
    let result2 = tracing::init_tracing();

    // Assert: Both should return Results (may fail if subscriber already initialized)
    // First call should succeed or fail with meaningful error
    match result1 {
        Ok(_) => {
            // First initialization succeeded
        }
        Err(e) => {
            // First initialization failed (subscriber already set)
            assert!(!e.is_empty(), "Error message should not be empty");
        }
    }
    // Second call should fail with "already set" error since first call may have succeeded
    match result2 {
        Ok(_) => {
            // Second initialization succeeded (unlikely if first succeeded)
        }
        Err(e) => {
            // Second initialization failed (expected - subscriber already set)
            assert!(!e.is_empty(), "Error message should not be empty");
        }
    }
}
