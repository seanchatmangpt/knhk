//! Rollback manager for graceful failure recovery

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

use crate::errors::{RollbackError, RollbackResult, SloViolation};
use crate::hot_path::{self, HotPathDescriptor, SigmaSnapshotId};
use crate::telemetry::PromotionTelemetry;

/// Record of a single promotion event
#[derive(Debug, Clone)]
pub struct PromotionRecord {
    pub snapshot_id: SigmaSnapshotId,
    pub promoted_at: SystemTime,
    pub duration_nanos: u64,
    pub success: bool,
    pub error: Option<String>,
    pub generation: u64,
}

impl PromotionRecord {
    pub fn new(
        snapshot_id: SigmaSnapshotId,
        duration_nanos: u64,
        success: bool,
        error: Option<String>,
        generation: u64,
    ) -> Self {
        Self {
            snapshot_id,
            promoted_at: SystemTime::now(),
            duration_nanos,
            success,
            error,
            generation,
        }
    }
}

/// Rollback manager for graceful failure recovery
pub struct RollbackManager {
    /// Promotion history (last N promotions)
    history: Arc<RwLock<VecDeque<PromotionRecord>>>,

    /// Maximum history size
    max_history_size: usize,

    /// Previous snapshot ID (for immediate rollback)
    previous_snapshot_id: Arc<RwLock<Option<SigmaSnapshotId>>>,

    /// Telemetry
    telemetry: Arc<PromotionTelemetry>,
}

impl RollbackManager {
    /// Create a new rollback manager
    pub fn new(max_history_size: usize, telemetry: Arc<PromotionTelemetry>) -> Self {
        Self {
            history: Arc::new(RwLock::new(VecDeque::with_capacity(max_history_size))),
            max_history_size,
            previous_snapshot_id: Arc::new(RwLock::new(None)),
            telemetry,
        }
    }

    /// Record a successful promotion
    pub async fn record_promotion(
        &self,
        snapshot_id: SigmaSnapshotId,
        duration_nanos: u64,
        generation: u64,
    ) -> RollbackResult<()> {
        // Get current snapshot ID to store as previous
        let current_id = hot_path::get_current_snapshot_id();
        if current_id != [0u8; 32] {
            *self.previous_snapshot_id.write().await = Some(current_id);
        }

        let record = PromotionRecord::new(
            snapshot_id,
            duration_nanos,
            true,
            None,
            generation,
        );

        let mut history = self.history.write().await;

        // Add to history
        history.push_back(record);

        // Trim if exceeds max size
        if history.len() > self.max_history_size {
            history.pop_front();
        }

        tracing::debug!(
            snapshot_id = hex::encode(snapshot_id),
            generation,
            history_size = history.len(),
            "Recorded promotion in history"
        );

        Ok(())
    }

    /// Record a failed promotion
    pub async fn record_failure(
        &self,
        snapshot_id: SigmaSnapshotId,
        error: String,
        generation: u64,
    ) -> RollbackResult<()> {
        let record = PromotionRecord::new(
            snapshot_id,
            0,
            false,
            Some(error.clone()),
            generation,
        );

        let mut history = self.history.write().await;
        history.push_back(record);

        if history.len() > self.max_history_size {
            history.pop_front();
        }

        tracing::warn!(
            snapshot_id = hex::encode(snapshot_id),
            error,
            "Recorded promotion failure"
        );

        Ok(())
    }

    /// Rollback to previous snapshot (atomic, ≤10 ticks)
    pub async fn rollback_immediate(&self) -> RollbackResult<SigmaSnapshotId> {
        let prev_id = self.previous_snapshot_id.read().await
            .ok_or(RollbackError::NoPreviousSnapshot)?;

        tracing::warn!(
            from_snapshot = hex::encode(hot_path::get_current_snapshot_id()),
            to_snapshot = hex::encode(prev_id),
            "Initiating immediate rollback"
        );

        // Get current descriptor
        let current_desc = hot_path::load_current_descriptor();

        // Create rollback descriptor
        let rollback_desc = HotPathDescriptor::new(
            prev_id,
            0,  // Will need to reload projections
            current_desc.generation + 1,
        );

        // Atomic swap back to previous
        hot_path::store_descriptor(rollback_desc)
            .map_err(|e| RollbackError::RollbackFailed(e.to_string()))?;

        // Emit telemetry
        self.telemetry
            .emit_rollback_event(
                &current_desc.current_snapshot_id,
                &prev_id,
                "immediate rollback",
            )
            .await
            .map_err(|e| RollbackError::RollbackFailed(e.to_string()))?;

        tracing::info!(
            snapshot_id = hex::encode(prev_id),
            "Rollback complete"
        );

        Ok(prev_id)
    }

    /// Rollback to a specific snapshot by ID
    pub async fn rollback_to_snapshot(
        &self,
        target_snapshot_id: SigmaSnapshotId,
    ) -> RollbackResult<()> {
        // Verify target exists in history
        let history = self.history.read().await;
        let _target_record = history
            .iter()
            .find(|r| r.snapshot_id == target_snapshot_id && r.success)
            .ok_or_else(|| {
                RollbackError::InvalidRollbackTarget(hex::encode(target_snapshot_id))
            })?;

        tracing::warn!(
            from_snapshot = hex::encode(hot_path::get_current_snapshot_id()),
            to_snapshot = hex::encode(target_snapshot_id),
            "Initiating rollback to specific snapshot"
        );

        // Get current descriptor
        let current_desc = hot_path::load_current_descriptor();

        // Create rollback descriptor
        let rollback_desc = HotPathDescriptor::new(
            target_snapshot_id,
            0,
            current_desc.generation + 1,
        );

        // Atomic swap
        hot_path::store_descriptor(rollback_desc)
            .map_err(|e| RollbackError::RollbackFailed(e.to_string()))?;

        // Emit telemetry
        self.telemetry
            .emit_rollback_event(
                &current_desc.current_snapshot_id,
                &target_snapshot_id,
                "rollback to specific snapshot",
            )
            .await
            .map_err(|e| RollbackError::RollbackFailed(e.to_string()))?;

        Ok(())
    }

    /// Automated rollback on SLO violation
    pub async fn auto_rollback_on_slo_violation(
        &self,
        violation: &SloViolation,
    ) -> RollbackResult<SigmaSnapshotId> {
        tracing::error!(
            error_rate = violation.error_rate,
            latency_p99_ms = violation.latency_p99_ms,
            threshold_error_rate = violation.threshold_error_rate,
            threshold_latency_p99_ms = violation.threshold_latency_p99_ms,
            "SLO violation detected, initiating auto-rollback"
        );

        self.rollback_immediate().await
    }

    /// Get promotion history
    pub async fn get_history(&self) -> Vec<PromotionRecord> {
        self.history.read().await.iter().cloned().collect()
    }

    /// Get last successful promotion
    pub async fn get_last_successful_promotion(&self) -> Option<PromotionRecord> {
        self.history
            .read()
            .await
            .iter()
            .rev()
            .find(|r| r.success)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> RollbackManager {
        let telemetry = Arc::new(PromotionTelemetry::new());
        RollbackManager::new(10, telemetry)
    }

    #[tokio::test]
    async fn test_record_promotion() {
        let manager = create_test_manager();
        let snapshot_id = [1u8; 32];

        manager.record_promotion(snapshot_id, 8_000, 1).await.unwrap();

        let history = manager.get_history().await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].snapshot_id, snapshot_id);
        assert!(history[0].success);
    }

    #[tokio::test]
    async fn test_record_failure() {
        let manager = create_test_manager();
        let snapshot_id = [2u8; 32];

        manager
            .record_failure(snapshot_id, "Test error".to_string(), 1)
            .await
            .unwrap();

        let history = manager.get_history().await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].snapshot_id, snapshot_id);
        assert!(!history[0].success);
        assert_eq!(history[0].error, Some("Test error".to_string()));
    }

    #[tokio::test]
    async fn test_history_limit() {
        let manager = create_test_manager();

        // Record 15 promotions (limit is 10)
        for i in 0..15 {
            let snapshot_id = [i as u8; 32];
            manager.record_promotion(snapshot_id, 8_000, i + 1).await.unwrap();
        }

        let history = manager.get_history().await;
        assert_eq!(history.len(), 10);

        // First 5 should be dropped
        assert_eq!(history[0].snapshot_id, [5u8; 32]);
        assert_eq!(history[9].snapshot_id, [14u8; 32]);
    }

    #[tokio::test]
    async fn test_get_last_successful_promotion() {
        let manager = create_test_manager();

        manager.record_promotion([1u8; 32], 8_000, 1).await.unwrap();
        manager.record_failure([2u8; 32], "Error".to_string(), 2).await.unwrap();
        manager.record_promotion([3u8; 32], 9_000, 3).await.unwrap();

        let last = manager.get_last_successful_promotion().await.unwrap();
        assert_eq!(last.snapshot_id, [3u8; 32]);
        assert_eq!(last.generation, 3);
    }

    #[tokio::test]
    async fn test_rollback_immediate_no_previous() {
        hot_path::init_hot_path();
        let manager = create_test_manager();

        let result = manager.rollback_immediate().await;
        assert!(matches!(result, Err(RollbackError::NoPreviousSnapshot)));
    }

    #[tokio::test]
    async fn test_rollback_immediate_with_previous() {
        hot_path::init_hot_path();
        let manager = create_test_manager();

        // Record a promotion to set previous
        let prev_id = [10u8; 32];
        let new_id = [20u8; 32];

        let prev_desc = HotPathDescriptor::new(prev_id, 0, 1);
        hot_path::store_descriptor(prev_desc).unwrap();

        manager.record_promotion(new_id, 8_000, 2).await.unwrap();

        let new_desc = HotPathDescriptor::new(new_id, 0, 2);
        hot_path::store_descriptor(new_desc).unwrap();

        // Now rollback
        let rolled_back = manager.rollback_immediate().await.unwrap();
        assert_eq!(rolled_back, prev_id);

        // Verify hot path was updated
        let current = hot_path::get_current_snapshot_id();
        assert_eq!(current, prev_id);
    }
}
