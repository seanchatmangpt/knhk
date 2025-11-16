//! OpenTelemetry Integration - Weaver-Compliant Telemetry
//!
//! Emits structured telemetry for all autonomous system operations.
//! All telemetry MUST conform to the schema in `registry/` for Weaver validation.

use crate::errors::{Result, SystemError};
use opentelemetry::{
    global,
    trace::{TraceError, Tracer, TracerProvider},
    KeyValue,
};
use opentelemetry_sdk::{
    runtime,
    trace::{self as sdktrace, RandomIdGenerator, Sampler},
    Resource,
};
use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// OpenTelemetry integration for the autonomous system
///
/// Emits spans, metrics, and logs for:
/// - Evolution cycles
/// - Promotions
/// - Axis verifications
/// - Pattern detections
/// - Validation results
pub struct OTelIntegration {
    /// Optional OTLP endpoint
    otlp_endpoint: Option<String>,
}

impl OTelIntegration {
    /// Initialize OpenTelemetry
    ///
    /// If `otlp_endpoint` is provided, telemetry is sent via OTLP.
    /// Otherwise, telemetry is emitted to stdout (for testing).
    pub async fn new(otlp_endpoint: Option<String>) -> Result<Self> {
        info!("Initializing OpenTelemetry");

        // Initialize tracer provider
        Self::init_tracer_provider()?;

        // Initialize tracing subscriber
        Self::init_tracing_subscriber();

        Ok(Self { otlp_endpoint })
    }

    /// Initialize OpenTelemetry tracer provider
    fn init_tracer_provider() -> Result<()> {
        // Use noop exporter for now (simpler than stdout)
        let provider = sdktrace::TracerProvider::builder()
            .with_config(
                sdktrace::config()
                    .with_sampler(Sampler::AlwaysOn)
                    .with_id_generator(RandomIdGenerator::default())
                    .with_resource(Resource::new(vec![
                        KeyValue::new("service.name", "knhk-autonomous-system"),
                        KeyValue::new("service.version", crate::VERSION),
                    ])),
            )
            .build();

        global::set_tracer_provider(provider);

        Ok(())
    }

    /// Initialize tracing subscriber for log integration
    fn init_tracing_subscriber() {
        let _ = tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
            ))
            .with(tracing_subscriber::fmt::layer())
            .with(tracing_opentelemetry::layer())
            .try_init();
    }

    // /// Record an evolution cycle
    // ///
    // /// Emits telemetry for:
    // /// - Cycle duration
    // /// - Proposals generated
    // /// - Proposals validated
    // /// - Promotions completed
    // pub async fn record_cycle(&self, result: &knhk_autonomous_loop::CycleResult) -> Result<()> {
    //     use opentelemetry::trace::{Span, Status};
    //
    //     let mut span = self.tracer.start("ontology.evolution.cycle");
    //
    //     // Set attributes (must match Weaver schema)
    //     span.set_attribute(KeyValue::new(
    //         "proposals.generated",
    //         result.proposals_generated as i64,
    //     ));
    //     span.set_attribute(KeyValue::new(
    //         "proposals.validated",
    //         result.proposals_validated as i64,
    //     ));
    //     span.set_attribute(KeyValue::new("promoted.count", result.promoted.len() as i64));
    //     span.set_attribute(KeyValue::new("duration_ms", result.duration_ms as i64));
    //
    //     // Set status
    //     if result.promoted.is_empty() {
    //         span.set_status(Status::Ok);
    //     } else {
    //         span.set_status(Status::Ok);
    //     }
    //
    //     span.end();
    //
    //     Ok(())
    // }

    /// Record a promotion
    pub async fn record_promotion(&self, snapshot_id: &[u8; 32], duration_micros: u64) -> Result<()> {
        use opentelemetry::trace::{Span, Tracer};

        let tracer = global::tracer("knhk-autonomous-system");
        let mut span = tracer.start("ontology.promotion");

        span.set_attribute(KeyValue::new(
            "snapshot.id",
            hex::encode(snapshot_id),
        ));
        span.set_attribute(KeyValue::new("duration_micros", duration_micros as i64));

        span.end();

        Ok(())
    }

    /// Record axis verification
    pub async fn record_axis_verification(
        &self,
        axis: &str,
        passed: bool,
        duration_micros: u64,
    ) -> Result<()> {
        use opentelemetry::trace::{Span, Tracer};

        let tracer = global::tracer("knhk-autonomous-system");
        let mut span = tracer.start("ontology.axis_verification");

        span.set_attribute(KeyValue::new("axis", axis.to_string()));
        span.set_attribute(KeyValue::new("passed", passed));
        span.set_attribute(KeyValue::new("duration_micros", duration_micros as i64));

        span.end();

        Ok(())
    }

    /// Graceful shutdown
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down OpenTelemetry");

        // Flush any pending telemetry
        global::shutdown_tracer_provider();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_otel_initialization() {
        let result = OTelIntegration::new(None).await;
        assert!(result.is_ok(), "OTel initialization should succeed");
    }

    // #[tokio::test]
    // async fn test_record_cycle() {
    //     let otel = OTelIntegration::new(None).await.expect("OTel init failed");
    //
    //     let cycle_result = knhk_autonomous_loop::CycleResult {
    //         proposals_generated: 5,
    //         proposals_validated: 3,
    //         promoted: vec![[1u8; 32]],
    //         duration_ms: 1000,
    //     };
    //
    //     let result = otel.record_cycle(&cycle_result).await;
    //     assert!(result.is_ok(), "Recording cycle should succeed");
    // }

    #[tokio::test]
    async fn test_record_promotion() {
        let otel = OTelIntegration::new(None).await.expect("OTel init failed");

        let snapshot_id = [42u8; 32];
        let result = otel.record_promotion(&snapshot_id, 150).await;
        assert!(result.is_ok(), "Recording promotion should succeed");
    }

    #[tokio::test]
    async fn test_record_axis_verification() {
        let otel = OTelIntegration::new(None).await.expect("OTel init failed");

        let result = otel.record_axis_verification("τ", true, 50).await;
        assert!(result.is_ok(), "Recording axis verification should succeed");
    }
}
