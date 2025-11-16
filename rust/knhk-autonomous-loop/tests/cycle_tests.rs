//! Tests for evolution cycle execution

use knhk_autonomous_loop::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

// ============================================================================
// Mock Implementations for Testing
// ============================================================================

use async_trait::async_trait;
use knhk_autonomous_loop::dependencies::*;

struct MockSnapshotStore {
    current_id: [u8; 32],
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
        receipt: ValidationReceipt,
    ) -> Result<SigmaSnapshotId> {
        // Generate new ID
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&overlay.name);
        Ok(hasher.finalize().into())
    }
}

struct MockPatternMiner {
    pattern_count: usize,
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

struct MockProposer {
    proposal_count: usize,
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

struct MockValidator {
    pass_validation: bool,
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

struct MockPromotionPipeline {
    should_fail: bool,
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

struct MockChangeExecutor;

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

struct MockReceiptLog {
    receipt_count: usize,
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

fn create_test_dependencies(
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

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn test_successful_cycle() {
    let config = AutonomousLoopConfig::default();
    let deps = create_test_dependencies(10, 3, true);

    let cycle = EvolutionCycle::execute(&config, &deps).await.unwrap();

    assert!(cycle.is_success());
    assert_eq!(cycle.cycle_id, 0);
    assert!(!cycle.steps.is_empty());

    // Verify all steps executed
    let step_types: Vec<_> = cycle
        .steps
        .iter()
        .map(|s| match s {
            CycleStep::Observe { .. } => "observe",
            CycleStep::Detect { .. } => "detect",
            CycleStep::Propose { .. } => "propose",
            CycleStep::Validate { .. } => "validate",
            CycleStep::Compile { .. } => "compile",
            CycleStep::Promote { .. } => "promote",
            CycleStep::Rollback { .. } => "rollback",
        })
        .collect();

    assert!(step_types.contains(&"observe"));
    assert!(step_types.contains(&"detect"));
    assert!(step_types.contains(&"propose"));
    assert!(step_types.contains(&"validate"));
    assert!(step_types.contains(&"compile"));
}

#[tokio::test]
async fn test_insufficient_patterns() {
    let mut config = AutonomousLoopConfig::default();
    config.min_patterns_for_proposal = 20;

    let deps = create_test_dependencies(5, 3, true);

    let cycle = EvolutionCycle::execute(&config, &deps).await.unwrap();

    match cycle.result {
        CycleResult::NoChange { reason } => {
            assert!(reason.contains("patterns"));
        }
        _ => panic!("Expected NoChange result"),
    }
}

#[tokio::test]
async fn test_no_proposals_generated() {
    let config = AutonomousLoopConfig::default();
    let deps = create_test_dependencies(10, 0, true);

    let cycle = EvolutionCycle::execute(&config, &deps).await.unwrap();

    match cycle.result {
        CycleResult::NoChange { reason } => {
            assert!(reason.contains("No proposals"));
        }
        _ => panic!("Expected NoChange result"),
    }
}

#[tokio::test]
async fn test_validation_failures() {
    let config = AutonomousLoopConfig::default();
    let deps = create_test_dependencies(10, 3, false);

    let cycle = EvolutionCycle::execute(&config, &deps).await.unwrap();

    match cycle.result {
        CycleResult::PartialSuccess {
            proposals_rejected, ..
        } => {
            assert_eq!(proposals_rejected, 3);
        }
        _ => panic!("Expected PartialSuccess result"),
    }
}

#[tokio::test]
async fn test_max_changes_limit() {
    let mut config = AutonomousLoopConfig::default();
    config.max_changes_per_cycle = 2;

    let deps = create_test_dependencies(10, 5, true);

    let cycle = EvolutionCycle::execute(&config, &deps).await.unwrap();

    // Should only process 2 proposals (max limit)
    assert!(cycle.is_success());

    // Check that Propose step shows limited proposals
    let propose_step = cycle.steps.iter().find_map(|s| match s {
        CycleStep::Propose { proposals } => Some(*proposals),
        _ => None,
    });

    assert_eq!(propose_step, Some(2));
}

#[tokio::test]
async fn test_auto_promote_disabled() {
    let mut config = AutonomousLoopConfig::default();
    config.auto_promote = false;

    let deps = create_test_dependencies(10, 2, true);

    let cycle = EvolutionCycle::execute(&config, &deps).await.unwrap();

    // Should succeed but not promote
    assert!(cycle.is_success());

    // Verify no Promote step
    let has_promote = cycle.steps.iter().any(|s| {
        matches!(s, CycleStep::Promote { .. })
    });

    assert!(!has_promote);
}

#[tokio::test]
async fn test_cycle_counter_increments() {
    let config = AutonomousLoopConfig::default();
    let deps = create_test_dependencies(10, 2, true);

    assert_eq!(deps.current_cycle(), 0);

    let cycle1 = EvolutionCycle::execute(&config, &deps).await.unwrap();
    assert_eq!(cycle1.cycle_id, 0);
    assert_eq!(deps.current_cycle(), 1);

    let cycle2 = EvolutionCycle::execute(&config, &deps).await.unwrap();
    assert_eq!(cycle2.cycle_id, 1);
    assert_eq!(deps.current_cycle(), 2);
}

#[tokio::test]
async fn test_cycle_duration_tracking() {
    let config = AutonomousLoopConfig::default();
    let deps = create_test_dependencies(10, 2, true);

    let cycle = EvolutionCycle::execute(&config, &deps).await.unwrap();

    match cycle.result {
        CycleResult::Success { duration_ms, .. } => {
            // Duration may be 0 for very fast execution in tests
            assert!(duration_ms < 10000); // Should complete in < 10s
        }
        _ => panic!("Expected Success result"),
    }
}
