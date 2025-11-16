//! Dependency injection for the Autonomous Evolution Loop
//!
//! This module defines the dependencies required by the evolution loop
//! and provides mock implementations for testing.

use crate::{Result, SigmaSnapshotId};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// All dependencies required by the evolution loop
///
/// This uses dependency injection to allow testing with mocks
/// and production use with real implementations.
pub struct LoopDependencies {
    pub snapshot_store: Arc<dyn SnapshotStore>,
    pub pattern_miner: Arc<dyn PatternMiner>,
    pub proposer: Arc<dyn DeltaSigmaProposer>,
    pub validator: Arc<dyn DeltaSigmaValidator>,
    pub promotion_pipeline: Arc<dyn PromotionPipeline>,
    pub change_executor: Arc<dyn ChangeExecutor>,
    pub receipt_log: Arc<dyn ReceiptLog>,
    pub cycle_counter: Arc<AtomicU64>,
}

impl LoopDependencies {
    /// Create new dependencies with provided implementations
    pub fn new(
        snapshot_store: Arc<dyn SnapshotStore>,
        pattern_miner: Arc<dyn PatternMiner>,
        proposer: Arc<dyn DeltaSigmaProposer>,
        validator: Arc<dyn DeltaSigmaValidator>,
        promotion_pipeline: Arc<dyn PromotionPipeline>,
        change_executor: Arc<dyn ChangeExecutor>,
        receipt_log: Arc<dyn ReceiptLog>,
    ) -> Self {
        Self {
            snapshot_store,
            pattern_miner,
            proposer,
            validator,
            promotion_pipeline,
            change_executor,
            receipt_log,
            cycle_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get current cycle count
    pub fn current_cycle(&self) -> u64 {
        self.cycle_counter.load(Ordering::SeqCst)
    }
}

/// Snapshot storage interface
#[async_trait::async_trait]
pub trait SnapshotStore: Send + Sync {
    /// Get current active snapshot
    async fn current_snapshot(&self) -> Result<SigmaSnapshot>;

    /// Get snapshot by ID
    fn get_snapshot(&self, id: &SigmaSnapshotId) -> Result<SigmaSnapshot>;

    /// Create new snapshot overlay for changes
    fn create_overlay(&self, name: String) -> SnapshotOverlay;

    /// Commit overlay as new snapshot
    async fn commit_overlay(
        &self,
        overlay: SnapshotOverlay,
        receipt: ValidationReceipt,
    ) -> Result<SigmaSnapshotId>;
}

/// Pattern mining interface
pub trait PatternMiner: Send + Sync {
    /// Scan observations for patterns
    fn scan(&self, observations: &[ObservationReceipt]) -> Result<DetectedPatterns>;
}

/// Proposal generation interface
#[async_trait::async_trait]
pub trait DeltaSigmaProposer: Send + Sync {
    /// Propose changes based on patterns
    async fn propose(&self, patterns: &DetectedPatterns) -> Result<Vec<ChangeProposal>>;
}

/// Validation interface
#[async_trait::async_trait]
pub trait DeltaSigmaValidator: Send + Sync {
    /// Validate a change proposal
    async fn validate(&self, proposal: &ChangeProposal) -> Result<ValidationReceipt>;
}

/// Promotion pipeline interface
#[async_trait::async_trait]
pub trait PromotionPipeline: Send + Sync {
    /// Promote snapshot to production
    async fn promote_snapshot(&self, snapshot_id: SigmaSnapshotId) -> Result<()>;

    /// Rollback to previous snapshot
    async fn rollback(
        &self,
        from: SigmaSnapshotId,
        to: SigmaSnapshotId,
    ) -> Result<()>;
}

/// Change execution interface
#[async_trait::async_trait]
pub trait ChangeExecutor: Send + Sync {
    /// Apply proposal to snapshot overlay
    async fn apply_proposal_to_overlay(
        &self,
        overlay: &mut SnapshotOverlay,
        proposal: &ChangeProposal,
    ) -> Result<()>;
}

/// Receipt log interface
#[async_trait::async_trait]
pub trait ReceiptLog: Send + Sync {
    /// Get recent observation receipts
    async fn recent_receipts(&self, limit: usize) -> Result<Vec<ObservationReceipt>>;
}

// ============================================================================
// Data Types
// ============================================================================

/// Ontology snapshot
#[derive(Clone, Debug)]
pub struct SigmaSnapshot {
    pub id: SigmaSnapshotId,
    pub version: u64,
    pub validation_receipt: Option<ValidationReceipt>,
}

/// Snapshot overlay for pending changes
#[derive(Clone, Debug)]
pub struct SnapshotOverlay {
    pub base_snapshot_id: SigmaSnapshotId,
    pub name: String,
    pub changes: Vec<ChangeProposal>,
}

/// Observation receipt from telemetry
#[derive(Clone, Debug)]
pub struct ObservationReceipt {
    pub id: String,
    pub operation: String,
    pub attributes: Vec<(String, String)>,
}

/// Detected patterns from observations
#[derive(Clone, Debug)]
pub struct DetectedPatterns {
    pub patterns: Vec<Pattern>,
}

impl DetectedPatterns {
    pub fn total_count(&self) -> usize {
        self.patterns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }
}

/// Individual pattern
#[derive(Clone, Debug)]
pub struct Pattern {
    pub pattern_type: String,
    pub confidence: f64,
    pub occurrences: usize,
}

/// Change proposal
#[derive(Clone, Debug)]
pub struct ChangeProposal {
    pub id: String,
    pub change_type: String,
    pub description: String,
}

/// Validation receipt
#[derive(Clone, Debug)]
pub struct ValidationReceipt {
    pub proposal_id: String,
    pub invariants_q_preserved: bool,
    pub production_ready: bool,
    pub validation_results: ValidationResults,
}

/// Validation results
#[derive(Clone, Debug)]
pub struct ValidationResults {
    pub invariants_q_preserved: bool,
    pub tests_passed: usize,
    pub tests_failed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detected_patterns() {
        let patterns = DetectedPatterns { patterns: vec![] };
        assert_eq!(patterns.total_count(), 0);
        assert!(patterns.is_empty());

        let patterns = DetectedPatterns {
            patterns: vec![Pattern {
                pattern_type: "test".to_string(),
                confidence: 0.95,
                occurrences: 10,
            }],
        };
        assert_eq!(patterns.total_count(), 1);
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_cycle_counter() {
        let counter = Arc::new(AtomicU64::new(0));
        assert_eq!(counter.load(Ordering::SeqCst), 0);

        counter.fetch_add(1, Ordering::SeqCst);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
