//! Main promotion pipeline orchestrator

use std::sync::Arc;
use std::time::{SystemTime, Instant};
use tokio::sync::RwLock;

use crate::errors::{PromotionError, Result};
use crate::hot_path::{self, HotPathDescriptor, SigmaSnapshotId};
use crate::rollback::RollbackManager;
use crate::telemetry::{PromotionPhase, PromotionTelemetry};

/// Mock snapshot store (will be replaced with real implementation)
pub trait SnapshotStore: Send + Sync {
    fn get_snapshot(&self, id: &SigmaSnapshotId) -> Result<SigmaSnapshot>;
}

/// Mock snapshot structure
#[derive(Debug, Clone)]
pub struct SigmaSnapshot {
    pub id: SigmaSnapshotId,
    pub validation_receipt: Option<ValidationReceipt>,
}

/// Validation receipt from snapshot validation
#[derive(Debug, Clone)]
pub struct ValidationReceipt {
    pub validation_results: ValidationResults,
    pub production_ready: bool,
}

#[derive(Debug, Clone)]
pub struct ValidationResults {
    pub invariants_q_preserved: bool,
}

/// Mock projection compiler (will be replaced with real implementation)
///
/// NOTE: No async trait methods to maintain dyn compatibility
pub trait ProjectionCompiler: Send + Sync {
    fn compile_all_sync(&self, snapshot_id: SigmaSnapshotId) -> Result<CompiledProjections>;
}

/// Compiled projections
#[derive(Debug, Clone)]
pub struct CompiledProjections {
    pub snapshot_id: SigmaSnapshotId,
    pub compiled_at: SystemTime,
    // In real implementation, would contain actual compiled projection data
}

/// Result of a successful promotion
#[derive(Debug, Clone)]
pub struct PromotionResult {
    pub snapshot_id: SigmaSnapshotId,
    pub promotion_duration_nanos: u64,
    pub compiled_projections: CompiledProjections,
    pub timestamp: SystemTime,
}

/// Main promotion pipeline orchestrator
pub struct PromotionPipeline {
    /// Snapshot store
    snapshot_store: Arc<dyn SnapshotStore>,

    /// Projection compiler
    compiler: Arc<dyn ProjectionCompiler>,

    /// Rollback manager
    rollback_mgr: Arc<RollbackManager>,

    /// Current hot-path state (tracking only, not source of truth)
    current_state: Arc<RwLock<HotPathDescriptor>>,

    /// Telemetry
    telemetry: Arc<PromotionTelemetry>,
}

impl PromotionPipeline {
    /// Create a new promotion pipeline
    pub fn new(
        snapshot_store: Arc<dyn SnapshotStore>,
        compiler: Arc<dyn ProjectionCompiler>,
        rollback_mgr: Arc<RollbackManager>,
        telemetry: Arc<PromotionTelemetry>,
    ) -> Self {
        Self {
            snapshot_store,
            compiler,
            rollback_mgr,
            current_state: Arc::new(RwLock::new(HotPathDescriptor::null())),
            telemetry,
        }
    }

    /// Main pipeline: validate → compile → promote → switch
    pub async fn promote_snapshot(
        &self,
        snapshot_id: SigmaSnapshotId,
    ) -> Result<PromotionResult> {
        let span = tracing::info_span!(
            "promote_snapshot",
            snapshot_id = %hex::encode(snapshot_id),
        );

        let _guard = span.enter();

        tracing::info!("Starting promotion pipeline");

        // Phase 1: Pre-flight checks (≤100ms)
        tracing::info!("Phase 1: Pre-flight validation");
        let phase1_start = Instant::now();

        if let Err(e) = self.validate_snapshot_ready(&snapshot_id).await {
            self.telemetry
                .emit_promotion_failure(&snapshot_id, PromotionPhase::PreFlightValidation, &e)
                .await?;
            return Err(e);
        }

        let phase1_duration = phase1_start.elapsed();
        tracing::debug!(
            duration_ms = phase1_duration.as_millis(),
            "Pre-flight validation complete"
        );

        // Phase 2: Compile projections (≤1s, parallelized)
        tracing::info!("Phase 2: Compiling projections");
        let phase2_start = Instant::now();

        let compiled = match self.compiler.compile_all_sync(snapshot_id) {
            Ok(c) => c,
            Err(e) => {
                self.telemetry
                    .emit_promotion_failure(&snapshot_id, PromotionPhase::CompileProjections, &e)
                    .await?;
                return Err(e);
            }
        };

        let phase2_duration = phase2_start.elapsed();
        tracing::debug!(
            duration_ms = phase2_duration.as_millis(),
            "Projection compilation complete"
        );

        // Phase 3: Atomic promotion (≤10 ticks)
        tracing::info!("Phase 3: Atomic promotion");
        let start = Instant::now();

        let current_gen = hot_path::get_current_generation();

        let new_descriptor = HotPathDescriptor::new(
            snapshot_id,
            Arc::as_ptr(&Arc::new(compiled.clone())) as usize,
            current_gen + 1,
        );

        // THIS IS THE CRITICAL MOMENT
        // All hot path threads now see the new snapshot
        hot_path::store_descriptor(new_descriptor.clone())?;

        let duration = start.elapsed();
        tracing::info!(
            duration_nanos = duration.as_nanos(),
            "Atomic promotion complete"
        );

        if duration.as_nanos() > 10_000 {
            tracing::warn!(
                actual_nanos = duration.as_nanos(),
                budget_nanos = 10_000,
                "Promotion exceeded 10-tick budget"
            );
        }

        // Update state tracker
        *self.current_state.write().await = new_descriptor;

        // Record in rollback manager
        self.rollback_mgr
            .record_promotion(snapshot_id, duration.as_nanos() as u64, current_gen + 1)
            .await
            .map_err(|e| PromotionError::HotPathUpdateFailed(e.to_string()))?;

        // Phase 4: Emit telemetry
        tracing::info!("Phase 4: Telemetry emission");
        self.telemetry
            .emit_promotion_complete(&snapshot_id, duration.as_nanos() as u64)
            .await?;

        // Phase 5: Validation (optional, post-promotion)
        tracing::info!("Phase 5: Post-promotion validation");
        if let Err(e) = self.validate_post_promotion(&snapshot_id).await {
            tracing::error!(error = ?e, "Post-promotion validation failed, initiating rollback");

            // Attempt rollback
            if let Err(rollback_err) = self.rollback_mgr.rollback_immediate().await {
                tracing::error!(
                    error = ?rollback_err,
                    "Rollback failed after post-promotion validation failure"
                );
            }

            self.telemetry
                .emit_promotion_failure(&snapshot_id, PromotionPhase::PostPromotionValidation, &e)
                .await?;

            return Err(e);
        }

        tracing::info!("Promotion pipeline complete");

        Ok(PromotionResult {
            snapshot_id,
            promotion_duration_nanos: duration.as_nanos() as u64,
            compiled_projections: compiled,
            timestamp: SystemTime::now(),
        })
    }

    /// Validate that snapshot is ready for promotion
    async fn validate_snapshot_ready(&self, snapshot_id: &SigmaSnapshotId) -> Result<()> {
        // 1. Check snapshot exists
        let snapshot = self.snapshot_store.get_snapshot(snapshot_id)?;

        // 2. Check receipt exists and validation passed
        let receipt = snapshot
            .validation_receipt
            .as_ref()
            .ok_or(PromotionError::NoValidationReceipt)?;

        if !receipt.validation_results.invariants_q_preserved {
            return Err(PromotionError::InvariantsNotPreserved);
        }

        // 3. Check production ready flag
        if !receipt.production_ready {
            return Err(PromotionError::NotProductionReady);
        }

        tracing::debug!(
            snapshot_id = hex::encode(snapshot_id),
            "Snapshot validation passed"
        );

        Ok(())
    }

    /// Validate promotion was successful
    async fn validate_post_promotion(&self, snapshot_id: &SigmaSnapshotId) -> Result<()> {
        // Verify promotion was visible to a test operator
        let descriptor = hot_path::load_current_descriptor();

        if descriptor.current_snapshot_id != *snapshot_id {
            return Err(PromotionError::PromotionNotVisible);
        }

        tracing::debug!(
            snapshot_id = hex::encode(snapshot_id),
            "Post-promotion validation passed"
        );

        Ok(())
    }

    /// Get current snapshot ID
    pub fn get_current_snapshot_id(&self) -> SigmaSnapshotId {
        hot_path::get_current_snapshot_id()
    }

    /// Get rollback manager (for testing)
    pub fn rollback_manager(&self) -> Arc<RollbackManager> {
        self.rollback_mgr.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::PromotionError;

    struct MockSnapshotStore {
        snapshot: SigmaSnapshot,
    }

    impl SnapshotStore for MockSnapshotStore {
        fn get_snapshot(&self, _id: &SigmaSnapshotId) -> Result<SigmaSnapshot> {
            Ok(self.snapshot.clone())
        }
    }

    struct MockCompiler;

    impl ProjectionCompiler for MockCompiler {
        fn compile_all_sync(&self, snapshot_id: SigmaSnapshotId) -> Result<CompiledProjections> {
            Ok(CompiledProjections {
                snapshot_id,
                compiled_at: SystemTime::now(),
            })
        }
    }

    fn create_test_pipeline(snapshot: SigmaSnapshot) -> PromotionPipeline {
        let store = Arc::new(MockSnapshotStore { snapshot });
        let compiler = Arc::new(MockCompiler);
        let telemetry = Arc::new(PromotionTelemetry::new());
        let rollback = Arc::new(RollbackManager::new(10, telemetry.clone()));

        PromotionPipeline::new(store, compiler, rollback, telemetry)
    }

    #[tokio::test]
    async fn test_promote_snapshot_success() {
        hot_path::init_hot_path();

        let snapshot_id = [1u8; 32];
        let snapshot = SigmaSnapshot {
            id: snapshot_id,
            validation_receipt: Some(ValidationReceipt {
                validation_results: ValidationResults {
                    invariants_q_preserved: true,
                },
                production_ready: true,
            }),
        };

        let pipeline = create_test_pipeline(snapshot);

        let result = pipeline.promote_snapshot(snapshot_id).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.snapshot_id, snapshot_id);
        assert!(result.promotion_duration_nanos < 1_000_000); // Less than 1ms
    }

    #[tokio::test]
    async fn test_promote_snapshot_no_receipt() {
        hot_path::init_hot_path();

        let snapshot_id = [2u8; 32];
        let snapshot = SigmaSnapshot {
            id: snapshot_id,
            validation_receipt: None,
        };

        let pipeline = create_test_pipeline(snapshot);

        let result = pipeline.promote_snapshot(snapshot_id).await;
        assert!(matches!(result, Err(PromotionError::NoValidationReceipt)));
    }

    #[tokio::test]
    async fn test_promote_snapshot_invariants_not_preserved() {
        hot_path::init_hot_path();

        let snapshot_id = [3u8; 32];
        let snapshot = SigmaSnapshot {
            id: snapshot_id,
            validation_receipt: Some(ValidationReceipt {
                validation_results: ValidationResults {
                    invariants_q_preserved: false,
                },
                production_ready: true,
            }),
        };

        let pipeline = create_test_pipeline(snapshot);

        let result = pipeline.promote_snapshot(snapshot_id).await;
        assert!(matches!(result, Err(PromotionError::InvariantsNotPreserved)));
    }

    #[tokio::test]
    async fn test_promote_snapshot_not_production_ready() {
        hot_path::init_hot_path();

        let snapshot_id = [4u8; 32];
        let snapshot = SigmaSnapshot {
            id: snapshot_id,
            validation_receipt: Some(ValidationReceipt {
                validation_results: ValidationResults {
                    invariants_q_preserved: true,
                },
                production_ready: false,
            }),
        };

        let pipeline = create_test_pipeline(snapshot);

        let result = pipeline.promote_snapshot(snapshot_id).await;
        assert!(matches!(result, Err(PromotionError::NotProductionReady)));
    }

    #[tokio::test]
    async fn test_get_current_snapshot_id() {
        hot_path::init_hot_path();

        let snapshot_id = [5u8; 32];
        let snapshot = SigmaSnapshot {
            id: snapshot_id,
            validation_receipt: Some(ValidationReceipt {
                validation_results: ValidationResults {
                    invariants_q_preserved: true,
                },
                production_ready: true,
            }),
        };

        let pipeline = create_test_pipeline(snapshot);

        pipeline.promote_snapshot(snapshot_id).await.unwrap();

        let current = pipeline.get_current_snapshot_id();
        assert_eq!(current, snapshot_id);
    }
}
