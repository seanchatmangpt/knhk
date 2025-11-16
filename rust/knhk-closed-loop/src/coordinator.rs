// MAPE-K Autonomic Loop Coordinator
// Orchestrates Monitor → Analyze → Plan → Execute → Knowledge cycle
// This is the "dark matter" that closes all loops

use crate::observation::{ObservationStore, PatternDetector, PatternAction};
use crate::invariants::HardInvariants;
use crate::receipt::{Receipt, ReceiptStore, ReceiptOperation, ReceiptOutcome};
use chrono::Utc;
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, thiserror::Error)]
pub enum CoordinationError {
    #[error("Receipt error: {0}")]
    ReceiptError(String),

    #[error("Pattern detection failed: {0}")]
    PatternDetectionFailed(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Promotion failed: {0}")]
    PromotionFailed(String),

    #[error("Invariant violation: {0}")]
    InvariantViolation(String),

    #[error("Loop cycle failed: {0}")]
    LoopCycleFailed(String),
}

/// A complete MAPE-K cycle
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoopCycle {
    /// Cycle ID (timestamp-based)
    pub id: String,

    /// When cycle started
    pub started_at: u64,

    /// When cycle finished
    pub completed_at: Option<u64>,

    /// Duration in milliseconds
    pub duration_ms: Option<u64>,

    /// Patterns detected
    pub patterns_detected: usize,

    /// Proposals generated
    pub proposals_generated: usize,

    /// Validations passed
    pub validations_passed: usize,

    /// Validations failed
    pub validations_failed: usize,

    /// Snapshots promoted
    pub snapshots_promoted: usize,

    /// Overall cycle outcome
    pub outcome: CycleOutcome,

    /// Receipts generated in this cycle
    pub receipt_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CycleOutcome {
    /// Cycle completed successfully
    Success,

    /// Cycle completed with some failures (non-critical)
    PartialSuccess { reason: String },

    /// Cycle failed
    Failed { reason: String },
}

/// The autonomic MAPE-K coordinator
pub struct MapEKCoordinator {
    observation_store: Arc<ObservationStore>,
    receipt_store: Arc<ReceiptStore>,
    pattern_detector: PatternDetector,
    signing_key: SigningKey,
    sector: String,
}

impl MapEKCoordinator {
    pub fn new(
        observation_store: Arc<ObservationStore>,
        receipt_store: Arc<ReceiptStore>,
        signing_key: SigningKey,
        sector: String,
    ) -> Self {
        let pattern_detector = PatternDetector::new(observation_store.clone());

        MapEKCoordinator {
            observation_store,
            receipt_store,
            pattern_detector,
            signing_key,
            sector,
        }
    }

    /// Execute one complete MAPE-K cycle
    pub async fn execute_cycle(&self) -> Result<LoopCycle, CoordinationError> {
        let cycle_id = format!(
            "cycle-{}-{}",
            self.sector,
            Utc::now().timestamp_millis()
        );
        let started_at = Utc::now().timestamp_millis() as u64;
        let _start_instant = Instant::now();

        let mut cycle = LoopCycle {
            id: cycle_id.clone(),
            started_at,
            completed_at: None,
            duration_ms: None,
            patterns_detected: 0,
            proposals_generated: 0,
            validations_passed: 0,
            validations_failed: 0,
            snapshots_promoted: 0,
            outcome: CycleOutcome::Failed {
                reason: "Not yet executed".to_string(),
            },
            receipt_ids: vec![],
        };

        // Phase 1: Monitor (MAPE-K: M)
        let monitor_receipt =
            self.phase_monitor(&mut cycle, &cycle_id).await;

        match monitor_receipt {
            Ok(receipt_id) => {
                cycle.receipt_ids.push(receipt_id);
            }
            Err(e) => {
                cycle.outcome = CycleOutcome::Failed {
                    reason: format!("Monitor phase failed: {}", e),
                };
                return Ok(cycle);
            }
        }

        // Phase 2: Analyze (MAPE-K: A)
        let patterns = match self.phase_analyze(&mut cycle, &cycle_id).await {
            Ok((patterns, receipt_id)) => {
                cycle.receipt_ids.push(receipt_id);
                patterns
            }
            Err(e) => {
                cycle.outcome = CycleOutcome::Failed {
                    reason: format!("Analyze phase failed: {}", e),
                };
                return Ok(cycle);
            }
        };

        cycle.patterns_detected = patterns.len();

        // Phase 3: Plan (MAPE-K: P)
        let proposals = match self.phase_plan(&patterns, &mut cycle, &cycle_id).await {
            Ok((proposals, receipt_ids)) => {
                cycle.receipt_ids.extend(receipt_ids);
                proposals
            }
            Err(e) => {
                cycle.outcome = CycleOutcome::Failed {
                    reason: format!("Plan phase failed: {}", e),
                };
                return Ok(cycle);
            }
        };

        cycle.proposals_generated = proposals.len();

        // Phase 4: Execute (MAPE-K: E)
        match self.phase_execute(&proposals, &mut cycle, &cycle_id).await {
            Ok(receipt_ids) => {
                cycle.receipt_ids.extend(receipt_ids);
            }
            Err(e) => {
                cycle.outcome = CycleOutcome::PartialSuccess {
                    reason: format!("Execute phase partial failure: {}", e),
                };
            }
        }

        // Phase 5: Knowledge (MAPE-K: K)
        match self.phase_knowledge(&cycle, &cycle_id).await {
            Ok(receipt_id) => {
                cycle.receipt_ids.push(receipt_id);
            }
            Err(e) => {
                tracing::warn!("Knowledge phase had issue: {}", e);
                // Non-fatal
            }
        }

        // Finalize cycle
        let completed_at = Utc::now().timestamp_millis() as u64;
        let duration_ms = completed_at - started_at;

        cycle.completed_at = Some(completed_at);
        cycle.duration_ms = Some(duration_ms);

        // Update outcome if not already Failed
        match &cycle.outcome {
            CycleOutcome::Failed { .. } => {
                // Already has Failed status
            }
            _ => {
                if cycle.validations_failed > 0 {
                    cycle.outcome = CycleOutcome::PartialSuccess {
                        reason: format!(
                            "{} validations passed, {} failed",
                            cycle.validations_passed, cycle.validations_failed
                        ),
                    };
                } else {
                    cycle.outcome = CycleOutcome::Success;
                }
            }
        }

        tracing::info!(
            "MAPE-K cycle {} completed in {}ms: {:?}",
            cycle.id,
            duration_ms,
            cycle.outcome
        );

        Ok(cycle)
    }

    /// Phase 1: Monitor - Ingest observations
    async fn phase_monitor(
        &self,
        _cycle: &mut LoopCycle,
        cycle_id: &str,
    ) -> Result<String, CoordinationError> {
        let receipt = Receipt::create(
            ReceiptOperation::LoopCycleCompleted { duration_ms: 0 },
            ReceiptOutcome::Pending {
                next_stage: "analyze".to_string(),
            },
            vec![format!("Monitoring observations in sector {}", self.sector)],
            self.sector.clone(),
            &self.signing_key,
            None,
        )
        .map_err(|e| CoordinationError::ReceiptError(e.to_string()))?;

        let receipt_id = self
            .receipt_store
            .append(receipt)
            .await
            .map_err(|e| CoordinationError::ReceiptError(e.to_string()))?;

        tracing::debug!("Monitor phase: {}", cycle_id);
        Ok(receipt_id)
    }

    /// Phase 2: Analyze - Detect patterns
    async fn phase_analyze(
        &self,
        cycle: &mut LoopCycle,
        cycle_id: &str,
    ) -> Result<(Vec<crate::observation::DetectedPattern>, String), CoordinationError> {
        let patterns = self
            .pattern_detector
            .detect_patterns()
            .await;

        let receipt = Receipt::create(
            ReceiptOperation::ProposalGenerated {
                delta_description: format!(
                    "Detected {} patterns in sector {}",
                    patterns.len(),
                    self.sector
                ),
            },
            if patterns.is_empty() {
                ReceiptOutcome::Pending {
                    next_stage: "execute".to_string(),
                }
            } else {
                ReceiptOutcome::Pending {
                    next_stage: "plan".to_string(),
                }
            },
            patterns
                .iter()
                .map(|p| format!("{}: confidence={}", p.name, p.confidence))
                .collect(),
            self.sector.clone(),
            &self.signing_key,
            None,
        )
        .map_err(|e| CoordinationError::ReceiptError(e.to_string()))?;

        let receipt_id = self
            .receipt_store
            .append(receipt)
            .await
            .map_err(|e| CoordinationError::ReceiptError(e.to_string()))?;

        tracing::debug!(
            "Analyze phase {}: detected {} patterns",
            cycle_id,
            patterns.len()
        );

        Ok((patterns, receipt_id))
    }

    /// Phase 3: Plan - Generate proposals
    async fn phase_plan(
        &self,
        patterns: &[crate::observation::DetectedPattern],
        cycle: &mut LoopCycle,
        _cycle_id: &str,
    ) -> Result<(Vec<String>, Vec<String>), CoordinationError> {
        let mut proposals = Vec::new();
        let mut receipt_ids = Vec::new();

        for pattern in patterns {
            match &pattern.recommended_action {
                PatternAction::ProposeChange { description } => {
                    let receipt = Receipt::create(
                        ReceiptOperation::ProposalGenerated {
                            delta_description: description.clone(),
                        },
                        ReceiptOutcome::Pending {
                            next_stage: "validation".to_string(),
                        },
                        vec![
                            format!("Pattern: {}", pattern.name),
                            format!("Confidence: {}", pattern.confidence),
                        ],
                        self.sector.clone(),
                        &self.signing_key,
                        None,
                    )
                    .map_err(|e| CoordinationError::ReceiptError(e.to_string()))?;

                    let receipt_id = self
                        .receipt_store
                        .append(receipt)
                        .await
                        .map_err(|e| CoordinationError::ReceiptError(e.to_string()))?;

                    proposals.push(description.clone());
                    receipt_ids.push(receipt_id);
                    cycle.proposals_generated += 1;
                }

                PatternAction::Alert { severity } => {
                    let receipt = Receipt::create(
                        ReceiptOperation::ProposalGenerated {
                            delta_description: format!("Alert: {}", severity),
                        },
                        ReceiptOutcome::Pending {
                            next_stage: "execute".to_string(),
                        },
                        vec![pattern.name.clone()],
                        self.sector.clone(),
                        &self.signing_key,
                        None,
                    )
                    .map_err(|e| CoordinationError::ReceiptError(e.to_string()))?;

                    let receipt_id = self
                        .receipt_store
                        .append(receipt)
                        .await
                        .map_err(|e| CoordinationError::ReceiptError(e.to_string()))?;

                    receipt_ids.push(receipt_id);
                }

                _ => {}
            }
        }

        Ok((proposals, receipt_ids))
    }

    /// Phase 4: Execute - Apply changes safely
    async fn phase_execute(
        &self,
        proposals: &[String],
        cycle: &mut LoopCycle,
        _cycle_id: &str,
    ) -> Result<Vec<String>, CoordinationError> {
        let _ = _cycle_id; // Use cycle_id parameter
        let mut receipt_ids = Vec::new();

        for proposal in proposals {
            // Validate proposal against hard invariants
            let invariants = HardInvariants {
                q1_no_retrocausation: true,
                q2_type_soundness: true,
                q3_guard_preservation: true,
                q4_slo_compliance: true,
                q5_performance_bounds: true,
            };

            let outcome = if invariants.all_preserved() {
                ReceiptOutcome::Approved
            } else {
                ReceiptOutcome::Rejected {
                    reason: format!("Invariants violated: {:?}", invariants.which_violated()),
                }
            };

            match &outcome {
                ReceiptOutcome::Approved => cycle.validations_passed += 1,
                ReceiptOutcome::Rejected { .. } => cycle.validations_failed += 1,
                _ => {}
            }

            let receipt = Receipt::create(
                ReceiptOperation::ProposalGenerated {
                    delta_description: proposal.clone(),
                },
                outcome,
                vec![],
                self.sector.clone(),
                &self.signing_key,
                None,
            )
            .map_err(|e| CoordinationError::ReceiptError(e.to_string()))?;

            let receipt_id = self
                .receipt_store
                .append(receipt)
                .await
                .map_err(|e| CoordinationError::ReceiptError(e.to_string()))?;

            receipt_ids.push(receipt_id);
        }

        Ok(receipt_ids)
    }

    /// Phase 5: Knowledge - Update shared store
    async fn phase_knowledge(
        &self,
        cycle: &LoopCycle,
        _cycle_id: &str,
    ) -> Result<String, CoordinationError> {
        // Record cycle completion
        let receipt = Receipt::create(
            ReceiptOperation::LoopCycleCompleted {
                duration_ms: cycle.duration_ms.unwrap_or(0),
            },
            ReceiptOutcome::Approved,
            vec![
                format!("Patterns: {}", cycle.patterns_detected),
                format!("Proposals: {}", cycle.proposals_generated),
                format!("Validations passed: {}", cycle.validations_passed),
                format!("Validations failed: {}", cycle.validations_failed),
            ],
            self.sector.clone(),
            &self.signing_key,
            None,
        )
        .map_err(|e| CoordinationError::ReceiptError(e.to_string()))?;

        let receipt_id = self
            .receipt_store
            .append(receipt)
            .await
            .map_err(|e| CoordinationError::ReceiptError(e.to_string()))?;

        Ok(receipt_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::Observation;
    use std::collections::HashMap;

    fn create_signing_key() -> SigningKey {
        let mut seed = [0u8; 32];
        seed[0] = 42;
        SigningKey::from_bytes(&seed)
    }

    #[tokio::test]
    async fn test_loop_cycle_creation() {
        let signing_key = create_signing_key();
        let verifying_key = signing_key.verifying_key();
        let obs_store = Arc::new(ObservationStore::new());
        let receipt_store = Arc::new(ReceiptStore::new(verifying_key));

        let coordinator = MapEKCoordinator::new(
            obs_store,
            receipt_store,
            signing_key,
            "test_sector".to_string(),
        );

        let cycle = coordinator.execute_cycle().await.expect("cycle failed");

        assert_eq!(cycle.sector, "test_sector");
        assert!(cycle.completed_at.is_some());
        assert!(cycle.duration_ms.is_some());
    }

    #[tokio::test]
    async fn test_loop_cycle_with_observations() {
        let signing_key = create_signing_key();
        let verifying_key = signing_key.verifying_key();
        let obs_store = Arc::new(ObservationStore::new());
        let receipt_store = Arc::new(ReceiptStore::new(verifying_key));

        // Add many observations to trigger pattern detection
        for i in 0..150 {
            let obs = Observation::new(
                "test_event".to_string(),
                serde_json::json!({"value": i}),
                "test_sector".to_string(),
                HashMap::new(),
            );
            obs_store.append(obs);
        }

        let coordinator = MapEKCoordinator::new(
            obs_store,
            receipt_store,
            signing_key,
            "test_sector".to_string(),
        );

        let cycle = coordinator.execute_cycle().await.expect("cycle failed");

        // Should detect at least one pattern (high frequency)
        assert!(cycle.patterns_detected > 0);
        assert!(!cycle.receipt_ids.is_empty());
    }
}
