//! Telemetry exporters for OTLP, Prometheus, and Jaeger
//!
//! This module provides exporters for various telemetry backends.

use async_trait::async_trait;

use crate::telemetry::{TelemetryEvent, TelemetryResult};

pub mod otlp;
pub mod prometheus;
pub mod jaeger;

pub use otlp::OtlpExporter;
pub use prometheus::PrometheusExporter;
pub use jaeger::JaegerExporter;

/// Telemetry exporter trait
#[async_trait]
pub trait Exporter: Send + Sync {
    /// Export a batch of telemetry events
    async fn export(&self, events: &[TelemetryEvent]) -> TelemetryResult<()>;

    /// Flush any buffered events
    async fn flush(&self) -> TelemetryResult<()> {
        Ok(())
    }

    /// Shutdown the exporter
    async fn shutdown(&self) -> TelemetryResult<()> {
        Ok(())
    }
}
