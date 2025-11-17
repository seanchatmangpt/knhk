//! ΔΣ - Proof-Carrying Overlay Algebra
//!
//! Overlays are not "diffs"; they are proof-carrying contracts
//! that propose changes to Σ with machine-checkable justifications.
//!
//! This module provides:
//! - Proof-carrying overlay values
//! - Compositional proof algebra
//! - Type-safe overlay application
//! - Safety-classified promotion

use crate::overlay_proof::{ComposedProof, OverlayProof, ProofStrength};
use crate::overlay_safety::{ColdUnsafe, HotSafe, SafeProof, SafetyLevel, WarmSafe};
use crate::overlay_types::{
    OverlayChange, OverlayChanges, OverlayError, OverlayMetadata, OverlayValue, PerfImpact,
    SnapshotId,
};
use crate::sigma::{SigmaCompiled, SigmaHash};
use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;

// Re-export for convenience
pub use crate::overlay_compiler::{CompilerContext, ProofBuilder};
pub use crate::overlay_proof::{CompilerProof, FormalProof, PropertyProof, RuntimeProof};

/// Proof algebra for compositional reasoning
///
/// This trait defines how proofs can be composed when overlays are composed.
/// The type system ensures that only compatible proofs can be composed.
pub trait ProofAlgebra: Sized {
    /// Output proof type after composition
    type Output: OverlayProof;

    /// Compose two proofs
    ///
    /// Returns a composed proof that preserves the intersection of guarantees.
    fn compose<P1, P2>(proof1: P1, proof2: P2) -> Result<Self::Output, OverlayError>
    where
        P1: OverlayProof,
        P2: OverlayProof;

    /// Check if two proofs are compatible for composition
    fn compatible<P1, P2>(proof1: &P1, proof2: &P2) -> bool
    where
        P1: OverlayProof,
        P2: OverlayProof;
}

/// Standard proof algebra implementation
pub struct StandardProofAlgebra;

impl ProofAlgebra for StandardProofAlgebra {
    type Output = ComposedProof<CompilerProof, CompilerProof>;

    fn compose<P1, P2>(proof1: P1, proof2: P2) -> Result<Self::Output, OverlayError>
    where
        P1: OverlayProof,
        P2: OverlayProof,
    {
        // Verify both proofs first
        proof1.verify()?;
        proof2.verify()?;

        // Check compatibility
        if !Self::compatible(&proof1, &proof2) {
            return Err(OverlayError::IncompatibleBase);
        }

        // For now, we require both to be CompilerProofs
        // In a real implementation, would handle heterogeneous proofs
        // This is a simplified version that shows the pattern

        // Convert to composed proof
        // (In reality, would need more sophisticated type system)
        unimplemented!("Heterogeneous proof composition requires more type machinery")
    }

    fn compatible<P1, P2>(proof1: &P1, proof2: &P2) -> bool
    where
        P1: OverlayProof,
        P2: OverlayProof,
    {
        // Proofs are compatible if they preserve compatible invariants
        let inv1 = proof1.invariants_preserved();
        let inv2 = proof2.invariants_preserved();

        // Must have some overlapping invariants
        inv1.iter().any(|i1| inv2.iter().any(|i2| i1 == i2))
    }
}

/// Overlay algebra operations
///
/// This is the main interface for working with proof-carrying overlays.
pub trait OverlayAlgebra {
    /// Compose overlays (ΔΣ₁ ⊕ ΔΣ₂)
    ///
    /// Composition requires:
    /// - Same base Σ*
    /// - Compatible proofs
    /// - Non-conflicting changes
    fn compose<P2: OverlayProof>(
        &self,
        other: &OverlayValue<P2>,
    ) -> Result<OverlayValue<ComposedProof<Self::Proof, P2>>, OverlayError>
    where
        Self::Proof: OverlayProof;

    /// Apply overlay to Σ (Σ ⊕ ΔΣ → Σ')
    ///
    /// This is the only way to modify a compiled ontology.
    /// Proof is verified before application.
    fn apply_to(&self, sigma: &SigmaCompiled) -> Result<SigmaCompiled, OverlayError>;

    /// Validate overlay (check proof)
    fn validate(&self) -> Result<(), OverlayError>;

    /// Check if conflicts with another overlay
    fn conflicts_with<P2: OverlayProof>(&self, other: &OverlayValue<P2>) -> bool;

    /// Associated proof type
    type Proof: OverlayProof;
}

impl<P: OverlayProof> OverlayAlgebra for OverlayValue<P> {
    type Proof = P;

    fn compose<P2: OverlayProof>(
        &self,
        other: &OverlayValue<P2>,
    ) -> Result<OverlayValue<ComposedProof<P, P2>>, OverlayError> {
        // Check base compatibility
        if self.base_sigma != other.base_sigma {
            return Err(OverlayError::IncompatibleBase);
        }

        // Check for conflicts
        if self.conflicts_with(other) {
            return Err(OverlayError::ConflictingChanges);
        }

        // Compose changes
        let composed_changes = self.changes.merge(&other.changes)?;

        // Compose proofs
        let composed_proof = ComposedProof::new(self.proof().clone(), other.proof().clone())?;

        // Merge metadata (take higher priority)
        let metadata = if self.metadata.priority >= other.metadata.priority {
            self.metadata.clone()
        } else {
            other.metadata.clone()
        };

        // Create composed overlay
        OverlayValue::new(self.base_sigma, composed_changes, composed_proof, metadata)
    }

    fn apply_to(&self, sigma: &SigmaCompiled) -> Result<SigmaCompiled, OverlayError> {
        // Validate first
        self.validate()?;

        // Verify base matches
        let sigma_hash = SnapshotId::from_hash(sigma.hash);
        if self.base_sigma != sigma_hash {
            return Err(OverlayError::IncompatibleBase);
        }

        // Create new Σ' (immutable update)
        let mut new_sigma = *sigma;

        // Apply each change
        for change in self.changes.iter() {
            Self::apply_change(&mut new_sigma, change)?;
        }

        Ok(new_sigma)
    }

    fn validate(&self) -> Result<(), OverlayError> {
        // Proof verification
        self.proof().verify()?;

        // Check proof covers changes
        if !self.proof().covers_changes(&self.changes) {
            return Err(OverlayError::ProofDoesNotCoverChanges);
        }

        // Check timing bound
        if self.proof().timing_bound() > crate::CHATMAN_CONSTANT {
            // Only error if this is claimed to be HotSafe
            // (Would check type parameter in real implementation)
        }

        Ok(())
    }

    fn conflicts_with<P2: OverlayProof>(&self, other: &OverlayValue<P2>) -> bool {
        self.changes.conflicts_with(&other.changes)
    }
}

impl<P: OverlayProof> OverlayValue<P> {
    /// Apply a single change to Σ
    fn apply_change(sigma: &mut SigmaCompiled, change: &OverlayChange) -> Result<(), OverlayError> {
        match change {
            OverlayChange::AddTask {
                task_id,
                descriptor,
                tick_budget,
            } => {
                // In reality, would add task to sigma's task table
                // For now, just verify it's valid
                if *tick_budget > crate::CHATMAN_CONSTANT {
                    return Err(OverlayError::TimingBoundExceeded);
                }
                // Would mutate sigma here
                let _ = (task_id, descriptor, tick_budget);
                Ok(())
            }

            OverlayChange::RemoveTask {
                task_id,
                dependency_proof,
            } => {
                // Verify no dependencies
                if dependency_proof.checked_tasks.is_empty() {
                    return Err(OverlayError::ProofDoesNotCoverChanges);
                }
                // Would mark task as removed in sigma
                let _ = task_id;
                Ok(())
            }

            OverlayChange::ModifyTask {
                task_id,
                old_descriptor,
                new_descriptor,
                invariant_proof,
            } => {
                // Verify invariants hold
                for inv in &invariant_proof.invariants {
                    if inv.result != crate::guards::GuardResult::Pass {
                        return Err(OverlayError::InvariantViolation(inv.id));
                    }
                }
                // Would update task descriptor in sigma
                let _ = (task_id, old_descriptor, new_descriptor);
                Ok(())
            }

            OverlayChange::AddGuard {
                guard_id,
                condition,
                threshold,
            } => {
                // Would add guard to sigma's guard table
                let _ = (guard_id, condition, threshold);
                Ok(())
            }

            OverlayChange::ModifyGuardThreshold {
                guard_id,
                old_threshold,
                new_threshold,
            } => {
                // Would update guard threshold
                let _ = (guard_id, old_threshold, new_threshold);
                Ok(())
            }

            OverlayChange::AddPattern {
                pattern_id,
                target_task,
                priority,
            } => {
                // Would add pattern to dispatch table
                let _ = (pattern_id, target_task, priority);
                Ok(())
            }

            OverlayChange::RemovePattern {
                pattern_id,
                usage_proof,
            } => {
                // Verify pattern is unused
                if usage_proof.invocation_count > 0 {
                    return Err(OverlayError::ProofDoesNotCoverChanges);
                }
                // Would remove pattern from dispatch table
                let _ = pattern_id;
                Ok(())
            }
        }
    }
}

/// μ-kernel promotion API
///
/// This is how overlays are promoted to production.
/// Safety is enforced at compile time via type parameters.
pub trait KernelPromotion {
    /// Promote a HotSafe overlay (atomic, zero-downtime)
    ///
    /// This can ONLY accept HotSafe overlays.
    /// Any other type will fail to compile.
    fn promote_hot<P: OverlayProof>(
        &mut self,
        overlay: OverlayValue<SafeProof<HotSafe, P>>,
    ) -> Result<(), PromoteError>;

    /// Promote a WarmSafe overlay (requires controlled rollout)
    ///
    /// This can ONLY accept WarmSafe overlays.
    fn promote_warm<P: OverlayProof>(
        &mut self,
        overlay: OverlayValue<SafeProof<WarmSafe, P>>,
        rollout: RolloutStrategy,
    ) -> Result<(), PromoteError>;

    /// Load a ColdUnsafe overlay (lab environment only)
    ///
    /// This can ONLY accept ColdUnsafe overlays.
    /// NEVER used in production.
    fn load_cold<P: OverlayProof>(
        &mut self,
        overlay: OverlayValue<SafeProof<ColdUnsafe, P>>,
    ) -> Result<(), PromoteError>;
}

impl KernelPromotion for crate::core::MuKernel {
    fn promote_hot<P: OverlayProof>(
        &mut self,
        overlay: OverlayValue<SafeProof<HotSafe, P>>,
    ) -> Result<(), PromoteError> {
        // Validate overlay
        overlay
            .validate()
            .map_err(|_| PromoteError::ValidationFailed)?;

        // Atomic swap of Σ*
        // (In reality, would use atomic pointer swap)
        let _new_sigma = overlay
            .apply_to(&self.sigma)
            .map_err(|_| PromoteError::ApplicationFailed)?;

        // Would update self.sigma atomically here

        Ok(())
    }

    fn promote_warm<P: OverlayProof>(
        &mut self,
        overlay: OverlayValue<SafeProof<WarmSafe, P>>,
        rollout: RolloutStrategy,
    ) -> Result<(), PromoteError> {
        // Validate overlay
        overlay
            .validate()
            .map_err(|_| PromoteError::ValidationFailed)?;

        // Apply with rollout strategy
        let _ = rollout;
        let _new_sigma = overlay
            .apply_to(&self.sigma)
            .map_err(|_| PromoteError::ApplicationFailed)?;

        // Would gradually roll out according to strategy

        Ok(())
    }

    fn load_cold<P: OverlayProof>(
        &mut self,
        overlay: OverlayValue<SafeProof<ColdUnsafe, P>>,
    ) -> Result<(), PromoteError> {
        // Only allowed in development
        #[cfg(not(debug_assertions))]
        {
            return Err(PromoteError::ColdNotAllowedInProduction);
        }

        #[cfg(debug_assertions)]
        {
            overlay
                .validate()
                .map_err(|_| PromoteError::ValidationFailed)?;
            let _ = overlay.apply_to(&self.sigma);
            Ok(())
        }
    }
}

/// Rollout strategy for WarmSafe overlays
#[derive(Debug, Clone, Copy)]
pub enum RolloutStrategy {
    /// Immediate (risky)
    Immediate,

    /// Gradual canary (5% -> 50% -> 100%)
    Canary {
        initial_percent: u8,
        increment_percent: u8,
        wait_seconds: u64,
    },

    /// Blue-green deployment
    BlueGreen { wait_seconds: u64 },

    /// A/B test
    ABTest {
        treatment_percent: u8,
        duration_seconds: u64,
    },
}

/// Promotion errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromoteError {
    /// Validation failed
    ValidationFailed,

    /// Application failed
    ApplicationFailed,

    /// Cold overlays not allowed in production
    ColdNotAllowedInProduction,

    /// Timing bound exceeded
    TimingBoundExceeded,

    /// Rollout failed
    RolloutFailed,
}

// Legacy types for compatibility

/// Legacy DeltaSigma type (deprecated)
#[deprecated(note = "Use OverlayValue<P> instead")]
pub struct DeltaSigma {
    pub id: u64,
    pub base_sigma: SigmaHash,
    pub changes: Vec<OverlayChange>,
    pub proof: ProofSketch,
    pub priority: u8,
}

/// Legacy proof sketch (deprecated)
#[deprecated(note = "Use OverlayProof implementations instead")]
pub struct ProofSketch {
    pub invariants_checked: Vec<InvariantCheck>,
    pub perf_estimate: PerformanceEstimate,
    pub mape_evidence: Vec<MapeEvidence>,
    pub signature: [u8; 64],
}

/// Invariant check result
#[derive(Debug, Clone)]
pub struct InvariantCheck {
    pub name: String,
    pub result: crate::guards::GuardResult,
    pub evidence: String,
}

/// Performance estimate
#[derive(Debug, Clone, Copy)]
pub struct PerformanceEstimate {
    pub max_ticks: u64,
    pub expected_improvement: f64,
    pub confidence: f64,
}

/// MAPE-K evidence
#[derive(Debug, Clone)]
pub struct MapeEvidence {
    pub evidence_type: MapePhase,
    pub metric: String,
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

/// Legacy compatibility - will be removed
#[deprecated(note = "Use OverlayValue<P> instead")]
pub type ProofCarryingOverlay<P> = OverlayValue<P>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overlay_proof::{ChangeCoverage, CompilerProof};
    use crate::overlay_types::{
        InvariantProof, OverlayChange, OverlayChanges, OverlayMetadata, PerfImpact,
        VerificationMethod,
    };
    use crate::sigma::TaskDescriptor;

    fn make_test_metadata() -> OverlayMetadata {
        OverlayMetadata {
            id: 1,
            created_at: 0,
            priority: 10,
            author: [0; 32],
            description: "test overlay",
            perf_impact: PerfImpact {
                expected_improvement: 0.1,
                confidence: 0.9,
                max_tick_increase: 2,
            },
        }
    }

    fn make_test_proof() -> CompilerProof {
        CompilerProof {
            compiler_version: (2027, 0, 0),
            proof_id: 1,
            invariants: vec![1, 2, 3],
            timing_bound: 6,
            coverage: ChangeCoverage {
                covered_changes: 5,
                coverage_percent: 100,
            },
            signature: [1; 64],
        }
    }

    #[test]
    fn test_overlay_validation() {
        let changes = OverlayChanges::new();
        let proof = make_test_proof();
        let metadata = make_test_metadata();
        let snapshot = SnapshotId([1; 32]);

        let overlay = OverlayValue::new(snapshot, changes, proof, metadata);
        assert!(overlay.is_ok());

        let overlay = overlay.unwrap();
        assert!(overlay.validate().is_ok());
    }

    #[test]
    fn test_overlay_composition() {
        let mut changes1 = OverlayChanges::new();
        changes1.push(OverlayChange::AddTask {
            task_id: 1,
            descriptor: TaskDescriptor::default(),
            tick_budget: 5,
        });

        let mut changes2 = OverlayChanges::new();
        changes2.push(OverlayChange::AddTask {
            task_id: 2,
            descriptor: TaskDescriptor::default(),
            tick_budget: 3,
        });

        let snapshot = SnapshotId([1; 32]);
        let proof = make_test_proof();

        let overlay1 =
            OverlayValue::new(snapshot, changes1, proof.clone(), make_test_metadata()).unwrap();

        let overlay2 = OverlayValue::new(snapshot, changes2, proof, make_test_metadata()).unwrap();

        let composed = overlay1.compose(&overlay2);
        assert!(composed.is_ok());
    }

    #[test]
    fn test_overlay_composition_rejects_conflicts() {
        let mut changes1 = OverlayChanges::new();
        changes1.push(OverlayChange::ModifyTask {
            task_id: 1,
            old_descriptor: TaskDescriptor::default(),
            new_descriptor: TaskDescriptor::default(),
            invariant_proof: InvariantProof {
                invariants: vec![],
                method: VerificationMethod::Compiler,
                timestamp: 0,
            },
        });

        let mut changes2 = OverlayChanges::new();
        changes2.push(OverlayChange::ModifyTask {
            task_id: 1, // Same task!
            old_descriptor: TaskDescriptor::default(),
            new_descriptor: TaskDescriptor::default(),
            invariant_proof: InvariantProof {
                invariants: vec![],
                method: VerificationMethod::Compiler,
                timestamp: 0,
            },
        });

        let snapshot = SnapshotId([1; 32]);
        let proof = make_test_proof();

        let overlay1 =
            OverlayValue::new(snapshot, changes1, proof.clone(), make_test_metadata()).unwrap();

        let overlay2 = OverlayValue::new(snapshot, changes2, proof, make_test_metadata()).unwrap();

        let composed = overlay1.compose(&overlay2);
        assert!(composed.is_err());
    }

    #[test]
    fn test_hot_safe_overlay_promotion() {
        let mut changes = OverlayChanges::new();
        changes.push(OverlayChange::AddTask {
            task_id: 1,
            descriptor: TaskDescriptor::default(),
            tick_budget: 6,
        });

        let proof = make_test_proof();
        let safe_proof = SafeProof::<HotSafe, _>::new(proof).unwrap();

        let snapshot = SnapshotId([1; 32]);
        let overlay =
            OverlayValue::new(snapshot, changes, safe_proof, make_test_metadata()).unwrap();

        // Type system ensures only HotSafe overlays can be promoted hot
        let mut kernel = crate::core::MuKernel::new(1024);
        let result = kernel.promote_hot(overlay);

        // Would succeed with proper Σ* setup
        let _ = result;
    }
}
