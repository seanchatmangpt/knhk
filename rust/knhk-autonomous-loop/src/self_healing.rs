//! Self-healing system - recovers from failures automatically

use crate::{LoopState, Result, SigmaSnapshotId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument, warn};

/// Recovery strategy when promotion fails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Rollback to previous known-good snapshot
    Rollback,

    /// Increase validation strictness and retry
    StrictValidation,

    /// Pause the loop and wait for manual intervention
    Pause,

    /// Ignore the failure and continue
    Continue,
}

/// Self-healing system that recovers from failures
///
/// When the autonomous loop encounters failures, this system:
/// 1. Detects the failure condition
/// 2. Determines if healing is needed
/// 3. Executes the configured recovery strategy
/// 4. Logs the healing action
pub struct SelfHealer {
    /// Recovery strategy to use
    strategy: RecoveryStrategy,

    /// Snapshot store (for rollback)
    snapshot_store: Arc<SnapshotStore>,

    /// Healing history
    history: Arc<RwLock<HealingHistory>>,
}

/// In-memory snapshot store (placeholder for actual implementation)
struct SnapshotStore {
    snapshots: Arc<RwLock<Vec<StoredSnapshot>>>,
}

#[derive(Debug, Clone)]
struct StoredSnapshot {
    id: SigmaSnapshotId,
    promoted: bool,
    created_at: std::time::SystemTime,
}

/// History of healing actions
#[derive(Debug, Clone, Default)]
struct HealingHistory {
    actions: Vec<HealingAction>,
}

#[derive(Debug, Clone)]
struct HealingAction {
    timestamp: std::time::SystemTime,
    strategy_used: RecoveryStrategy,
    success: bool,
    error: Option<String>,
}

impl SelfHealer {
    /// Create a new self-healer with the given strategy
    #[instrument(skip(strategy))]
    pub async fn new(strategy: RecoveryStrategy) -> Result<Self> {
        info!(?strategy, "Initializing self-healer");

        Ok(Self {
            strategy,
            snapshot_store: Arc::new(SnapshotStore::new().await?),
            history: Arc::new(RwLock::new(HealingHistory::default())),
        })
    }

    /// Check if healing should be triggered
    #[instrument(skip(self, state))]
    pub async fn should_heal(&self, state: &RwLock<LoopState>) -> Result<bool> {
        let state = state.read().await;

        // Heal if recent failures detected
        if state.failure_count >= 3 {
            warn!(
                failures = state.failure_count,
                "Failure threshold reached, healing needed"
            );
            return Ok(true);
        }

        // Heal if failure rate is high and we have enough data
        if state.cycle_count >= 10 {
            let failure_rate = state.failure_count as f64 / state.cycle_count as f64;
            if failure_rate > 0.3 {
                warn!(failure_rate, "High failure rate, healing needed");
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Execute healing action
    #[instrument(skip(self))]
    pub async fn heal(&self) -> Result<()> {
        info!(?self.strategy, "Executing healing strategy");

        let result = match &self.strategy {
            RecoveryStrategy::Rollback => self.rollback().await,
            RecoveryStrategy::StrictValidation => self.increase_validation().await,
            RecoveryStrategy::Pause => self.pause_loop().await,
            RecoveryStrategy::Continue => Ok(()),
        };

        // Record healing action
        self.record_healing_action(result.is_ok(), result.as_ref().err())
            .await?;

        result
    }

    /// Rollback to previous known-good snapshot
    #[instrument(skip(self))]
    async fn rollback(&self) -> Result<()> {
        info!("Rolling back to previous snapshot");

        // Find last promoted snapshot
        let previous = self.snapshot_store.find_last_promoted().await?;

        info!(snapshot_id = %previous.id.0, "Found previous snapshot");

        // Promote it (this would actually restore the snapshot)
        self.snapshot_store.promote_snapshot(&previous.id).await?;

        info!("Rollback completed successfully");
        Ok(())
    }

    /// Increase validation strictness for next cycle
    #[instrument(skip(self))]
    async fn increase_validation(&self) -> Result<()> {
        info!("Increasing validation strictness");

        // TODO: Integration with validation system
        // This would adjust validation parameters to be more strict

        Ok(())
    }

    /// Pause the loop
    #[instrument(skip(self))]
    async fn pause_loop(&self) -> Result<()> {
        warn!("Pausing loop - manual intervention required");

        // TODO: Integration with loop controller to pause it
        // This would set a flag that the loop checks

        Ok(())
    }

    /// Record a healing action in history
    async fn record_healing_action(
        &self,
        success: bool,
        error: Option<&crate::AutonomousLoopError>,
    ) -> Result<()> {
        let mut history = self.history.write().await;

        history.actions.push(HealingAction {
            timestamp: std::time::SystemTime::now(),
            strategy_used: self.strategy.clone(),
            success,
            error: error.map(|e| e.to_string()),
        });

        Ok(())
    }

    /// Get healing history (for monitoring)
    pub async fn get_history(&self) -> Vec<HealingAction> {
        self.history.read().await.actions.clone()
    }
}

impl SnapshotStore {
    async fn new() -> Result<Self> {
        Ok(Self {
            snapshots: Arc::new(RwLock::new(Vec::new())),
        })
    }

    async fn find_last_promoted(&self) -> Result<StoredSnapshot> {
        let snapshots = self.snapshots.read().await;

        snapshots
            .iter()
            .filter(|s| s.promoted)
            .max_by_key(|s| s.created_at)
            .cloned()
            .ok_or_else(|| {
                crate::AutonomousLoopError::RecoveryFailed(
                    "No promoted snapshot found".to_string(),
                )
            })
    }

    async fn promote_snapshot(&self, _id: &SigmaSnapshotId) -> Result<()> {
        // TODO: Actual promotion logic
        Ok(())
    }

    /// Add a snapshot for testing
    #[cfg(test)]
    async fn add_snapshot(&self, snapshot: StoredSnapshot) {
        self.snapshots.write().await.push(snapshot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_healer_creation() {
        let healer = SelfHealer::new(RecoveryStrategy::Rollback).await.unwrap();
        assert!(matches!(healer.strategy, RecoveryStrategy::Rollback));
    }

    #[tokio::test]
    async fn test_should_heal_failure_threshold() {
        let healer = SelfHealer::new(RecoveryStrategy::Rollback).await.unwrap();

        let state = Arc::new(RwLock::new(LoopState {
            cycle_count: 10,
            failure_count: 5,
            ..Default::default()
        }));

        let should_heal = healer.should_heal(&state).await.unwrap();
        assert!(should_heal);
    }

    #[tokio::test]
    async fn test_should_heal_failure_rate() {
        let healer = SelfHealer::new(RecoveryStrategy::Rollback).await.unwrap();

        let state = Arc::new(RwLock::new(LoopState {
            cycle_count: 10,
            failure_count: 4, // 40% failure rate
            ..Default::default()
        }));

        let should_heal = healer.should_heal(&state).await.unwrap();
        assert!(should_heal);
    }

    #[tokio::test]
    async fn test_rollback() {
        let healer = SelfHealer::new(RecoveryStrategy::Rollback).await.unwrap();

        // Add a promoted snapshot
        healer
            .snapshot_store
            .add_snapshot(StoredSnapshot {
                id: SigmaSnapshotId::new("snapshot-1"),
                promoted: true,
                created_at: std::time::SystemTime::now(),
            })
            .await;

        let result = healer.heal().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_healing_history() {
        let healer = SelfHealer::new(RecoveryStrategy::Continue).await.unwrap();

        healer.heal().await.unwrap();
        healer.heal().await.unwrap();

        let history = healer.get_history().await;
        assert_eq!(history.len(), 2);
    }
}
