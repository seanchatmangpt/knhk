//! Unified validation helpers for OTEL spans, metrics, and Weaver schema validation
//!
//! Provides comprehensive validation functions to ensure:
//! - OTEL spans and metrics conform to semantic conventions
//! - Telemetry conforms to Weaver schema requirements
//! - Runtime validation via Weaver live-check

use crate::{Metric, Span, SpanStatus};

#[cfg(feature = "std")]
use crate::WeaverLiveCheck;
#[cfg(feature = "std")]
use std::path::Path;

// ============================================================================
// OTEL Validation
// ============================================================================

/// Validation error
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Invalid span structure
    InvalidSpanStructure(String),
    /// Invalid metric structure
    InvalidMetricStructure(String),
    /// Invalid span timing
    InvalidSpanTiming(String),
    /// Invalid metric values
    InvalidMetricValues(String),
    /// Missing required attributes
    MissingAttributes(String),
    /// Invalid attribute types
    InvalidAttributeTypes(String),
    /// Schema file not found
    SchemaNotFound(String),
    /// Schema parsing error
    SchemaParseError(String),
    /// Telemetry does not conform to schema
    SchemaMismatch(String),
    /// Weaver binary not available
    WeaverNotAvailable(String),
    /// Weaver live-check failed
    LiveCheckFailed(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidSpanStructure(msg) => {
                write!(f, "Invalid span structure: {}", msg)
            }
            ValidationError::InvalidMetricStructure(msg) => {
                write!(f, "Invalid metric structure: {}", msg)
            }
            ValidationError::InvalidSpanTiming(msg) => {
                write!(f, "Invalid span timing: {}", msg)
            }
            ValidationError::InvalidMetricValues(msg) => {
                write!(f, "Invalid metric values: {}", msg)
            }
            ValidationError::MissingAttributes(msg) => {
                write!(f, "Missing required attributes: {}", msg)
            }
            ValidationError::InvalidAttributeTypes(msg) => {
                write!(f, "Invalid attribute types: {}", msg)
            }
            ValidationError::SchemaNotFound(path) => {
                write!(f, "Schema file not found: {}", path)
            }
            ValidationError::SchemaParseError(msg) => {
                write!(f, "Schema parsing error: {}", msg)
            }
            ValidationError::SchemaMismatch(msg) => {
                write!(f, "Telemetry does not conform to schema: {}", msg)
            }
            ValidationError::WeaverNotAvailable(msg) => {
                write!(f, "Weaver binary not available: {}", msg)
            }
            ValidationError::LiveCheckFailed(msg) => {
                write!(f, "Weaver live-check failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate span structure
///
/// Ensures span has required fields and valid structure.
pub fn validate_span_structure(span: &Span) -> Result<(), ValidationError> {
    // Check span name is not empty
    if span.name.is_empty() {
        return Err(ValidationError::InvalidSpanStructure(
            "Span name must not be empty".to_string(),
        ));
    }

    // Check span has valid context
    if span.context.trace_id.0 == 0 {
        return Err(ValidationError::InvalidSpanStructure(
            "Span trace_id must not be zero".to_string(),
        ));
    }

    if span.context.span_id.0 == 0 {
        return Err(ValidationError::InvalidSpanStructure(
            "Span span_id must not be zero".to_string(),
        ));
    }

    // Check timing: start_time must be set
    if span.start_time_ms == 0 {
        return Err(ValidationError::InvalidSpanStructure(
            "Span start_time_ms must be set".to_string(),
        ));
    }

    // If span is ended, end_time must be >= start_time
    if let Some(end_time) = span.end_time_ms {
        if end_time < span.start_time_ms {
            return Err(ValidationError::InvalidSpanTiming(format!(
                "Span end_time_ms {} < start_time_ms {}",
                end_time, span.start_time_ms
            )));
        }
    }

    Ok(())
}

/// Validate span attributes against schema
///
/// Validates that span has required attributes and attribute types match schema.
/// For now, performs basic validation. Full schema validation requires Weaver integration.
pub fn validate_span_attributes(
    span: &Span,
    required_attributes: &[&str],
) -> Result<(), ValidationError> {
    // Check required attributes are present
    for attr_name in required_attributes {
        if !span.attributes.contains_key(*attr_name) {
            return Err(ValidationError::MissingAttributes(format!(
                "Missing required attribute: {}",
                attr_name
            )));
        }
    }

    Ok(())
}

/// Validate span timing
///
/// Ensures span timing is valid (start_time set, end_time >= start_time if set).
pub fn validate_span_timing(span: &Span) -> Result<(), ValidationError> {
    // Check start_time is set
    if span.start_time_ms == 0 {
        return Err(ValidationError::InvalidSpanTiming(
            "Span start_time_ms must be set".to_string(),
        ));
    }

    // If span is ended, check end_time >= start_time
    if let Some(end_time) = span.end_time_ms {
        if end_time < span.start_time_ms {
            return Err(ValidationError::InvalidSpanTiming(format!(
                "Span end_time_ms {} < start_time_ms {}",
                end_time, span.start_time_ms
            )));
        }

        // Check duration is reasonable (not negative, not too large)
        let duration_ms = end_time - span.start_time_ms;
        if duration_ms > 86400000 {
            // More than 24 hours seems suspicious
            return Err(ValidationError::InvalidSpanTiming(format!(
                "Span duration {}ms exceeds 24 hours",
                duration_ms
            )));
        }
    }

    Ok(())
}

/// Validate metric structure
///
/// Ensures metric has required fields and valid structure.
pub fn validate_metric_structure(metric: &Metric) -> Result<(), ValidationError> {
    // Check metric name is not empty
    if metric.name.is_empty() {
        return Err(ValidationError::InvalidMetricStructure(
            "Metric name must not be empty".to_string(),
        ));
    }

    // Check timestamp is set
    if metric.timestamp_ms == 0 {
        return Err(ValidationError::InvalidMetricStructure(
            "Metric timestamp_ms must be set".to_string(),
        ));
    }

    Ok(())
}

/// Validate metric values
///
/// Ensures metric values are valid (non-negative counters, valid gauges, etc.).
pub fn validate_metric_values(metric: &Metric) -> Result<(), ValidationError> {
    match &metric.value {
        crate::MetricValue::Counter(count) => {
            // Counters must be non-negative
            // Note: u64 is always non-negative, but check for overflow concerns
            if *count > u64::MAX / 2 {
                // Very large counter might indicate overflow
                return Err(ValidationError::InvalidMetricValues(format!(
                    "Counter value {} is suspiciously large (possible overflow)",
                    count
                )));
            }
        }
        crate::MetricValue::Gauge(value) => {
            // Gauges can be negative, but check for NaN/Inf
            if value.is_nan() {
                return Err(ValidationError::InvalidMetricValues(
                    "Gauge value is NaN".to_string(),
                ));
            }
            if value.is_infinite() {
                return Err(ValidationError::InvalidMetricValues(
                    "Gauge value is infinite".to_string(),
                ));
            }
        }
        crate::MetricValue::Histogram(buckets) => {
            // Histogram buckets must be non-empty
            if buckets.is_empty() {
                return Err(ValidationError::InvalidMetricValues(
                    "Histogram buckets must not be empty".to_string(),
                ));
            }
            // Check buckets are non-negative and sorted
            for (i, bucket) in buckets.iter().enumerate() {
                if *bucket > u64::MAX / 2 {
                    return Err(ValidationError::InvalidMetricValues(format!(
                        "Histogram bucket {} value {} is suspiciously large",
                        i, bucket
                    )));
                }
                if i > 0 && buckets[i - 1] > *bucket {
                    return Err(ValidationError::InvalidMetricValues(format!(
                        "Histogram buckets not sorted: bucket {} ({}) > bucket {} ({})",
                        i - 1,
                        buckets[i - 1],
                        i,
                        bucket
                    )));
                }
            }
        }
    }

    Ok(())
}

// ============================================================================
// Weaver Validation
// ============================================================================

/// Telemetry data for validation
#[derive(Debug, Clone)]
pub struct Telemetry {
    /// Spans to validate
    pub spans: Vec<Span>,
    /// Metrics to validate
    pub metrics: Vec<Metric>,
}

/// Validate telemetry against Weaver schema
///
/// Validates that telemetry conforms to the Weaver schema at the given registry path.
/// This is a basic validation - full validation requires Weaver live-check.
///
/// # Arguments
/// * `telemetry` - Telemetry data to validate
/// * `registry_path` - Path to Weaver registry directory
///
/// # Returns
/// * `Ok(())` if telemetry is valid
/// * `Err(ValidationError)` if validation fails
#[cfg(feature = "std")]
pub fn validate_telemetry_against_schema(
    telemetry: &Telemetry,
    registry_path: &Path,
) -> Result<(), ValidationError> {
    // Check registry path exists
    if !registry_path.exists() {
        return Err(ValidationError::SchemaNotFound(
            registry_path.display().to_string(),
        ));
    }

    if !registry_path.is_dir() {
        return Err(ValidationError::SchemaNotFound(format!(
            "Registry path is not a directory: {}",
            registry_path.display()
        )));
    }

    // Basic validation: check spans and metrics have required fields
    for span in &telemetry.spans {
        if span.name.is_empty() {
            return Err(ValidationError::SchemaMismatch(
                "Span name must not be empty".to_string(),
            ));
        }
        if span.context.trace_id.0 == 0 {
            return Err(ValidationError::SchemaMismatch(
                "Span trace_id must not be zero".to_string(),
            ));
        }
    }

    for metric in &telemetry.metrics {
        if metric.name.is_empty() {
            return Err(ValidationError::SchemaMismatch(
                "Metric name must not be empty".to_string(),
            ));
        }
    }

    // Note: Full schema validation requires Weaver binary and live-check
    // This function performs basic structural validation only
    Ok(())
}

/// Validate Weaver live-check
///
/// Validates that Weaver live-check can be executed and is healthy.
/// This checks if Weaver binary is available and can validate telemetry.
///
/// # Arguments
/// * `registry_path` - Path to Weaver registry directory
///
/// # Returns
/// * `Ok(())` if Weaver live-check is available and healthy
/// * `Err(ValidationError)` if validation fails
#[cfg(feature = "std")]
pub fn validate_weaver_live_check(registry_path: &Path) -> Result<(), ValidationError> {
    // Check Weaver binary is available
    match WeaverLiveCheck::check_weaver_available() {
        Ok(()) => {}
        Err(e) => {
            return Err(ValidationError::WeaverNotAvailable(e));
        }
    }

    // Check registry path exists
    if !registry_path.exists() {
        return Err(ValidationError::SchemaNotFound(
            registry_path.display().to_string(),
        ));
    }

    // Create Weaver live-check instance
    let live_check = WeaverLiveCheck::new()
        .with_registry(
            registry_path
                .to_str()
                .ok_or_else(|| {
                    ValidationError::SchemaParseError(
                        "Registry path is not valid UTF-8".to_string(),
                    )
                })?
                .to_string(),
        )
        .with_otlp_port(4317)
        .with_admin_port(8080)
        .with_inactivity_timeout(300)
        .with_format("json".to_string())
        .with_output("./weaver-reports".to_string());

    // Check health (this validates Weaver can start and validate telemetry)
    match live_check.check_health() {
        Ok(true) => Ok(()),
        Ok(false) => Err(ValidationError::LiveCheckFailed(
            "Weaver live-check health check returned false".to_string(),
        )),
        Err(e) => Err(ValidationError::LiveCheckFailed(format!(
            "Weaver health check error: {}",
            e
        ))),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SpanContext, SpanId, SpanStatus, TraceId};

    #[test]
    fn test_validate_span_structure() {
        // Valid span
        let span = Span {
            context: SpanContext {
                trace_id: TraceId(12345),
                span_id: SpanId(67890),
                parent_span_id: None,
                flags: 0,
            },
            name: "test.span".to_string(),
            start_time_ms: 1000,
            end_time_ms: Some(2000),
            attributes: Default::default(),
            events: Vec::new(),
            status: SpanStatus::Ok,
        };
        assert!(validate_span_structure(&span).is_ok());

        // Invalid: empty name
        let mut span = span.clone();
        span.name = String::new();
        assert!(validate_span_structure(&span).is_err());

        // Invalid: zero trace_id
        let mut span = span.clone();
        span.name = "test.span".to_string();
        span.context.trace_id = TraceId(0);
        assert!(validate_span_structure(&span).is_err());

        // Invalid: end_time < start_time
        let mut span = span.clone();
        span.context.trace_id = TraceId(12345);
        span.end_time_ms = Some(500);
        assert!(validate_span_timing(&span).is_err());
    }

    #[test]
    fn test_validate_span_timing() {
        let span = Span {
            context: SpanContext {
                trace_id: TraceId(12345),
                span_id: SpanId(67890),
                parent_span_id: None,
                flags: 0,
            },
            name: "test.span".to_string(),
            start_time_ms: 1000,
            end_time_ms: Some(2000),
            attributes: Default::default(),
            events: Vec::new(),
            status: SpanStatus::Ok,
        };
        assert!(validate_span_timing(&span).is_ok());

        // Invalid: zero start_time
        let mut span = span.clone();
        span.start_time_ms = 0;
        assert!(validate_span_timing(&span).is_err());
    }

    #[test]
    fn test_validate_metric_structure() {
        // Valid counter
        let metric = Metric {
            name: "test.counter".to_string(),
            value: crate::MetricValue::Counter(100),
            timestamp_ms: 1000,
            attributes: Default::default(),
        };
        assert!(validate_metric_structure(&metric).is_ok());
        assert!(validate_metric_values(&metric).is_ok());

        // Invalid: empty name
        let mut metric = metric.clone();
        metric.name = String::new();
        assert!(validate_metric_structure(&metric).is_err());

        // Invalid: zero timestamp
        let mut metric = metric.clone();
        metric.name = "test.counter".to_string();
        metric.timestamp_ms = 0;
        assert!(validate_metric_structure(&metric).is_err());
    }

    #[test]
    fn test_validate_metric_values() {
        // Valid gauge
        let metric = Metric {
            name: "test.gauge".to_string(),
            value: crate::MetricValue::Gauge(42.5),
            timestamp_ms: 1000,
            attributes: Default::default(),
        };
        assert!(validate_metric_values(&metric).is_ok());

        // Invalid: NaN gauge
        let mut metric = metric.clone();
        metric.value = crate::MetricValue::Gauge(f64::NAN);
        assert!(validate_metric_values(&metric).is_err());

        // Invalid: empty histogram
        let mut metric = metric.clone();
        metric.value = crate::MetricValue::Histogram(Vec::new());
        assert!(validate_metric_values(&metric).is_err());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_validate_telemetry_against_schema() {
        let telemetry = Telemetry {
            spans: vec![Span {
                context: SpanContext {
                    trace_id: TraceId(12345),
                    span_id: SpanId(67890),
                    parent_span_id: None,
                    flags: 0,
                },
                name: "test.span".to_string(),
                start_time_ms: 1000,
                end_time_ms: Some(2000),
                attributes: Default::default(),
                events: Vec::new(),
                status: SpanStatus::Ok,
            }],
            metrics: vec![],
        };

        // Test with non-existent path (should fail)
        let result = validate_telemetry_against_schema(&telemetry, Path::new("/nonexistent"));
        assert!(result.is_err());

        // Test with valid telemetry structure (basic validation passes)
        // Note: Full validation requires actual registry path
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_validate_telemetry_empty_span_name() {
        let telemetry = Telemetry {
            spans: vec![Span {
                context: SpanContext {
                    trace_id: TraceId(12345),
                    span_id: SpanId(67890),
                    parent_span_id: None,
                    flags: 0,
                },
                name: String::new(), // Empty name
                start_time_ms: 1000,
                end_time_ms: Some(2000),
                attributes: Default::default(),
                events: Vec::new(),
                status: SpanStatus::Ok,
            }],
            metrics: vec![],
        };

        // Should fail validation
        let result = validate_telemetry_against_schema(&telemetry, Path::new("/tmp"));
        assert!(result.is_err());
    }
}
