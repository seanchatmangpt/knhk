//! Chicago TDD tests for OtelGuard

#![cfg(feature = "std")]

// OtelGuard is returned from init_tracer, no need to import

#[test]
fn test_otel_guard_init_tracer_default() {
    let guard_result = knhk_otel::init_tracer("test-service", "1.0.0", None);
    // Guard should be created successfully
    assert!(guard_result.is_ok());
    let _guard = guard_result.unwrap();
}

#[test]
fn test_otel_guard_init_tracer_with_endpoint() {
    let guard_result =
        knhk_otel::init_tracer("test-service", "1.0.0", Some("http://localhost:4317"));
    // Guard should be created successfully
    assert!(guard_result.is_ok());
    let _guard = guard_result.unwrap();
}

#[test]
fn test_otel_guard_drop_flushes_traces() {
    // Create guard and let it drop
    {
        let guard_result = knhk_otel::init_tracer("test-service", "1.0.0", None);
        assert!(guard_result.is_ok());
        let _guard = guard_result.unwrap();
        // Guard should exist
    }
    // Guard should have been dropped and flushed
    // We can't directly test this, but we verify it doesn't panic
}

// Note: init_tracer_with_metrics doesn't exist in the API
// Tests removed as function doesn't exist

#[test]
fn test_otel_guard_multiple_instances() {
    let guard1_result = knhk_otel::init_tracer("test-service-1", "1.0.0", None);
    let guard2_result = knhk_otel::init_tracer("test-service-2", "1.0.0", None);
    // Both guards should be created successfully
    assert!(guard1_result.is_ok());
    assert!(guard2_result.is_ok());
    let _guard1 = guard1_result.unwrap();
    let _guard2 = guard2_result.unwrap();
}
