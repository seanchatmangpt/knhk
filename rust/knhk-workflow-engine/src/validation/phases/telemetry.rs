//! Phase Telemetry - OpenTelemetry Integration
//!
//! Provides comprehensive OTEL telemetry for phase execution:
//! - Spans for each phase execution
//! - Metrics for performance tracking
//! - Events for significant phase milestones

use std::time::Instant;

#[cfg(feature = "otel")]
use tracing::{debug, error, info, warn, Span};

/// Phase telemetry context
pub struct PhaseTelemetry {
    phase_name: String,
    start_time: Instant,
    #[cfg(feature = "otel")]
    span: Span,
}

impl PhaseTelemetry {
    /// Create a new phase telemetry context
    pub fn new(phase_name: impl Into<String>) -> Self {
        let phase_name = phase_name.into();
        let start_time = Instant::now();

        #[cfg(feature = "otel")]
        let span = tracing::info_span!(
            "phase_execution",
            phase.name = %phase_name,
            phase.status = tracing::field::Empty,
            phase.duration_ms = tracing::field::Empty,
            phase.passed = tracing::field::Empty,
            phase.failed = tracing::field::Empty,
            phase.warnings = tracing::field::Empty,
        );

        Self {
            phase_name,
            start_time,
            #[cfg(feature = "otel")]
            span,
        }
    }

    /// Record phase start
    pub fn record_start(&self) {
        #[cfg(feature = "otel")]
        {
            let _enter = self.span.enter();
            info!(phase = %self.phase_name, "phase_started");
        }
    }

    /// Record phase completion
    pub fn record_completion(
        &self,
        status: &str,
        passed: usize,
        failed: usize,
        warnings: usize,
    ) {
        let duration = self.start_time.elapsed();

        #[cfg(feature = "otel")]
        {
            let _enter = self.span.enter();
            self.span.record("phase.status", status);
            self.span.record("phase.duration_ms", duration.as_millis());
            self.span.record("phase.passed", passed);
            self.span.record("phase.failed", failed);
            self.span.record("phase.warnings", warnings);

            match status {
                "pass" => info!(
                    phase = %self.phase_name,
                    duration_ms = duration.as_millis(),
                    passed,
                    "phase_completed_successfully"
                ),
                "fail" => error!(
                    phase = %self.phase_name,
                    duration_ms = duration.as_millis(),
                    failed,
                    "phase_failed"
                ),
                "warning" => warn!(
                    phase = %self.phase_name,
                    duration_ms = duration.as_millis(),
                    warnings,
                    "phase_completed_with_warnings"
                ),
                _ => debug!(
                    phase = %self.phase_name,
                    duration_ms = duration.as_millis(),
                    status,
                    "phase_completed"
                ),
            }
        }

        #[cfg(not(feature = "otel"))]
        {
            // Silent compilation without otel
            let _ = (status, passed, failed, warnings, duration);
        }
    }

    /// Record a phase metric
    pub fn record_metric(&self, key: &str, value: f64) {
        #[cfg(feature = "otel")]
        {
            let _enter = self.span.enter();
            debug!(
                phase = %self.phase_name,
                metric.key = key,
                metric.value = value,
                "phase_metric_recorded"
            );
        }

        #[cfg(not(feature = "otel"))]
        {
            let _ = (key, value);
        }
    }

    /// Record a phase error
    pub fn record_error(&self, error: &str) {
        #[cfg(feature = "otel")]
        {
            let _enter = self.span.enter();
            error!(
                phase = %self.phase_name,
                error = error,
                "phase_error"
            );
        }

        #[cfg(not(feature = "otel"))]
        {
            let _ = error;
        }
    }
}

/// Macro to wrap phase execution with telemetry
#[macro_export]
macro_rules! with_phase_telemetry {
    ($phase_name:expr, $block:block) => {{
        let telemetry = $crate::validation::phases::telemetry::PhaseTelemetry::new($phase_name);
        telemetry.record_start();

        let result = $block;

        match &result {
            Ok(phase_result) => {
                let status = match phase_result.status {
                    $crate::validation::phases::core::PhaseStatus::Pass => "pass",
                    $crate::validation::phases::core::PhaseStatus::Fail => "fail",
                    $crate::validation::phases::core::PhaseStatus::Warning => "warning",
                    $crate::validation::phases::core::PhaseStatus::Skipped => "skipped",
                };
                telemetry.record_completion(
                    status,
                    phase_result.passed,
                    phase_result.failed,
                    phase_result.warnings,
                );

                // Record all metrics
                for (key, value) in &phase_result.metrics {
                    telemetry.record_metric(key, *value);
                }
            }
            Err(e) => {
                telemetry.record_error(&format!("{:?}", e));
            }
        }

        result
    }};
}
