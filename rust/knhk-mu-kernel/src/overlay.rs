//! ΔΣ - Proof-Carrying Overlay Algebra
//!
//! Overlays are not "diffs"; they are proof-carrying contracts
//! that propose changes to Σ with machine-checkable justifications.

use crate::sigma::{SigmaHash, SigmaCompiled, TaskDescriptor};
use crate::guards::GuardResult;

/// ΔΣ - Ontology overlay with proof
#[derive(Debug, Clone)]
pub struct DeltaSigma {
    /// Overlay ID
    pub id: u64,
    /// Base Σ hash (what this overlays)
    pub base_sigma: SigmaHash,
    /// Proposed changes
    pub changes: Vec<SigmaChange>,
    /// Proof sketch (machine-checkable)
    pub proof: ProofSketch,
    /// Priority (for conflict resolution)
    pub priority: u8,
}

/// Type of change in ΔΣ
#[derive(Debug, Clone)]
pub enum SigmaChange {
    /// Add a new task
    AddTask(TaskDescriptor),
    /// Remove a task
    RemoveTask(u64),  // task_id
    /// Modify a task
    ModifyTask { task_id: u64, new_descriptor: TaskDescriptor },
    /// Add a guard
    AddGuard { guard_id: u16, condition: Vec<u8> },
    /// Modify guard threshold
    ModifyGuardThreshold { guard_id: u16, new_threshold: u64 },
}

/// Proof sketch (justification for ΔΣ)
#[derive(Debug, Clone)]
pub struct ProofSketch {
    /// Invariants checked (Q)
    pub invariants_checked: Vec<InvariantCheck>,
    /// Performance estimates
    pub perf_estimate: PerformanceEstimate,
    /// MAPE-K evidence
    pub mape_evidence: Vec<MapeEvidence>,
    /// Signature (proof author)
    pub signature: [u8; 64],
}

/// Invariant check result
#[derive(Debug, Clone)]
pub struct InvariantCheck {
    /// Invariant name
    pub name: String,
    /// Check result
    pub result: GuardResult,
    /// Evidence (why it holds)
    pub evidence: String,
}

/// Performance estimate
#[derive(Debug, Clone)]
pub struct PerformanceEstimate {
    /// Estimated tick usage (max)
    pub max_ticks: u64,
    /// Expected improvement (0.0 - 1.0)
    pub expected_improvement: f64,
    /// Confidence (0.0 - 1.0)
    pub confidence: f64,
}

/// MAPE-K evidence for overlay
#[derive(Debug, Clone)]
pub struct MapeEvidence {
    /// Evidence type
    pub evidence_type: MapePhase,
    /// Metric name
    pub metric: String,
    /// Value
    pub value: f64,
}

/// MAPE-K phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapePhase {
    Monitor,
    Analyze,
    Plan,
    Execute,
}

/// Overlay algebra operations
pub trait OverlayAlgebra {
    /// Compose overlays (ΔΣ₁ ∘ ΔΣ₂)
    fn compose(&self, other: &DeltaSigma) -> Result<DeltaSigma, OverlayError>;

    /// Apply overlay to Σ (Σ ⊕ ΔΣ → Σ')
    fn apply_to(&self, sigma: &SigmaCompiled) -> Result<SigmaCompiled, OverlayError>;

    /// Validate overlay (check proof sketch)
    fn validate(&self) -> Result<(), OverlayError>;

    /// Conflicts with another overlay
    fn conflicts_with(&self, other: &DeltaSigma) -> bool;
}

impl OverlayAlgebra for DeltaSigma {
    fn compose(&self, other: &DeltaSigma) -> Result<DeltaSigma, OverlayError> {
        // Check if overlays can be composed
        if self.base_sigma != other.base_sigma {
            return Err(OverlayError::IncompatibleBase);
        }

        // Merge changes (simplified)
        let mut composed_changes = self.changes.clone();
        composed_changes.extend(other.changes.clone());

        Ok(DeltaSigma {
            id: self.id + other.id,  // Simplified
            base_sigma: self.base_sigma,
            changes: composed_changes,
            proof: self.proof.clone(),  // Would merge proofs
            priority: self.priority.max(other.priority),
        })
    }

    fn apply_to(&self, sigma: &SigmaCompiled) -> Result<SigmaCompiled, OverlayError> {
        // Validate first
        self.validate()?;

        // Create new Σ' (immutable update)
        let mut new_sigma = *sigma;

        // Apply each change
        for change in &self.changes {
            match change {
                SigmaChange::AddTask(_task) => {
                    // Would add task to new_sigma
                }
                SigmaChange::RemoveTask(_id) => {
                    // Would mark task as removed
                }
                SigmaChange::ModifyTask { .. } => {
                    // Would modify task descriptor
                }
                _ => {}
            }
        }

        Ok(new_sigma)
    }

    fn validate(&self) -> Result<(), OverlayError> {
        // Check all invariants hold
        for check in &self.proof.invariants_checked {
            if check.result != GuardResult::Pass {
                return Err(OverlayError::InvariantViolation(check.name.clone()));
            }
        }

        // Check performance estimate
        if self.proof.perf_estimate.max_ticks > crate::CHATMAN_CONSTANT {
            return Err(OverlayError::PerformanceViolation);
        }

        Ok(())
    }

    fn conflicts_with(&self, other: &DeltaSigma) -> bool {
        // Simplified conflict detection
        for change in &self.changes {
            for other_change in &other.changes {
                match (change, other_change) {
                    (SigmaChange::ModifyTask { task_id: id1, .. },
                     SigmaChange::ModifyTask { task_id: id2, .. }) => {
                        if id1 == id2 {
                            return true;  // Both modify same task
                        }
                    }
                    _ => {}
                }
            }
        }

        false
    }
}

/// Overlay errors
#[derive(Debug, Clone)]
pub enum OverlayError {
    /// Incompatible base Σ
    IncompatibleBase,
    /// Invariant violation
    InvariantViolation(String),
    /// Performance violation
    PerformanceViolation,
    /// Conflicting changes
    ConflictingChanges,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_validation() {
        let delta = DeltaSigma {
            id: 1,
            base_sigma: SigmaHash([1; 32]),
            changes: vec![],
            proof: ProofSketch {
                invariants_checked: vec![
                    InvariantCheck {
                        name: "tick_budget".to_string(),
                        result: GuardResult::Pass,
                        evidence: "all tasks ≤8 ticks".to_string(),
                    }
                ],
                perf_estimate: PerformanceEstimate {
                    max_ticks: 6,
                    expected_improvement: 0.3,
                    confidence: 0.9,
                },
                mape_evidence: vec![],
                signature: [0; 64],
            },
            priority: 10,
        };

        assert!(delta.validate().is_ok());
    }
}
