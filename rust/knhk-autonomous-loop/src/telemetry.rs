//! OpenTelemetry integration for the Autonomous Evolution Loop

use crate::cycle::{CycleResult, EvolutionCycle};
use crate::EvolutionError;
use tracing::{info, warn};

/// Telemetry integration for evolution loop
pub struct LoopTelemetry {
    /// Service name for telemetry
    service_name: String,
}

impl LoopTelemetry {
    /// Create new telemetry instance
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }

    /// Emit cycle complete event
    pub async fn emit_cycle_complete(
        &self,
        cycle: &EvolutionCycle,
    ) -> crate::Result<()> {
        let span = tracing::info_span!(
            "evolution.cycle",
            service.name = %self.service_name,
            cycle.id = cycle.cycle_id,
            cycle.steps = cycle.steps.len(),
        );

        let _enter = span.enter();

        match &cycle.result {
            CycleResult::Success {
                new_snapshot_id,
                duration_ms,
            } => {
                info!(
                    cycle.result = "success",
                    snapshot.id = %hex::encode(new_snapshot_id),
                    duration_ms = duration_ms,
                    "Evolution cycle completed successfully"
                );

                // Record metrics
                self.record_cycle_duration(*duration_ms);
                self.record_cycle_success();
            }
            CycleResult::PartialSuccess {
                patterns_detected,
                proposals_rejected,
                reason,
            } => {
                warn!(
                    cycle.result = "partial_success",
                    patterns_detected = patterns_detected,
                    proposals_rejected = proposals_rejected,
                    reason = %reason,
                    "Evolution cycle partially succeeded"
                );

                self.record_cycle_partial();
            }
            CycleResult::NoChange { reason } => {
                info!(
                    cycle.result = "no_change",
                    reason = %reason,
                    "Evolution cycle completed with no changes"
                );

                self.record_cycle_no_change();
            }
            CycleResult::Failure {
                error,
                rollback_performed,
            } => {
                warn!(
                    cycle.result = "failure",
                    error = %error,
                    rollback_performed = rollback_performed,
                    "Evolution cycle failed"
                );

                self.record_cycle_failure();
            }
        }

        Ok(())
    }

    /// Emit cycle error event
    pub async fn emit_cycle_error(&self, error: &EvolutionError) -> crate::Result<()> {
        let span = tracing::error_span!(
            "evolution.cycle.error",
            service.name = %self.service_name,
            error.type = "cycle_error",
        );

        let _enter = span.enter();

        tracing::error!(
            error = %error,
            "Evolution cycle encountered error"
        );

        self.record_cycle_failure();

        Ok(())
    }

    /// Emit snapshot promoted event
    pub async fn emit_snapshot_promoted(
        &self,
        snapshot_id: &[u8; 32],
    ) -> crate::Result<()> {
        let span = tracing::info_span!(
            "evolution.snapshot.promoted",
            service.name = %self.service_name,
            snapshot.id = %hex::encode(snapshot_id),
        );

        let _enter = span.enter();

        info!("Snapshot promoted to production");

        Ok(())
    }

    /// Emit rollback performed event
    pub async fn emit_rollback_performed(
        &self,
        from_snapshot: &[u8; 32],
        to_snapshot: &[u8; 32],
        reason: &str,
    ) -> crate::Result<()> {
        let span = tracing::warn_span!(
            "evolution.snapshot.rollback",
            service.name = %self.service_name,
            from_snapshot.id = %hex::encode(from_snapshot),
            to_snapshot.id = %hex::encode(to_snapshot),
        );

        let _enter = span.enter();

        warn!(reason = %reason, "Snapshot rollback performed");

        Ok(())
    }

    /// Emit pattern detection event
    pub async fn emit_patterns_detected(
        &self,
        count: usize,
        cycle_id: u64,
    ) -> crate::Result<()> {
        let span = tracing::info_span!(
            "evolution.patterns.detected",
            service.name = %self.service_name,
            cycle.id = cycle_id,
            patterns.count = count,
        );

        let _enter = span.enter();

        info!("Patterns detected");

        Ok(())
    }

    /// Record cycle duration metric
    fn record_cycle_duration(&self, duration_ms: u64) {
        // In production, this would record to OpenTelemetry metrics
        tracing::debug!(duration_ms = duration_ms, "Recorded cycle duration");
    }

    /// Record successful cycle
    fn record_cycle_success(&self) {
        tracing::debug!("Recorded successful cycle");
    }

    /// Record partial success cycle
    fn record_cycle_partial(&self) {
        tracing::debug!("Recorded partial success cycle");
    }

    /// Record no-change cycle
    fn record_cycle_no_change(&self) {
        tracing::debug!("Recorded no-change cycle");
    }

    /// Record failed cycle
    fn record_cycle_failure(&self) {
        tracing::debug!("Recorded failed cycle");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cycle::{CycleResult, CycleStep, EvolutionCycle};
    use std::time::SystemTime;

    #[tokio::test]
    async fn test_emit_cycle_success() {
        let telemetry = LoopTelemetry::new("test-service".to_string());

        let cycle = EvolutionCycle {
            cycle_id: 1,
            started_at: SystemTime::now(),
            steps: vec![],
            result: CycleResult::Success {
                new_snapshot_id: [42u8; 32],
                duration_ms: 1000,
            },
        };

        assert!(telemetry.emit_cycle_complete(&cycle).await.is_ok());
    }

    #[tokio::test]
    async fn test_emit_cycle_failure() {
        let telemetry = LoopTelemetry::new("test-service".to_string());

        let cycle = EvolutionCycle {
            cycle_id: 2,
            started_at: SystemTime::now(),
            steps: vec![],
            result: CycleResult::Failure {
                error: "test error".to_string(),
                rollback_performed: true,
            },
        };

        assert!(telemetry.emit_cycle_complete(&cycle).await.is_ok());
    }

    #[tokio::test]
    async fn test_emit_snapshot_promoted() {
        let telemetry = LoopTelemetry::new("test-service".to_string());
        let snapshot_id = [1u8; 32];

        assert!(telemetry.emit_snapshot_promoted(&snapshot_id).await.is_ok());
    }

    #[tokio::test]
    async fn test_emit_rollback() {
        let telemetry = LoopTelemetry::new("test-service".to_string());
        let from = [1u8; 32];
        let to = [2u8; 32];

        assert!(telemetry
            .emit_rollback_performed(&from, &to, "SLO violation")
            .await
            .is_ok());
    }
}
