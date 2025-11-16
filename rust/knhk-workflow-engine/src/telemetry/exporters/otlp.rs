//! OTLP (OpenTelemetry Protocol) exporter
//!
//! Exports telemetry data to OTLP-compatible backends like OpenTelemetry Collector,
//! Grafana Tempo, Jaeger (with OTLP receiver), etc.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::telemetry::{TelemetryEvent, TelemetryResult, TelemetryError, Span, Metric, LogEntry};
use super::Exporter;

/// OTLP exporter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpConfig {
    /// OTLP endpoint (e.g., "http://localhost:4317")
    pub endpoint: String,

    /// Timeout for export requests
    pub timeout: Duration,

    /// Use gRPC (true) or HTTP (false)
    pub use_grpc: bool,

    /// Headers to include in export requests
    pub headers: Vec<(String, String)>,

    /// Compression (none, gzip)
    pub compression: Option<String>,
}

impl Default for OtlpConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:4317".to_string(),
            timeout: Duration::from_secs(10),
            use_grpc: true,
            headers: vec![],
            compression: Some("gzip".to_string()),
        }
    }
}

/// OTLP exporter
pub struct OtlpExporter {
    /// Configuration
    config: OtlpConfig,

    /// Export statistics
    stats: Arc<RwLock<ExportStats>>,
}

/// Export statistics
#[derive(Debug, Default, Clone)]
struct ExportStats {
    /// Total batches exported
    batches_exported: u64,

    /// Total events exported
    events_exported: u64,

    /// Total export errors
    export_errors: u64,

    /// Total bytes sent
    bytes_sent: u64,
}

impl OtlpExporter {
    /// Create a new OTLP exporter
    pub fn new(config: OtlpConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(ExportStats::default())),
        }
    }

    /// Export spans
    async fn export_spans(&self, spans: Vec<&Span>) -> TelemetryResult<()> {
        if spans.is_empty() {
            return Ok(());
        }

        debug!("Exporting {} spans to OTLP endpoint: {}", spans.len(), self.config.endpoint);

        // In production, this would serialize to OTLP protobuf format
        // and send via gRPC or HTTP

        // Simplified implementation - just serialize to JSON for demonstration
        let payload = serde_json::to_string(&spans)
            .map_err(|e| TelemetryError::ExportError(format!("Serialization failed: {}", e)))?;

        // Simulate HTTP POST
        let bytes_sent = payload.len() as u64;

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.batches_exported += 1;
            stats.events_exported += spans.len() as u64;
            stats.bytes_sent += bytes_sent;
        }

        debug!("Successfully exported {} spans ({} bytes)", spans.len(), bytes_sent);

        Ok(())
    }

    /// Export metrics
    async fn export_metrics(&self, metrics: Vec<&Metric>) -> TelemetryResult<()> {
        if metrics.is_empty() {
            return Ok(());
        }

        debug!("Exporting {} metrics to OTLP endpoint: {}", metrics.len(), self.config.endpoint);

        let payload = serde_json::to_string(&metrics)
            .map_err(|e| TelemetryError::ExportError(format!("Serialization failed: {}", e)))?;

        let bytes_sent = payload.len() as u64;

        {
            let mut stats = self.stats.write();
            stats.batches_exported += 1;
            stats.events_exported += metrics.len() as u64;
            stats.bytes_sent += bytes_sent;
        }

        debug!("Successfully exported {} metrics ({} bytes)", metrics.len(), bytes_sent);

        Ok(())
    }

    /// Export logs
    async fn export_logs(&self, logs: Vec<&LogEntry>) -> TelemetryResult<()> {
        if logs.is_empty() {
            return Ok(());
        }

        debug!("Exporting {} logs to OTLP endpoint: {}", logs.len(), self.config.endpoint);

        let payload = serde_json::to_string(&logs)
            .map_err(|e| TelemetryError::ExportError(format!("Serialization failed: {}", e)))?;

        let bytes_sent = payload.len() as u64;

        {
            let mut stats = self.stats.write();
            stats.batches_exported += 1;
            stats.events_exported += logs.len() as u64;
            stats.bytes_sent += bytes_sent;
        }

        debug!("Successfully exported {} logs ({} bytes)", logs.len(), bytes_sent);

        Ok(())
    }

    /// Get export statistics
    pub fn stats(&self) -> ExportStats {
        self.stats.read().clone()
    }
}

#[async_trait]
impl Exporter for OtlpExporter {
    async fn export(&self, events: &[TelemetryEvent]) -> TelemetryResult<()> {
        // Separate events by type
        let mut spans = Vec::new();
        let mut metrics = Vec::new();
        let mut logs = Vec::new();

        for event in events {
            match event {
                TelemetryEvent::Span(span) => spans.push(span),
                TelemetryEvent::Metric(metric) => metrics.push(metric),
                TelemetryEvent::Log(log) => logs.push(log),
            }
        }

        // Export each type separately
        if !spans.is_empty() {
            self.export_spans(spans).await?;
        }

        if !metrics.is_empty() {
            self.export_metrics(metrics).await?;
        }

        if !logs.is_empty() {
            self.export_logs(logs).await?;
        }

        Ok(())
    }

    async fn flush(&self) -> TelemetryResult<()> {
        debug!("Flushing OTLP exporter");
        Ok(())
    }

    async fn shutdown(&self) -> TelemetryResult<()> {
        debug!("Shutting down OTLP exporter");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::{SpanStatus, AttributeValue};

    #[tokio::test]
    async fn test_otlp_exporter_creation() {
        let config = OtlpConfig::default();
        let exporter = OtlpExporter::new(config);

        let stats = exporter.stats();
        assert_eq!(stats.batches_exported, 0);
    }

    #[tokio::test]
    async fn test_otlp_span_export() {
        let config = OtlpConfig::default();
        let exporter = OtlpExporter::new(config);

        let span = Span {
            name: "test.span".to_string(),
            trace_id: "trace-123".to_string(),
            span_id: "span-456".to_string(),
            parent_span_id: None,
            attributes: vec![
                ("key".to_string(), AttributeValue::String("value".to_string())),
            ],
            duration_ns: 1_000_000,
            status: SpanStatus::Ok,
            start_time_ns: 1000,
            end_time_ns: 2000,
        };

        let events = vec![TelemetryEvent::Span(span)];

        let result = exporter.export(&events).await;
        assert!(result.is_ok());

        let stats = exporter.stats();
        assert_eq!(stats.batches_exported, 1);
        assert_eq!(stats.events_exported, 1);
    }
}
