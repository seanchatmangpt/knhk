//! Jaeger exporter
//!
//! Exports distributed traces to Jaeger for visualization and analysis.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::telemetry::{TelemetryEvent, TelemetryResult, TelemetryError, Span};
use super::Exporter;

/// Jaeger exporter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JaegerConfig {
    /// Jaeger agent endpoint (e.g., "localhost:6831")
    pub agent_endpoint: String,

    /// Service name
    pub service_name: String,

    /// Timeout for export requests
    pub timeout: Duration,

    /// Max packet size
    pub max_packet_size: usize,
}

impl Default for JaegerConfig {
    fn default() -> Self {
        Self {
            agent_endpoint: "localhost:6831".to_string(),
            service_name: "knhk-workflow-engine".to_string(),
            timeout: Duration::from_secs(10),
            max_packet_size: 65000,  // UDP packet size limit
        }
    }
}

/// Jaeger exporter
pub struct JaegerExporter {
    /// Configuration
    config: JaegerConfig,

    /// Export statistics
    stats: Arc<RwLock<ExportStats>>,
}

/// Export statistics
#[derive(Debug, Default, Clone)]
struct ExportStats {
    /// Total spans exported
    spans_exported: u64,

    /// Total export errors
    export_errors: u64,

    /// Total bytes sent
    bytes_sent: u64,

    /// Total traces exported
    traces_exported: u64,
}

impl JaegerExporter {
    /// Create a new Jaeger exporter
    pub fn new(config: JaegerConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(ExportStats::default())),
        }
    }

    /// Export spans to Jaeger
    async fn export_spans(&self, spans: Vec<&Span>) -> TelemetryResult<()> {
        if spans.is_empty() {
            return Ok(());
        }

        debug!("Exporting {} spans to Jaeger: {}", spans.len(), self.config.agent_endpoint);

        // Group spans by trace_id
        let mut traces: std::collections::HashMap<String, Vec<&Span>> = std::collections::HashMap::new();

        for span in &spans {
            traces.entry(span.trace_id.clone())
                .or_insert_with(Vec::new)
                .push(span);
        }

        // Export each trace
        for (trace_id, trace_spans) in traces {
            self.export_trace(&trace_id, &trace_spans).await?;
        }

        Ok(())
    }

    /// Export a single trace to Jaeger
    async fn export_trace(&self, trace_id: &str, spans: &[&Span]) -> TelemetryResult<()> {
        // In production, this would:
        // 1. Convert spans to Jaeger Thrift format
        // 2. Batch spans into UDP packets (respecting max_packet_size)
        // 3. Send to Jaeger agent via UDP

        // Simplified implementation - just serialize to JSON for demonstration
        let payload = serde_json::to_string(&spans)
            .map_err(|e| TelemetryError::ExportError(format!("Serialization failed: {}", e)))?;

        let bytes_sent = payload.len() as u64;

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.spans_exported += spans.len() as u64;
            stats.traces_exported += 1;
            stats.bytes_sent += bytes_sent;
        }

        debug!("Exported trace {} with {} spans ({} bytes)", trace_id, spans.len(), bytes_sent);

        Ok(())
    }

    /// Get export statistics
    pub fn stats(&self) -> ExportStats {
        self.stats.read().clone()
    }
}

#[async_trait]
impl Exporter for JaegerExporter {
    async fn export(&self, events: &[TelemetryEvent]) -> TelemetryResult<()> {
        // Extract only span events
        let spans: Vec<&Span> = events.iter()
            .filter_map(|e| match e {
                TelemetryEvent::Span(span) => Some(span),
                _ => None,
            })
            .collect();

        if !spans.is_empty() {
            self.export_spans(spans).await?;
        }

        Ok(())
    }

    async fn flush(&self) -> TelemetryResult<()> {
        debug!("Flushing Jaeger exporter");
        Ok(())
    }

    async fn shutdown(&self) -> TelemetryResult<()> {
        debug!("Shutting down Jaeger exporter");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::{SpanStatus, AttributeValue};

    #[tokio::test]
    async fn test_jaeger_exporter_creation() {
        let config = JaegerConfig::default();
        let exporter = JaegerExporter::new(config);

        let stats = exporter.stats();
        assert_eq!(stats.spans_exported, 0);
    }

    #[tokio::test]
    async fn test_jaeger_span_export() {
        let config = JaegerConfig::default();
        let exporter = JaegerExporter::new(config);

        let span1 = Span {
            name: "test.span1".to_string(),
            trace_id: "trace-123".to_string(),
            span_id: "span-1".to_string(),
            parent_span_id: None,
            attributes: vec![
                ("service.name".to_string(), AttributeValue::String("test-service".to_string())),
            ],
            duration_ns: 1_000_000,
            status: SpanStatus::Ok,
            start_time_ns: 1000,
            end_time_ns: 2000,
        };

        let span2 = Span {
            name: "test.span2".to_string(),
            trace_id: "trace-123".to_string(),
            span_id: "span-2".to_string(),
            parent_span_id: Some("span-1".to_string()),
            attributes: vec![
                ("service.name".to_string(), AttributeValue::String("test-service".to_string())),
            ],
            duration_ns: 500_000,
            status: SpanStatus::Ok,
            start_time_ns: 1500,
            end_time_ns: 2000,
        };

        let events = vec![
            TelemetryEvent::Span(span1),
            TelemetryEvent::Span(span2),
        ];

        let result = exporter.export(&events).await;
        assert!(result.is_ok());

        let stats = exporter.stats();
        assert_eq!(stats.spans_exported, 2);
        assert_eq!(stats.traces_exported, 1);  // Both spans in same trace
    }

    #[tokio::test]
    async fn test_jaeger_multiple_traces() {
        let config = JaegerConfig::default();
        let exporter = JaegerExporter::new(config);

        let span1 = Span {
            name: "test.span1".to_string(),
            trace_id: "trace-123".to_string(),
            span_id: "span-1".to_string(),
            parent_span_id: None,
            attributes: vec![],
            duration_ns: 1_000_000,
            status: SpanStatus::Ok,
            start_time_ns: 1000,
            end_time_ns: 2000,
        };

        let span2 = Span {
            name: "test.span2".to_string(),
            trace_id: "trace-456".to_string(),  // Different trace
            span_id: "span-2".to_string(),
            parent_span_id: None,
            attributes: vec![],
            duration_ns: 500_000,
            status: SpanStatus::Ok,
            start_time_ns: 1500,
            end_time_ns: 2000,
        };

        let events = vec![
            TelemetryEvent::Span(span1),
            TelemetryEvent::Span(span2),
        ];

        let result = exporter.export(&events).await;
        assert!(result.is_ok());

        let stats = exporter.stats();
        assert_eq!(stats.spans_exported, 2);
        assert_eq!(stats.traces_exported, 2);  // Two separate traces
    }
}
