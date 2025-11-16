//! Common test utilities and mock implementations

use async_trait::async_trait;
use knhk_autonomous_loop::dependencies::*;
use knhk_autonomous_loop::*;
use std::sync::Arc;

pub struct MockSnapshotStore {
    pub current_id: [u8; 32],
}

#[async_trait]
impl SnapshotStore for MockSnapshotStore {
    async fn current_snapshot(&self) -> Result<SigmaSnapshot> {
        Ok(SigmaSnapshot {
            id: self.current_id,
            version: 1,
            validation_receipt: Some(ValidationReceipt {
                proposal_id: "test".to_string(),
                invariants_q_preserved: true,
                production_ready: true,
                validation_results: ValidationResults {
                    invariants_q_preserved: true,
                    tests_passed: 10,
                    tests_failed: 0,
                },
            }),
        })
    }

    fn get_snapshot(&self, id: &SigmaSnapshotId) -> Result<SigmaSnapshot> {
        Ok(SigmaSnapshot {
            id: *id,
            version: 1,
            validation_receipt: None,
        })
    }

    fn create_overlay(&self, name: String) -> SnapshotOverlay {
        SnapshotOverlay {
            base_snapshot_id: self.current_id,
            name,
            changes: vec![],
        }
    }

    async fn commit_overlay(
        &self,
        overlay: SnapshotOverlay,
        _receipt: ValidationReceipt,
    ) -> Result<SigmaSnapshotId> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&overlay.name);
        Ok(hasher.finalize().into())
    }
}

pub struct MockPatternMiner {
    pub pattern_count: usize,
}

impl PatternMiner for MockPatternMiner {
    fn scan(&self, _observations: &[ObservationReceipt]) -> Result<DetectedPatterns> {
        Ok(DetectedPatterns {
            patterns: (0..self.pattern_count)
                .map(|i| Pattern {
                    pattern_type: format!("pattern-{}", i),
                    confidence: 0.95,
                    occurrences: 10,
                })
                .collect(),
        })
    }
}

pub struct MockProposer {
    pub proposal_count: usize,
}

#[async_trait]
impl DeltaSigmaProposer for MockProposer {
    async fn propose(
        &self,
        _patterns: &DetectedPatterns,
    ) -> Result<Vec<ChangeProposal>> {
        Ok((0..self.proposal_count)
            .map(|i| ChangeProposal {
                id: format!("proposal-{}", i),
                change_type: "add_entity".to_string(),
                description: format!("Add entity {}", i),
            })
            .collect())
    }
}

pub struct MockValidator {
    pub pass_validation: bool,
}

#[async_trait]
impl DeltaSigmaValidator for MockValidator {
    async fn validate(
        &self,
        proposal: &ChangeProposal,
    ) -> Result<ValidationReceipt> {
        Ok(ValidationReceipt {
            proposal_id: proposal.id.clone(),
            invariants_q_preserved: self.pass_validation,
            production_ready: self.pass_validation,
            validation_results: ValidationResults {
                invariants_q_preserved: self.pass_validation,
                tests_passed: if self.pass_validation { 10 } else { 0 },
                tests_failed: if self.pass_validation { 0 } else { 10 },
            },
        })
    }
}

pub struct MockPromotionPipeline {
    pub should_fail: bool,
}

#[async_trait]
impl PromotionPipeline for MockPromotionPipeline {
    async fn promote_snapshot(&self, _snapshot_id: SigmaSnapshotId) -> Result<()> {
        if self.should_fail {
            Err(EvolutionError::PromotionFailed(
                "Mock promotion failure".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    async fn rollback(
        &self,
        _from: SigmaSnapshotId,
        _to: SigmaSnapshotId,
    ) -> Result<()> {
        Ok(())
    }
}

pub struct MockChangeExecutor;

#[async_trait]
impl ChangeExecutor for MockChangeExecutor {
    async fn apply_proposal_to_overlay(
        &self,
        overlay: &mut SnapshotOverlay,
        proposal: &ChangeProposal,
    ) -> Result<()> {
        overlay.changes.push(proposal.clone());
        Ok(())
    }
}

pub struct MockReceiptLog {
    pub receipt_count: usize,
}

#[async_trait]
impl ReceiptLog for MockReceiptLog {
    async fn recent_receipts(&self, limit: usize) -> Result<Vec<ObservationReceipt>> {
        Ok((0..self.receipt_count.min(limit))
            .map(|i| ObservationReceipt {
                id: format!("receipt-{}", i),
                operation: "test_operation".to_string(),
                attributes: vec![("key".to_string(), "value".to_string())],
            })
            .collect())
    }
}

pub fn create_test_dependencies(
    pattern_count: usize,
    proposal_count: usize,
    pass_validation: bool,
) -> LoopDependencies {
    LoopDependencies::new(
        Arc::new(MockSnapshotStore {
            current_id: [1u8; 32],
        }),
        Arc::new(MockPatternMiner { pattern_count }),
        Arc::new(MockProposer { proposal_count }),
        Arc::new(MockValidator { pass_validation }),
        Arc::new(MockPromotionPipeline { should_fail: false }),
        Arc::new(MockChangeExecutor),
        Arc::new(MockReceiptLog { receipt_count: 100 }),
    )
}
