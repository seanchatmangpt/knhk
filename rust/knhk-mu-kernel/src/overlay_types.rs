//! Overlay Types - Typed, Immutable ΔΣ Values
//!
//! Overlays are first-class values with type-level proof obligations.
//! They cannot be applied without a valid proof, enforced at compile time.

use crate::guards::GuardResult;
use crate::overlay_proof::OverlayProof;
use crate::sigma::{SigmaHash, TaskDescriptor};
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::marker::PhantomData;

/// Snapshot ID for base Σ*
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SnapshotId(pub [u8; 32]);

impl SnapshotId {
    /// Create from SigmaHash
    pub const fn from_hash(hash: SigmaHash) -> Self {
        Self(hash.0)
    }

    /// Convert to SigmaHash
    pub const fn to_hash(&self) -> SigmaHash {
        SigmaHash(self.0)
    }
}

/// Overlay changes at IR level
///
/// These are not "diffs" but structured, typed changes to the ontology.
/// Each change type carries enough information for both application and verification.
#[derive(Debug, Clone)]
pub enum OverlayChange {
    /// Add a new task with full descriptor
    AddTask {
        task_id: u64,
        descriptor: TaskDescriptor,
        /// Expected tick budget for this task
        tick_budget: u64,
    },

    /// Remove an existing task
    RemoveTask {
        task_id: u64,
        /// Proof that no dependencies exist
        dependency_proof: DependencyProof,
    },

    /// Modify task descriptor (immutable update)
    ModifyTask {
        task_id: u64,
        old_descriptor: TaskDescriptor,
        new_descriptor: TaskDescriptor,
        /// Proof that modification preserves invariants
        invariant_proof: InvariantProof,
    },

    /// Add a new guard
    AddGuard {
        guard_id: u16,
        condition: GuardCondition,
        threshold: u64,
    },

    /// Modify guard threshold (tuning only)
    ModifyGuardThreshold {
        guard_id: u16,
        old_threshold: u64,
        new_threshold: u64,
    },

    /// Add a pattern dispatch entry
    AddPattern {
        pattern_id: u32,
        target_task: u64,
        priority: u8,
    },

    /// Remove a pattern dispatch entry
    RemovePattern {
        pattern_id: u32,
        /// Proof that pattern is unused
        usage_proof: UsageProof,
    },
}

/// Guard condition specification
#[derive(Debug, Clone)]
pub struct GuardCondition {
    /// Guard bytecode (compiled)
    pub bytecode: [u8; 256],
    /// Input arity
    pub arity: u8,
    /// Expected tick count
    pub tick_estimate: u64,
}

/// Proof that a task has no dependencies
#[derive(Debug, Clone)]
pub struct DependencyProof {
    /// Task IDs checked for dependencies
    pub checked_tasks: Vec<u64>,
    /// Proof timestamp (when analysis was done)
    pub timestamp: u64,
    /// Signature over proof
    pub signature: [u8; 64],
}

/// Proof that modification preserves invariants
#[derive(Debug, Clone)]
pub struct InvariantProof {
    /// Invariants checked
    pub invariants: Vec<InvariantCheck>,
    /// Verification method used
    pub method: VerificationMethod,
    /// Proof timestamp
    pub timestamp: u64,
}

/// Invariant check record
#[derive(Debug, Clone)]
pub struct InvariantCheck {
    /// Invariant identifier
    pub id: u16,
    /// Human-readable name
    pub name: &'static str,
    /// Check result
    pub result: GuardResult,
    /// Evidence (why it holds)
    pub evidence: Evidence,
}

/// Evidence for invariant holding
#[derive(Debug, Clone)]
pub enum Evidence {
    /// Formal verification proof
    Formal {
        prover: &'static str,
        proof_hash: [u8; 32],
    },
    /// Exhaustive testing
    Tested {
        test_count: u64,
        coverage: u8, // Percentage
    },
    /// Runtime monitoring
    Monitored { samples: u64, violations: u64 },
    /// Compiler-generated proof
    Compiler {
        compiler_version: (u8, u8, u8),
        proof_id: u64,
    },
}

/// Verification method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationMethod {
    /// Formal verification (strongest)
    Formal,
    /// Property-based testing
    PropertyBased,
    /// Symbolic execution
    Symbolic,
    /// Runtime verification
    Runtime,
    /// Compiler analysis
    Compiler,
}

/// Proof that a pattern is unused
#[derive(Debug, Clone)]
pub struct UsageProof {
    /// Observation period (ticks)
    pub observation_period: u64,
    /// Pattern invocation count
    pub invocation_count: u64,
    /// Proof timestamp
    pub timestamp: u64,
}

/// Immutable overlay value
///
/// This is a zero-copy, type-safe representation of ΔΣ.
/// It can only be constructed with a valid proof.
#[derive(Clone)]
pub struct OverlayValue<P: OverlayProof> {
    /// Base Σ* snapshot this overlays
    pub base_sigma: SnapshotId,

    /// Structured changes at IR level
    pub changes: OverlayChanges,

    /// Associated proof (required, cannot be None)
    proof: P,

    /// Overlay metadata
    pub metadata: OverlayMetadata,

    /// Type marker (zero-sized)
    _marker: PhantomData<fn() -> P>,
}

impl<P: OverlayProof> OverlayValue<P> {
    /// Create a new overlay value with proof
    ///
    /// This is the ONLY way to construct an overlay.
    /// The proof must be provided and verified.
    pub fn new(
        base_sigma: SnapshotId,
        changes: OverlayChanges,
        proof: P,
        metadata: OverlayMetadata,
    ) -> Result<Self, OverlayError> {
        // Verify proof on construction
        proof.verify()?;

        // Verify changes are compatible with proof
        if !proof.covers_changes(&changes) {
            return Err(OverlayError::ProofDoesNotCoverChanges);
        }

        Ok(Self {
            base_sigma,
            changes,
            proof,
            metadata,
            _marker: PhantomData,
        })
    }

    /// Access the proof (immutable)
    pub fn proof(&self) -> &P {
        &self.proof
    }

    /// Get invariants preserved by this overlay
    pub fn invariants_preserved(&self) -> &[u16] {
        self.proof.invariants_preserved()
    }

    /// Get timing bound (maximum ticks)
    pub fn timing_bound(&self) -> u64 {
        self.proof.timing_bound()
    }

    /// Check if overlay is compatible with another
    pub fn compatible_with<P2: OverlayProof>(&self, other: &OverlayValue<P2>) -> bool {
        // Same base
        if self.base_sigma != other.base_sigma {
            return false;
        }

        // No conflicting changes
        !self.changes.conflicts_with(&other.changes)
    }

    /// Decompose into parts (consumes self)
    pub fn into_parts(self) -> (SnapshotId, OverlayChanges, P, OverlayMetadata) {
        (self.base_sigma, self.changes, self.proof, self.metadata)
    }
}

impl<P: OverlayProof> fmt::Debug for OverlayValue<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OverlayValue")
            .field("base_sigma", &self.base_sigma)
            .field("changes", &self.changes)
            .field("metadata", &self.metadata)
            .field("proof_type", &core::any::type_name::<P>())
            .finish()
    }
}

/// Collection of overlay changes
///
/// Zero-copy, bounds-checked access to change set.
#[derive(Debug, Clone)]
pub struct OverlayChanges {
    changes: Vec<OverlayChange>,
}

impl OverlayChanges {
    /// Create empty change set
    pub const fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    /// Create from vector of changes
    pub fn from_vec(changes: Vec<OverlayChange>) -> Self {
        Self { changes }
    }

    /// Add a change
    pub fn push(&mut self, change: OverlayChange) {
        self.changes.push(change);
    }

    /// Get number of changes
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Iterate over changes
    pub fn iter(&self) -> impl Iterator<Item = &OverlayChange> {
        self.changes.iter()
    }

    /// Check if changes conflict with another set
    pub fn conflicts_with(&self, other: &OverlayChanges) -> bool {
        for c1 in &self.changes {
            for c2 in &other.changes {
                if Self::changes_conflict(c1, c2) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if two individual changes conflict
    fn changes_conflict(c1: &OverlayChange, c2: &OverlayChange) -> bool {
        match (c1, c2) {
            // Modifying same task
            (
                OverlayChange::ModifyTask { task_id: id1, .. },
                OverlayChange::ModifyTask { task_id: id2, .. },
            ) => id1 == id2,

            // Removing and modifying same task
            (
                OverlayChange::RemoveTask { task_id: id1, .. },
                OverlayChange::ModifyTask { task_id: id2, .. },
            ) => id1 == id2,
            (
                OverlayChange::ModifyTask { task_id: id1, .. },
                OverlayChange::RemoveTask { task_id: id2, .. },
            ) => id1 == id2,

            // Adding same task ID
            (
                OverlayChange::AddTask { task_id: id1, .. },
                OverlayChange::AddTask { task_id: id2, .. },
            ) => id1 == id2,

            // Modifying same guard
            (
                OverlayChange::ModifyGuardThreshold { guard_id: id1, .. },
                OverlayChange::ModifyGuardThreshold { guard_id: id2, .. },
            ) => id1 == id2,

            _ => false,
        }
    }

    /// Merge with another change set (for composition)
    pub fn merge(&self, other: &OverlayChanges) -> Result<OverlayChanges, OverlayError> {
        if self.conflicts_with(other) {
            return Err(OverlayError::ConflictingChanges);
        }

        let mut merged = self.changes.clone();
        merged.extend(other.changes.clone());

        Ok(OverlayChanges { changes: merged })
    }
}

/// Overlay metadata
#[derive(Debug, Clone)]
pub struct OverlayMetadata {
    /// Overlay ID (unique)
    pub id: u64,

    /// Creation timestamp (ticks since epoch)
    pub created_at: u64,

    /// Priority (for conflict resolution)
    pub priority: u8,

    /// Author (public key hash)
    pub author: [u8; 32],

    /// Description
    pub description: &'static str,

    /// Expected performance impact
    pub perf_impact: PerfImpact,
}

/// Performance impact estimate
#[derive(Debug, Clone, Copy)]
pub struct PerfImpact {
    /// Expected improvement (0.0 - 1.0, or negative for degradation)
    pub expected_improvement: f64,

    /// Confidence (0.0 - 1.0)
    pub confidence: f64,

    /// Maximum tick increase
    pub max_tick_increase: u64,
}

/// Overlay errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayError {
    /// Proof verification failed
    ProofVerificationFailed,

    /// Proof doesn't cover all changes
    ProofDoesNotCoverChanges,

    /// Conflicting changes in overlay
    ConflictingChanges,

    /// Incompatible base Σ
    IncompatibleBase,

    /// Timing bound exceeded
    TimingBoundExceeded,

    /// Invariant violation
    InvariantViolation(u16),
}

impl fmt::Display for OverlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProofVerificationFailed => write!(f, "proof verification failed"),
            Self::ProofDoesNotCoverChanges => write!(f, "proof does not cover all changes"),
            Self::ConflictingChanges => write!(f, "conflicting changes in overlay"),
            Self::IncompatibleBase => write!(f, "incompatible base Σ"),
            Self::TimingBoundExceeded => write!(f, "timing bound exceeded"),
            Self::InvariantViolation(id) => write!(f, "invariant {} violated", id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_id_conversion() {
        let hash = SigmaHash([42; 32]);
        let snapshot = SnapshotId::from_hash(hash);
        assert_eq!(snapshot.to_hash(), hash);
    }

    #[test]
    fn test_overlay_changes_conflict_detection() {
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
            task_id: 1,
            old_descriptor: TaskDescriptor::default(),
            new_descriptor: TaskDescriptor::default(),
            invariant_proof: InvariantProof {
                invariants: vec![],
                method: VerificationMethod::Compiler,
                timestamp: 0,
            },
        });

        assert!(changes1.conflicts_with(&changes2));
    }

    #[test]
    fn test_overlay_changes_no_conflict() {
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
            task_id: 2,
            old_descriptor: TaskDescriptor::default(),
            new_descriptor: TaskDescriptor::default(),
            invariant_proof: InvariantProof {
                invariants: vec![],
                method: VerificationMethod::Compiler,
                timestamp: 0,
            },
        });

        assert!(!changes1.conflicts_with(&changes2));
    }
}
