//! High-level promotion pipeline orchestrator

use crate::{PromotionError, PromotionGuard, Result, PromotionTelemetry};
use knhk_ontology::{SigmaSnapshotId, SnapshotStore};
use knhk_projections::ProjectionCompiler;
use std::sync::Arc;
use std::time::{SystemTime, Duration};
use tracing::{info, instrument, error};

/// Result of a promotion operation
#[derive(Debug, Clone)]
pub struct PromotionResult {
    /// Snapshot that was promoted
    pub snapshot_id: SigmaSnapshotId,

    /// When promotion completed
    pub promoted_at: SystemTime,

    /// How long the entire promotion took
    pub total_duration: Duration,

    /// Epoch counter after promotion
    pub new_epoch: u64,
}

/// High-level promotion pipeline
///
/// This orchestrates the entire promotion workflow:
/// 1. Validate snapshot is production-ready
/// 2. Compile all projections
/// 3. Create type-safe promotion guard
/// 4. Execute atomic promotion
/// 5. Emit telemetry
pub struct PromotionPipeline {
    /// Snapshot store
    store: Arc<SnapshotStore>,

    /// Projection compiler
    compiler: Arc<ProjectionCompiler>,

    /// Telemetry tracker
    telemetry: PromotionTelemetry,
}

impl PromotionPipeline {
    /// Create new promotion pipeline
    pub fn new(store: Arc<SnapshotStore>, compiler: Arc<ProjectionCompiler>) -> Self {
        Self {
            store,
            compiler,
            telemetry: PromotionTelemetry::new(),
        }
    }

    /// Promote a snapshot with full type-safety guarantees
    ///
    /// This is the main entry point for promotion. It performs all
    /// validation, compilation, and promotion with full observability.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Snapshot not found
    /// - Snapshot not production-ready
    /// - Compilation fails
    /// - Atomic promotion fails
    #[instrument(skip(self), fields(snapshot_id = ?snapshot_id))]
    pub async fn promote_snapshot(
        &self,
        snapshot_id: SigmaSnapshotId,
    ) -> Result<PromotionResult> {
        let start_time = SystemTime::now();

        info!("Starting promotion pipeline for snapshot {:?}", snapshot_id);

        // Phase 1: Get snapshot and validate
        let snapshot = self.store
            .get_snapshot(&snapshot_id)
            .ok_or_else(|| PromotionError::Snapshot(
                knhk_ontology::SnapshotError::NotFound(snapshot_id)
            ))?;

        self.telemetry.record_phase_start("validation");

        let receipt = snapshot.validation_receipt
            .as_ref()
            .ok_or_else(|| PromotionError::NotProductionReady(
                "Snapshot has no validation receipt".to_string()
            ))?;

        if !receipt.production_ready {
            error!("Snapshot not production-ready");
            return Err(PromotionError::NotProductionReady(
                "Receipt indicates snapshot not ready".to_string()
            ));
        }

        self.telemetry.record_phase_end("validation");

        // Phase 2: Compile all projections
        self.telemetry.record_phase_start("compilation");

        info!("Compiling projections for snapshot {:?}", snapshot_id);

        let compiled = self.compiler
            .compile_all(Arc::new(snapshot.clone()))
            .await
            .map_err(|e| PromotionError::CompilationNotReady(e.to_string()))?;

        self.telemetry.record_phase_end("compilation");

        // Phase 3: Create type-safe promotion guard (Preparing state)
        self.telemetry.record_phase_start("guard_creation");

        let guard = PromotionGuard::new(
            snapshot_id,
            Arc::new(receipt.clone()),
            Arc::new(compiled),
        )?;

        self.telemetry.record_phase_end("guard_creation");

        // Phase 4: Transition to Ready state
        self.telemetry.record_phase_start("ready");

        let guard = guard.ready().await?;

        self.telemetry.record_phase_end("ready");

        // Phase 5: Atomic promotion (≤10 ticks)
        self.telemetry.record_phase_start("atomic_promotion");

        let promoted = guard.promote()?;

        self.telemetry.record_phase_end("atomic_promotion");

        // Phase 6: Verify promotion succeeded
        promoted.verify_promoted()?;

        let total_duration = SystemTime::now()
            .duration_since(start_time)
            .unwrap_or(Duration::from_secs(0));

        info!(
            duration_ms = total_duration.as_millis(),
            "Promotion complete for snapshot {:?}",
            snapshot_id
        );

        self.telemetry.record_success(snapshot_id, total_duration);

        Ok(PromotionResult {
            snapshot_id,
            promoted_at: SystemTime::now(),
            total_duration,
            new_epoch: crate::hot_path::load_current_descriptor().epoch(),
        })
    }

    /// Promote with automatic compilation
    ///
    /// This is a convenience method that handles the entire workflow.
    pub async fn promote_with_compilation(
        &self,
        snapshot_id: SigmaSnapshotId,
    ) -> Result<PromotionResult> {
        self.promote_snapshot(snapshot_id).await
    }

    /// Get telemetry metrics
    pub fn telemetry(&self) -> &PromotionTelemetry {
        &self.telemetry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use knhk_ontology::*;
    use std::time::SystemTime;

    fn create_test_snapshot() -> SigmaSnapshot {
        let mut store = TripleStore::new();
        store.add(Triple::new("company1", "rdf:type", "Company"));
        store.add(Triple::new("company1", "sector", "Technology"));

        let metadata = SnapshotMetadata {
            created_by: "test".to_string(),
            description: "Test snapshot".to_string(),
            created_at: SystemTime::now(),
            sector: Some("Technology".to_string()),
        };

        let mut snapshot = SigmaSnapshot::new(None, store, metadata)
            .expect("Failed to create snapshot");

        // Add production-ready receipt
        let results = ValidationResults {
            static_checks_passed: true,
            dynamic_checks_passed: true,
            performance_checks_passed: true,
            invariants_q_preserved: true,
            errors: vec![],
            warnings: vec![],
        };

        let receipt = SigmaReceipt::new(
            snapshot.id,
            None,
            "Test snapshot".to_string(),
            results,
            100,
        );

        snapshot.validation_receipt = Some(receipt);
        snapshot
    }

    #[tokio::test]
    async fn test_promotion_pipeline() {
        crate::hot_path::init_hot_path();

        let store = Arc::new(SnapshotStore::new());
        let compiler = Arc::new(ProjectionCompiler::new());
        let pipeline = PromotionPipeline::new(store.clone(), compiler);

        let snapshot = create_test_snapshot();
        let snapshot_id = snapshot.id;

        // Add snapshot to store
        store.add_snapshot(snapshot);

        // Promote
        let result = pipeline.promote_snapshot(snapshot_id).await;

        assert!(result.is_ok(), "Promotion should succeed");

        let result = result.unwrap();
        assert_eq!(result.snapshot_id, snapshot_id);
        assert!(result.total_duration.as_millis() > 0);
    }

    #[tokio::test]
    async fn test_promotion_not_production_ready() {
        crate::hot_path::init_hot_path();

        let store = Arc::new(SnapshotStore::new());
        let compiler = Arc::new(ProjectionCompiler::new());
        let pipeline = PromotionPipeline::new(store.clone(), compiler);

        let mut snapshot = create_test_snapshot();

        // Make it NOT production-ready
        if let Some(receipt) = &mut snapshot.validation_receipt {
            let mut bad_results = receipt.validation_results.clone();
            bad_results.invariants_q_preserved = false;

            let bad_receipt = SigmaReceipt::new(
                snapshot.id,
                None,
                "Bad snapshot".to_string(),
                bad_results,
                100,
            );

            snapshot.validation_receipt = Some(bad_receipt);
        }

        let snapshot_id = snapshot.id;
        store.add_snapshot(snapshot);

        // Promotion should fail
        let result = pipeline.promote_snapshot(snapshot_id).await;
        assert!(result.is_err(), "Should reject non-production-ready snapshot");
    }
}
