//! Change Executor - Applies validated ΔΣ proposals
//!
//! The Executor orchestrates the full change cycle:
//!
//! 1. Detect patterns in receipts (Pattern Miner)
//! 2. Propose changes (Proposer)
//! 3. Validate proposals (Validator)
//! 4. Apply validated changes to create new snapshots
//! 5. Optionally promote to production if all checks pass

use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn, span, Level};
use crate::{
    pattern_miner::{PatternMiner, DetectedPatterns},
    proposer::{DeltaSigmaProposer, DeltaSigmaProposal},
    validator::{DeltaSigmaValidator, ValidationResult},
};

/// Change Executor - orchestrates the full ΔΣ cycle
#[derive(Debug)]
pub struct ChangeExecutor {
    /// Pattern miner
    miner: Arc<RwLock<PatternMiner>>,

    /// Proposer
    proposer: Arc<DeltaSigmaProposer>,

    /// Validator
    validator: Arc<DeltaSigmaValidator>,

    /// Configuration
    config: ExecutorConfig,
}

/// Executor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorConfig {
    /// Auto-promote to production if validation passes
    pub auto_promote: bool,

    /// Minimum validation score to accept proposal (0.0 - 1.0)
    pub min_validation_score: f64,

    /// Maximum proposals to process per cycle
    pub max_proposals_per_cycle: usize,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            auto_promote: false,
            min_validation_score: 0.95,
            max_proposals_per_cycle: 10,
        }
    }
}

/// Snapshot ID (placeholder - would reference actual snapshot store)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotId(pub String);

/// Change cycle result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeCycleResult {
    /// Snapshot ID created
    pub snapshot_id: SnapshotId,

    /// Proposals submitted
    pub proposals_submitted: usize,

    /// Proposals validated
    pub proposals_validated: usize,

    /// Proposals applied
    pub proposals_applied: usize,

    /// Whether promoted to production
    pub promoted: bool,

    /// Validation results
    pub validation_results: Vec<ValidationResult>,
}

impl ChangeExecutor {
    /// Create a new change executor
    pub fn new() -> Self {
        Self::with_config(ExecutorConfig::default())
    }

    /// Create a new change executor with custom config
    pub fn with_config(config: ExecutorConfig) -> Self {
        let span = span!(Level::INFO, "executor_init");
        let _enter = span.enter();

        info!(?config, "Initializing change executor");

        let patterns = Arc::new(RwLock::new(DetectedPatterns::default()));
        let miner = Arc::new(RwLock::new(PatternMiner::new(1000)));
        let proposer = Arc::new(DeltaSigmaProposer::new(patterns));
        let validator = Arc::new(DeltaSigmaValidator::new());

        Self {
            miner,
            proposer,
            validator,
            config,
        }
    }

    /// Execute full change cycle: detect → propose → validate → commit
    pub async fn execute_change_cycle(&self) -> crate::Result<ChangeCycleResult> {
        let span = span!(Level::INFO, "execute_change_cycle");
        let _enter = span.enter();

        info!("Starting autonomous change cycle");

        // 1. Propose changes based on detected patterns
        let proposals = self.proposer.propose_delta_sigma().await?;
        let proposals_submitted = proposals.len();

        info!(proposals = proposals_submitted, "Generated ΔΣ proposals");

        if proposals.is_empty() {
            info!("No proposals generated, skipping cycle");
            return Ok(ChangeCycleResult {
                snapshot_id: SnapshotId("none".to_string()),
                proposals_submitted: 0,
                proposals_validated: 0,
                proposals_applied: 0,
                promoted: false,
                validation_results: vec![],
            });
        }

        // 2. Validate each proposal
        let mut validated_proposals = Vec::new();
        let mut validation_results = Vec::new();

        for (i, proposal) in proposals.iter().enumerate().take(self.config.max_proposals_per_cycle) {
            debug!(proposal_index = i, "Validating proposal");

            let result = self.validator.validate_proposal(proposal).await?;
            validation_results.push(result.clone());

            if result.passed {
                validated_proposals.push(proposal.clone());
            } else {
                warn!(proposal_index = i, "Proposal failed validation");
            }
        }

        let proposals_validated = validated_proposals.len();
        info!(validated = proposals_validated, total = proposals_submitted, "Validation complete");

        // 3. Apply validated proposals
        let proposals_applied = if !validated_proposals.is_empty() {
            self.apply_proposals(&validated_proposals).await?
        } else {
            0
        };

        // 4. Create snapshot
        let snapshot_id = self.create_snapshot(&validated_proposals).await?;

        // 5. Optionally promote to production
        let promoted = if self.config.auto_promote && proposals_applied > 0 {
            self.promote_snapshot(&snapshot_id).await?
        } else {
            false
        };

        info!(
            snapshot_id = %snapshot_id.0,
            applied = proposals_applied,
            promoted,
            "Change cycle complete"
        );

        Ok(ChangeCycleResult {
            snapshot_id,
            proposals_submitted,
            proposals_validated,
            proposals_applied,
            promoted,
            validation_results,
        })
    }

    /// Apply proposals to shadow Σ
    async fn apply_proposals(&self, proposals: &[DeltaSigmaProposal]) -> crate::Result<usize> {
        let span = span!(Level::DEBUG, "apply_proposals", count = proposals.len());
        let _enter = span.enter();

        debug!(count = proposals.len(), "Applying validated proposals");

        // In production, this would:
        // 1. Create overlay/shadow Σ
        // 2. Apply each proposal to shadow
        // 3. Verify integrity
        // 4. Return count of successfully applied proposals

        Ok(proposals.len())
    }

    /// Create a new snapshot with applied changes
    async fn create_snapshot(&self, proposals: &[DeltaSigmaProposal]) -> crate::Result<SnapshotId> {
        let span = span!(Level::DEBUG, "create_snapshot");
        let _enter = span.enter();

        debug!(proposals = proposals.len(), "Creating snapshot");

        // In production, this would:
        // 1. Serialize applied proposals
        // 2. Create validation receipt
        // 3. Commit to snapshot store
        // 4. Return snapshot ID

        let snapshot_id = SnapshotId(format!("snapshot_{}", uuid::Uuid::new_v4()));

        Ok(snapshot_id)
    }

    /// Promote snapshot to production
    async fn promote_snapshot(&self, snapshot_id: &SnapshotId) -> crate::Result<bool> {
        let span = span!(Level::INFO, "promote_snapshot", snapshot_id = %snapshot_id.0);
        let _enter = span.enter();

        info!(snapshot_id = %snapshot_id.0, "Promoting snapshot to production");

        // In production, this would:
        // 1. Run final validation
        // 2. Update production pointer
        // 3. Notify subscribers
        // 4. Return success/failure

        Ok(true)
    }

    /// Get pattern miner (for testing/inspection)
    pub fn pattern_miner(&self) -> Arc<RwLock<PatternMiner>> {
        self.miner.clone()
    }

    /// Get configuration
    pub fn config(&self) -> &ExecutorConfig {
        &self.config
    }
}

impl Default for ChangeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = ChangeExecutor::new();
        assert!(!executor.config().auto_promote);
    }

    #[test]
    fn test_executor_with_custom_config() {
        let config = ExecutorConfig {
            auto_promote: true,
            min_validation_score: 0.99,
            max_proposals_per_cycle: 5,
        };

        let executor = ChangeExecutor::with_config(config.clone());
        assert_eq!(executor.config().auto_promote, true);
        assert_eq!(executor.config().min_validation_score, 0.99);
        assert_eq!(executor.config().max_proposals_per_cycle, 5);
    }

    #[tokio::test]
    async fn test_execute_change_cycle_no_patterns() {
        let executor = ChangeExecutor::new();

        let result = executor.execute_change_cycle().await.unwrap();

        // With no patterns, should generate no proposals
        assert_eq!(result.proposals_submitted, 0);
        assert_eq!(result.proposals_applied, 0);
        assert!(!result.promoted);
    }
}
