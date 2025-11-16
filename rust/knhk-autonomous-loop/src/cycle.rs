//! Single evolution cycle implementation

use crate::config::AutonomousLoopConfig;
use crate::dependencies::LoopDependencies;
use crate::{EvolutionError, LegacySigmaSnapshotId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use std::time::SystemTime;

/// A single evolution cycle
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvolutionCycle {
    /// Unique cycle identifier
    pub cycle_id: u64,

    /// When the cycle started
    #[serde(with = "humantime_serde")]
    pub started_at: SystemTime,

    /// Steps executed in this cycle
    pub steps: Vec<CycleStep>,

    /// Final result of the cycle
    pub result: CycleResult,
}

/// Individual step in evolution cycle
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CycleStep {
    /// Observation step
    Observe { observations: u32 },

    /// Pattern detection step
    Detect { patterns_found: usize },

    /// Proposal generation step
    Propose { proposals: usize },

    /// Validation step
    Validate { passed: usize, failed: usize },

    /// Compilation step
    Compile { artifacts_generated: usize },

    /// Promotion step
    Promote { snapshot_id: LegacySigmaSnapshotId },

    /// Rollback step
    Rollback { reason: String },
}

/// Result of an evolution cycle
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CycleResult {
    /// Cycle completed successfully with new snapshot
    Success {
        new_snapshot_id: LegacySigmaSnapshotId,
        duration_ms: u64,
    },

    /// Cycle completed but with some failures
    PartialSuccess {
        patterns_detected: usize,
        proposals_rejected: usize,
        reason: String,
    },

    /// Cycle completed but no changes made
    NoChange { reason: String },

    /// Cycle failed
    Failure {
        error: String,
        rollback_performed: bool,
    },
}

impl EvolutionCycle {
    /// Execute one complete evolution cycle
    ///
    /// This implements the core evolution loop:
    /// 1. Observe - Fetch recent receipts from telemetry
    /// 2. Detect - Mine patterns from observations
    /// 3. Propose - Generate change proposals from patterns
    /// 4. Validate - Validate proposals against invariants
    /// 5. Compile - Apply changes and create new snapshot
    /// 6. Promote - Promote snapshot if production-ready
    pub async fn execute(
        config: &AutonomousLoopConfig,
        dependencies: &LoopDependencies,
    ) -> Result<Self> {
        let cycle_id = dependencies.cycle_counter.fetch_add(1, Ordering::SeqCst);
        let started_at = SystemTime::now();
        let mut steps = Vec::new();

        tracing::info!(cycle_id, "Starting evolution cycle");

        // ====================================================================
        // Step 1: Observe
        // ====================================================================
        tracing::info!(cycle_id, "Step 1: Observing telemetry data");

        let observations = dependencies.receipt_log.recent_receipts(1000).await?;

        tracing::info!(
            cycle_id,
            observations = observations.len(),
            "Observations collected"
        );

        steps.push(CycleStep::Observe {
            observations: observations.len() as u32,
        });

        // ====================================================================
        // Step 2: Detect Patterns
        // ====================================================================
        tracing::info!(cycle_id, "Step 2: Detecting patterns");

        let patterns = dependencies.pattern_miner.scan(&observations)?;

        tracing::info!(
            cycle_id,
            patterns = patterns.total_count(),
            "Patterns detected"
        );

        // Check if we have enough patterns to proceed
        if patterns.total_count() < config.min_patterns_for_proposal {
            tracing::info!(
                cycle_id,
                patterns = patterns.total_count(),
                min_required = config.min_patterns_for_proposal,
                "Insufficient patterns for proposal"
            );

            return Ok(Self {
                cycle_id,
                started_at,
                steps,
                result: CycleResult::NoChange {
                    reason: format!(
                        "Only {} patterns detected (minimum {})",
                        patterns.total_count(),
                        config.min_patterns_for_proposal
                    ),
                },
            });
        }

        steps.push(CycleStep::Detect {
            patterns_found: patterns.total_count(),
        });

        // ====================================================================
        // Step 3: Propose Changes
        // ====================================================================
        tracing::info!(cycle_id, "Step 3: Proposing changes");

        let proposals = dependencies.proposer.propose(&patterns).await?;

        tracing::info!(
            cycle_id,
            proposals = proposals.len(),
            "Proposals generated"
        );

        if proposals.is_empty() {
            return Ok(Self {
                cycle_id,
                started_at,
                steps,
                result: CycleResult::NoChange {
                    reason: "No proposals generated from patterns".to_string(),
                },
            });
        }

        // Enforce max changes limit
        let proposals: Vec<_> = proposals
            .into_iter()
            .take(config.max_changes_per_cycle)
            .collect();

        steps.push(CycleStep::Propose {
            proposals: proposals.len(),
        });

        // ====================================================================
        // Step 4: Validate Proposals
        // ====================================================================
        tracing::info!(
            cycle_id,
            proposals = proposals.len(),
            "Step 4: Validating proposals"
        );

        let mut passed = 0;
        let mut failed = 0;
        let mut validated_proposals = Vec::new();

        for proposal in proposals {
            match dependencies.validator.validate(&proposal).await {
                Ok(receipt) if receipt.validation_results.invariants_q_preserved => {
                    tracing::info!(
                        cycle_id,
                        proposal_id = %proposal.id,
                        "Proposal validated successfully"
                    );
                    passed += 1;
                    validated_proposals.push((proposal, receipt));
                }
                Ok(_receipt) => {
                    tracing::warn!(
                        cycle_id,
                        proposal_id = %proposal.id,
                        "Proposal failed: invariants not preserved"
                    );
                    failed += 1;
                }
                Err(e) => {
                    tracing::warn!(
                        cycle_id,
                        proposal_id = %proposal.id,
                        error = %e,
                        "Proposal validation error"
                    );
                    failed += 1;
                }
            }
        }

        steps.push(CycleStep::Validate { passed, failed });

        if validated_proposals.is_empty() {
            return Ok(Self {
                cycle_id,
                started_at,
                steps,
                result: CycleResult::PartialSuccess {
                    patterns_detected: patterns.total_count(),
                    proposals_rejected: failed,
                    reason: "No proposals passed validation".to_string(),
                },
            });
        }

        // ====================================================================
        // Step 5: Compile - Apply changes and create snapshot
        // ====================================================================
        tracing::info!(
            cycle_id,
            validated_proposals = validated_proposals.len(),
            "Step 5: Compiling changes into snapshot"
        );

        let _current_snapshot = dependencies.snapshot_store.current_snapshot().await?;
        let mut overlay = dependencies
            .snapshot_store
            .create_overlay(format!("auto-evolution-{}", cycle_id));

        // Apply each validated proposal to the overlay
        for (proposal, _receipt) in &validated_proposals {
            dependencies
                .change_executor
                .apply_proposal_to_overlay(&mut overlay, proposal)
                .await?;
        }

        // Commit the overlay as a new snapshot
        let last_receipt = validated_proposals.last().unwrap().1.clone();
        let new_snapshot_id = dependencies
            .snapshot_store
            .commit_overlay(overlay, last_receipt.clone())
            .await?;

        tracing::info!(
            cycle_id,
            snapshot_id = %hex::encode(&new_snapshot_id),
            "New snapshot created"
        );

        steps.push(CycleStep::Compile {
            artifacts_generated: 5, // 5 projections (OWL, YAWL, etc.)
        });

        // ====================================================================
        // Step 6: Promote (if auto-promote enabled and production-ready)
        // ====================================================================
        if config.auto_promote && last_receipt.production_ready {
            tracing::info!(cycle_id, "Step 6: Promoting snapshot to production");

            match dependencies
                .promotion_pipeline
                .promote_snapshot(new_snapshot_id)
                .await
            {
                Ok(()) => {
                    tracing::info!(
                        cycle_id,
                        snapshot_id = %hex::encode(&new_snapshot_id),
                        "Snapshot promoted successfully"
                    );

                    steps.push(CycleStep::Promote {
                        snapshot_id: new_snapshot_id,
                    });
                }
                Err(e) => {
                    tracing::error!(
                        cycle_id,
                        error = %e,
                        "Failed to promote snapshot"
                    );

                    return Ok(Self {
                        cycle_id,
                        started_at,
                        steps,
                        result: CycleResult::Failure {
                            error: format!("Promotion failed: {}", e),
                            rollback_performed: false,
                        },
                    });
                }
            }
        } else {
            tracing::info!(
                cycle_id,
                auto_promote = config.auto_promote,
                production_ready = last_receipt.production_ready,
                "Snapshot not promoted"
            );
        }

        // ====================================================================
        // Success!
        // ====================================================================
        let duration_ms = started_at
            .elapsed()
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_millis() as u64;

        tracing::info!(
            cycle_id,
            duration_ms,
            snapshot_id = %hex::encode(&new_snapshot_id),
            "Evolution cycle completed successfully"
        );

        Ok(Self {
            cycle_id,
            started_at,
            steps,
            result: CycleResult::Success {
                new_snapshot_id,
                duration_ms,
            },
        })
    }

    /// Get cycle duration in milliseconds
    pub fn duration_ms(&self) -> u64 {
        self.started_at
            .elapsed()
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_millis() as u64
    }

    /// Check if cycle was successful
    pub fn is_success(&self) -> bool {
        matches!(self.result, CycleResult::Success { .. })
    }

    /// Check if cycle failed
    pub fn is_failure(&self) -> bool {
        matches!(self.result, CycleResult::Failure { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycle_step_serialization() {
        let step = CycleStep::Observe { observations: 100 };
        let json = serde_json::to_string(&step).unwrap();
        let deserialized: CycleStep = serde_json::from_str(&json).unwrap();

        match deserialized {
            CycleStep::Observe { observations } => assert_eq!(observations, 100),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_cycle_result_success() {
        let result = CycleResult::Success {
            new_snapshot_id: [42u8; 32],
            duration_ms: 1000,
        };

        let cycle = EvolutionCycle {
            cycle_id: 1,
            started_at: SystemTime::now(),
            steps: vec![],
            result,
        };

        assert!(cycle.is_success());
        assert!(!cycle.is_failure());
    }

    #[test]
    fn test_cycle_result_failure() {
        let result = CycleResult::Failure {
            error: "test error".to_string(),
            rollback_performed: false,
        };

        let cycle = EvolutionCycle {
            cycle_id: 1,
            started_at: SystemTime::now(),
            steps: vec![],
            result,
        };

        assert!(!cycle.is_success());
        assert!(cycle.is_failure());
    }
}
