//! Chicago TDD tests for OpenTelemetry core types
//!
//! Following Chicago TDD principles:
//! - AAA pattern (Arrange, Act, Assert)
//! - Test behavior, not implementation
//! - Verify outcomes and state changes
//! - Use real collaborators when possible

#![cfg(feature = "std")]

use knhk_otel::{
    generate_span_id, get_timestamp_ms, Metric, MetricValue, Span, SpanContext, SpanEvent, SpanId,
    SpanStatus, TraceId,
};

/// Test: generate_span_id returns valid 64-bit span ID
/// Chicago TDD: Test behavior (ID generation) not implementation (random generation)
#[test]
fn test_generate_span_id_returns_valid_id() {
    // Arrange: No setup needed

    // Act: Generate span ID
    let span_id = generate_span_id();

    // Assert: Span ID is valid (non-zero, 64-bit)
    assert_ne!(span_id, 0, "Span ID should be non-zero");
    // 64-bit value is always valid, no upper bound check needed
}

/// Test: generate_span_id returns different IDs on each call
/// Chicago TDD: Test behavior (uniqueness) not implementation (randomness)
#[test]
fn test_generate_span_id_returns_unique_ids() {
    // Arrange: Generate multiple span IDs
    let id1 = generate_span_id();
    let id2 = generate_span_id();
    let id3 = generate_span_id();

    // Act: Check uniqueness
    // Note: Very unlikely but possible to get duplicates with random generation
    // We verify that at least 2 out of 3 are different (high probability)
    let unique_count = [id1, id2, id3]
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len();
    assert!(
        unique_count >= 2,
        "At least 2 out of 3 span IDs should be unique"
    );
}

/// Test: get_timestamp_ms returns valid timestamp
/// Chicago TDD: Test behavior (timestamp generation) not implementation (system time)
#[test]
fn test_get_timestamp_ms_returns_valid_timestamp() {
    // Arrange: Get current timestamp
    let timestamp = get_timestamp_ms();

    // Act: Verify timestamp is reasonable (after Unix epoch)
    // Assert: Timestamp should be > 0 (Unix epoch in milliseconds)
    assert!(timestamp > 0, "Timestamp should be positive");
    // Timestamp should be reasonable (not too far in future)
    let max_reasonable = 10_000_000_000_000; // Year 2286 in milliseconds
    assert!(
        timestamp < max_reasonable,
        "Timestamp should be reasonable (not too far in future)"
    );
}

/// Test: get_timestamp_ms returns increasing timestamps
/// Chicago TDD: Test behavior (monotonicity) not implementation (time source)
#[test]
fn test_get_timestamp_ms_returns_increasing_timestamps() {
    // Arrange: Get first timestamp
    let timestamp1 = get_timestamp_ms();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let timestamp2 = get_timestamp_ms();

    // Act: Verify timestamps are increasing
    // Assert: Second timestamp should be >= first (allowing for clock adjustments)
    assert!(
        timestamp2 >= timestamp1,
        "Timestamps should be monotonically increasing"
    );
}

/// Test: TraceId creation and equality
/// Chicago TDD: Test behavior (ID creation and comparison) not implementation (storage)
#[test]
fn test_trace_id_creation_and_equality() {
    // Arrange: Create trace IDs
    let trace_id1 = TraceId(12345678901234567890123456789012345678u128);
    let trace_id2 = TraceId(12345678901234567890123456789012345678u128);
    let trace_id3 = TraceId(98765432109876543210987654321098765432u128);

    // Act: Compare trace IDs
    // Assert: Equal trace IDs should be equal
    assert_eq!(trace_id1, trace_id2, "Equal trace IDs should be equal");
    // Assert: Different trace IDs should not be equal
    assert_ne!(
        trace_id1, trace_id3,
        "Different trace IDs should not be equal"
    );
}

/// Test: SpanId creation and equality
/// Chicago TDD: Test behavior (ID creation and comparison) not implementation (storage)
#[test]
fn test_span_id_creation_and_equality() {
    // Arrange: Create span IDs
    let span_id1 = SpanId(12345678901234567890u64);
    let span_id2 = SpanId(12345678901234567890u64);
    let span_id3 = SpanId(9876543210987654321u64);

    // Act: Compare span IDs
    // Assert: Equal span IDs should be equal
    assert_eq!(span_id1, span_id2, "Equal span IDs should be equal");
    // Assert: Different span IDs should not be equal
    assert_ne!(span_id1, span_id3, "Different span IDs should not be equal");
}

/// Test: SpanContext creation with trace and span IDs
/// Chicago TDD: Test behavior (context creation) not implementation (field storage)
#[test]
fn test_span_context_creation() {
    // Arrange: Create trace and span IDs
    let trace_id = TraceId(12345678901234567890123456789012345678u128);
    let span_id = SpanId(12345678901234567890u64);

    // Act: Create span context
    let context = SpanContext {
        trace_id,
        span_id,
        parent_span_id: None,
        flags: 1,
    };

    // Assert: Context has correct values
    assert_eq!(
        context.trace_id, trace_id,
        "Context should have correct trace ID"
    );
    assert_eq!(
        context.span_id, span_id,
        "Context should have correct span ID"
    );
    assert_eq!(
        context.parent_span_id, None,
        "Context should have no parent span ID"
    );
    assert_eq!(context.flags, 1, "Context should have correct flags");
}

/// Test: SpanContext with parent span ID
/// Chicago TDD: Test behavior (parent relationship) not implementation (optional field)
#[test]
fn test_span_context_with_parent() {
    // Arrange: Create parent and child span IDs
    let trace_id = TraceId(12345678901234567890123456789012345678u128);
    let parent_span_id = SpanId(1111111111111111111u64);
    let child_span_id = SpanId(2222222222222222222u64);

    // Act: Create child context with parent
    let child_context = SpanContext {
        trace_id,
        span_id: child_span_id,
        parent_span_id: Some(parent_span_id),
        flags: 1,
    };

    // Assert: Child context has parent reference
    assert_eq!(
        child_context.parent_span_id,
        Some(parent_span_id),
        "Child context should reference parent span ID"
    );
    assert_eq!(
        child_context.trace_id, trace_id,
        "Child context should share trace ID with parent"
    );
}

/// Test: SpanStatus enum values
/// Chicago TDD: Test behavior (status values) not implementation (enum storage)
#[test]
fn test_span_status_enum_values() {
    // Arrange: Create spans with different statuses
    let status_ok = SpanStatus::Ok;
    let status_error = SpanStatus::Error;
    let status_unset = SpanStatus::Unset;

    // Act: Verify status values
    // Assert: Statuses are distinct
    assert_ne!(status_ok, status_error, "Ok and Error should be different");
    assert_ne!(status_ok, status_unset, "Ok and Unset should be different");
    assert_ne!(
        status_error, status_unset,
        "Error and Unset should be different"
    );
}

/// Test: SpanEvent creation
/// Chicago TDD: Test behavior (event creation) not implementation (field storage)
#[test]
fn test_span_event_creation() {
    // Arrange: Create event data
    let event_name = "operation.started".to_string();
    let timestamp = get_timestamp_ms();
    let mut attributes = std::collections::BTreeMap::new();
    attributes.insert("event.type".to_string(), "start".to_string());

    // Act: Create span event
    let event = SpanEvent {
        name: event_name.clone(),
        timestamp_ms: timestamp,
        attributes: attributes.clone(),
    };

    // Assert: Event has correct values
    assert_eq!(event.name, event_name, "Event should have correct name");
    assert_eq!(
        event.timestamp_ms, timestamp,
        "Event should have correct timestamp"
    );
    assert_eq!(
        event.attributes, attributes,
        "Event should have correct attributes"
    );
}

/// Test: Span creation and fields
/// Chicago TDD: Test behavior (span creation) not implementation (field storage)
#[test]
fn test_span_creation() {
    // Arrange: Create span context and data
    let context = SpanContext {
        trace_id: TraceId(123456789012345678901234567890123456789u128),
        span_id: SpanId(12345678901234567890u64),
        parent_span_id: None,
        flags: 1,
    };
    let span_name = "knhk.operation.execute".to_string();
    let start_time = get_timestamp_ms();

    // Act: Create span
    let span = Span {
        context: context.clone(),
        name: span_name.clone(),
        start_time_ms: start_time,
        end_time_ms: None,
        attributes: std::collections::BTreeMap::new(),
        events: Vec::new(),
        status: SpanStatus::Unset,
    };

    // Assert: Span has correct values
    assert_eq!(
        span.context.trace_id, context.trace_id,
        "Span should have correct trace ID"
    );
    assert_eq!(
        span.context.span_id, context.span_id,
        "Span should have correct span ID"
    );
    assert_eq!(
        span.context.parent_span_id, context.parent_span_id,
        "Span should have correct parent span ID"
    );
    assert_eq!(span.name, span_name, "Span should have correct name");
    assert_eq!(
        span.start_time_ms, start_time,
        "Span should have correct start time"
    );
    assert_eq!(
        span.end_time_ms, None,
        "Span should have no end time initially"
    );
    assert_eq!(
        span.attributes.len(),
        0,
        "Span should have no attributes initially"
    );
    assert_eq!(span.events.len(), 0, "Span should have no events initially");
    assert_eq!(
        span.status,
        SpanStatus::Unset,
        "Span should have Unset status initially"
    );
}

/// Test: Span with end time and status
/// Chicago TDD: Test behavior (span completion) not implementation (field updates)
#[test]
fn test_span_with_end_time_and_status() {
    // Arrange: Create span
    let context = SpanContext {
        trace_id: TraceId(12345678901234567890123456789012345678u128),
        span_id: SpanId(12345678901234567890u64),
        parent_span_id: None,
        flags: 1,
    };
    let start_time = get_timestamp_ms();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let end_time = get_timestamp_ms();

    // Act: Create completed span
    let span = Span {
        context: context.clone(),
        name: "knhk.operation.execute".to_string(),
        start_time_ms: start_time,
        end_time_ms: Some(end_time),
        attributes: std::collections::BTreeMap::new(),
        events: Vec::new(),
        status: SpanStatus::Ok,
    };

    // Assert: Span has end time and status
    assert_eq!(
        span.end_time_ms,
        Some(end_time),
        "Span should have end time"
    );
    assert!(
        span.end_time_ms.unwrap() >= span.start_time_ms,
        "End time should be >= start time"
    );
    assert_eq!(span.status, SpanStatus::Ok, "Span should have Ok status");
}

/// Test: MetricValue Counter variant
/// Chicago TDD: Test behavior (counter value) not implementation (enum storage)
#[test]
fn test_metric_value_counter() {
    // Arrange: Create counter value
    let counter_value = MetricValue::Counter(42);

    // Act: Verify counter value
    // Assert: Counter has correct value
    match counter_value {
        MetricValue::Counter(count) => assert_eq!(count, 42, "Counter should have correct value"),
        _ => panic!("Expected Counter variant"),
    }
}

/// Test: MetricValue Gauge variant
/// Chicago TDD: Test behavior (gauge value) not implementation (enum storage)
#[test]
fn test_metric_value_gauge() {
    // Arrange: Create gauge value
    let gauge_value = MetricValue::Gauge(3.14);

    // Act: Verify gauge value
    // Assert: Gauge has correct value
    match gauge_value {
        MetricValue::Gauge(value) => {
            assert!(
                (value - 3.14).abs() < 0.001,
                "Gauge should have correct value"
            )
        }
        _ => panic!("Expected Gauge variant"),
    }
}

/// Test: MetricValue Histogram variant
/// Chicago TDD: Test behavior (histogram value) not implementation (enum storage)
#[test]
fn test_metric_value_histogram() {
    // Arrange: Create histogram value
    let histogram_value = MetricValue::Histogram(vec![10, 20, 30, 40, 50]);

    // Act: Verify histogram value
    // Assert: Histogram has correct values
    match histogram_value {
        MetricValue::Histogram(buckets) => {
            assert_eq!(buckets.len(), 5, "Histogram should have 5 buckets");
            assert_eq!(buckets[0], 10, "First bucket should be 10");
            assert_eq!(buckets[4], 50, "Last bucket should be 50");
        }
        _ => panic!("Expected Histogram variant"),
    }
}

/// Test: Metric creation
/// Chicago TDD: Test behavior (metric creation) not implementation (field storage)
#[test]
fn test_metric_creation() {
    // Arrange: Create metric data
    let metric_name = "knhk.operation.executed".to_string();
    let metric_value = MetricValue::Counter(1);
    let timestamp = get_timestamp_ms();
    let mut attributes = std::collections::BTreeMap::new();
    attributes.insert("operation".to_string(), "test.operation".to_string());

    // Act: Create metric
    let metric = Metric {
        name: metric_name.clone(),
        value: metric_value.clone(),
        timestamp_ms: timestamp,
        attributes: attributes.clone(),
    };

    // Assert: Metric has correct values
    assert_eq!(metric.name, metric_name, "Metric should have correct name");
    match (&metric.value, &metric_value) {
        (MetricValue::Counter(a), MetricValue::Counter(b)) => assert_eq!(a, b),
        _ => panic!("Metric values should match"),
    }
    assert_eq!(
        metric.timestamp_ms, timestamp,
        "Metric should have correct timestamp"
    );
    assert_eq!(
        metric.attributes, attributes,
        "Metric should have correct attributes"
    );
}
