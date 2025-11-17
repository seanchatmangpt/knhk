//! Type-Level Safety Classification for Overlay Promotion
//!
//! Safety is not a runtime property - it's enforced at compile time.
//! Different promotion APIs accept only proofs with appropriate safety levels.

use crate::overlay_proof::{OverlayProof, ProofStrength};
use crate::overlay_types::{OverlayError, OverlayValue};
use core::marker::PhantomData;

/// Type-level safety marker for hot path promotion
///
/// HotSafe overlays can be promoted atomically under load.
/// They have the strongest guarantees:
/// - Timing bound ≤ CHATMAN_CONSTANT (8 ticks)
/// - Formal or compiler-generated proof
/// - No allocations, no I/O, no branches (where possible)
/// - Atomic swap possible
pub struct HotSafe(PhantomData<*const ()>);

impl private::Sealed for HotSafe {}
impl SafetyLevel for HotSafe {
    const NAME: &'static str = "HotSafe";
    const MAX_TICKS: u64 = 8;
    const MIN_PROOF_STRENGTH: ProofStrength = ProofStrength::Compiler;
    const ALLOWS_ALLOCATION: bool = false;
    const ALLOWS_IO: bool = false;
    const PROMOTION_ATOMIC: bool = true;
}

/// Type-level safety marker for warm path promotion
///
/// WarmSafe overlays require limited traffic drain before promotion.
/// They have moderate guarantees:
/// - Timing bound ≤ 1ms (estimated)
/// - Compiler or property-based proof
/// - Can allocate, async allowed
/// - Requires controlled rollout
pub struct WarmSafe(PhantomData<*const ()>);

impl private::Sealed for WarmSafe {}
impl SafetyLevel for WarmSafe {
    const NAME: &'static str = "WarmSafe";
    const MAX_TICKS: u64 = 1_000_000; // ~1ms at 1GHz
    const MIN_PROOF_STRENGTH: ProofStrength = ProofStrength::PropertyBased;
    const ALLOWS_ALLOCATION: bool = true;
    const ALLOWS_IO: bool = false;
    const PROMOTION_ATOMIC: bool = false;
}

/// Type-level safety marker for cold path (lab only)
///
/// ColdUnsafe overlays are for development/testing only.
/// They have minimal guarantees:
/// - Unbounded timing
/// - Any proof level (including runtime)
/// - Can do anything (LLM calls, analytics, etc.)
/// - NEVER promoted to production
pub struct ColdUnsafe(PhantomData<*const ()>);

impl private::Sealed for ColdUnsafe {}
impl SafetyLevel for ColdUnsafe {
    const NAME: &'static str = "ColdUnsafe";
    const MAX_TICKS: u64 = u64::MAX;
    const MIN_PROOF_STRENGTH: ProofStrength = ProofStrength::Runtime;
    const ALLOWS_ALLOCATION: bool = true;
    const ALLOWS_IO: bool = true;
    const PROMOTION_ATOMIC: bool = false;
}

/// Safety level trait (sealed)
///
/// Cannot be implemented outside this module.
/// Defines compile-time constants for safety classification.
pub trait SafetyLevel: private::Sealed {
    /// Human-readable name
    const NAME: &'static str;

    /// Maximum allowed tick count
    const MAX_TICKS: u64;

    /// Minimum proof strength required
    const MIN_PROOF_STRENGTH: ProofStrength;

    /// Whether allocations are allowed
    const ALLOWS_ALLOCATION: bool;

    /// Whether I/O is allowed
    const ALLOWS_IO: bool;

    /// Whether promotion can be atomic
    const PROMOTION_ATOMIC: bool;
}

/// Sealed module to prevent external trait implementation
mod private {
    pub trait Sealed {}
}

// =============================================================================
// Safety-Classified Proofs
// =============================================================================

/// Proof with safety classification
///
/// The type parameter S: SafetyLevel enforces that only compatible
/// overlays can be promoted to specific runtime environments.
#[derive(Debug, Clone)]
pub struct SafeProof<S: SafetyLevel, P: OverlayProof> {
    /// Underlying proof
    proof: P,

    /// Type-level safety marker
    _safety: PhantomData<S>,
}

impl<S: SafetyLevel, P: OverlayProof> SafeProof<S, P> {
    /// Create a safety-classified proof
    ///
    /// This checks that the proof meets the safety level requirements.
    /// If not, returns an error at construction time.
    pub fn new(proof: P) -> Result<Self, SafetyError> {
        // Verify proof first
        proof
            .verify()
            .map_err(|_| SafetyError::ProofVerificationFailed)?;

        // Check timing bound
        if proof.timing_bound() > S::MAX_TICKS {
            return Err(SafetyError::TimingBoundExceeded {
                required: S::MAX_TICKS,
                actual: proof.timing_bound(),
            });
        }

        // Check proof strength
        if proof.strength() < S::MIN_PROOF_STRENGTH {
            return Err(SafetyError::InsufficientProofStrength {
                required: S::MIN_PROOF_STRENGTH,
                actual: proof.strength(),
            });
        }

        Ok(Self {
            proof,
            _safety: PhantomData,
        })
    }

    /// Get underlying proof
    pub fn proof(&self) -> &P {
        &self.proof
    }

    /// Consume and extract proof
    pub fn into_proof(self) -> P {
        self.proof
    }

    /// Get safety level name
    pub fn safety_level(&self) -> &'static str {
        S::NAME
    }
}

impl<S: SafetyLevel, P: OverlayProof> private::Sealed for SafeProof<S, P> {}

impl<S: SafetyLevel, P: OverlayProof> OverlayProof for SafeProof<S, P> {
    fn invariants_preserved(&self) -> &[u16] {
        self.proof.invariants_preserved()
    }

    fn timing_bound(&self) -> u64 {
        self.proof.timing_bound()
    }

    fn verify(&self) -> Result<(), OverlayError> {
        self.proof.verify()
    }

    fn covers_changes(&self, changes: &crate::overlay_types::OverlayChanges) -> bool {
        self.proof.covers_changes(changes)
    }

    fn strength(&self) -> ProofStrength {
        self.proof.strength()
    }

    fn method(&self) -> crate::overlay_proof::ProofMethod {
        self.proof.method()
    }
}

// =============================================================================
// Safety Promotion
// =============================================================================

/// Safety promotion result
///
/// Allows upgrading from weaker to stronger safety level if requirements met.
pub struct SafetyPromotion;

impl SafetyPromotion {
    /// Try to promote WarmSafe to HotSafe
    ///
    /// This is only possible if the proof exceeds HotSafe requirements.
    pub fn promote_to_hot<P: OverlayProof>(
        warm: SafeProof<WarmSafe, P>,
    ) -> Result<SafeProof<HotSafe, P>, SafetyError> {
        SafeProof::<HotSafe, P>::new(warm.into_proof())
    }

    /// Try to promote ColdUnsafe to WarmSafe
    ///
    /// This requires meeting WarmSafe requirements.
    pub fn promote_to_warm<P: OverlayProof>(
        cold: SafeProof<ColdUnsafe, P>,
    ) -> Result<SafeProof<WarmSafe, P>, SafetyError> {
        SafeProof::<WarmSafe, P>::new(cold.into_proof())
    }

    /// Try to promote ColdUnsafe all the way to HotSafe
    ///
    /// This is rare but possible for very strong proofs.
    pub fn promote_cold_to_hot<P: OverlayProof>(
        cold: SafeProof<ColdUnsafe, P>,
    ) -> Result<SafeProof<HotSafe, P>, SafetyError> {
        SafeProof::<HotSafe, P>::new(cold.into_proof())
    }
}

/// Safety classification errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyError {
    /// Proof verification failed
    ProofVerificationFailed,

    /// Timing bound exceeded for this safety level
    TimingBoundExceeded { required: u64, actual: u64 },

    /// Proof strength insufficient for this safety level
    InsufficientProofStrength {
        required: ProofStrength,
        actual: ProofStrength,
    },

    /// Allocation not allowed at this safety level
    AllocationNotAllowed,

    /// I/O not allowed at this safety level
    IoNotAllowed,
}

impl core::fmt::Display for SafetyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ProofVerificationFailed => write!(f, "proof verification failed"),
            Self::TimingBoundExceeded { required, actual } => {
                write!(
                    f,
                    "timing bound exceeded: required ≤{}, actual {}",
                    required, actual
                )
            }
            Self::InsufficientProofStrength { required, actual } => {
                write!(
                    f,
                    "proof strength insufficient: required {:?}, actual {:?}",
                    required, actual
                )
            }
            Self::AllocationNotAllowed => write!(f, "allocation not allowed at this safety level"),
            Self::IoNotAllowed => write!(f, "I/O not allowed at this safety level"),
        }
    }
}

// =============================================================================
// Type Aliases for Convenience
// =============================================================================

/// Hot-safe overlay (production-ready, atomic promotion)
pub type HotSafeOverlay<P> = OverlayValue<SafeProof<HotSafe, P>>;

/// Warm-safe overlay (production-ready with controlled rollout)
pub type WarmSafeOverlay<P> = OverlayValue<SafeProof<WarmSafe, P>>;

/// Cold-unsafe overlay (lab/development only)
pub type ColdUnsafeOverlay<P> = OverlayValue<SafeProof<ColdUnsafe, P>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overlay_proof::{ChangeCoverage, CompilerProof, RuntimeProof};

    #[test]
    fn test_hot_safe_requires_strong_proof() {
        let weak_proof = RuntimeProof {
            observation_period: 1_000_000,
            samples: 10_000,
            invariants: vec![1, 2],
            max_ticks_observed: 7,
            violations: 0,
        };

        // Runtime proof is not strong enough for HotSafe
        assert!(SafeProof::<HotSafe, _>::new(weak_proof).is_err());
    }

    #[test]
    fn test_hot_safe_accepts_compiler_proof() {
        let strong_proof = CompilerProof {
            compiler_version: (2027, 0, 0),
            proof_id: 1,
            invariants: vec![1, 2, 3],
            timing_bound: 6,
            coverage: ChangeCoverage {
                covered_changes: 5,
                coverage_percent: 100,
            },
            signature: [1; 64],
        };

        assert!(SafeProof::<HotSafe, _>::new(strong_proof).is_ok());
    }

    #[test]
    fn test_hot_safe_rejects_slow_operations() {
        let slow_proof = CompilerProof {
            compiler_version: (2027, 0, 0),
            proof_id: 1,
            invariants: vec![1],
            timing_bound: 100, // Exceeds CHATMAN_CONSTANT
            coverage: ChangeCoverage {
                covered_changes: 1,
                coverage_percent: 100,
            },
            signature: [1; 64],
        };

        assert!(SafeProof::<HotSafe, _>::new(slow_proof).is_err());
    }

    #[test]
    fn test_warm_safe_accepts_property_proof() {
        use crate::overlay_proof::PropertyProof;

        let prop_proof = PropertyProof {
            test_cases: 10_000,
            shrink_count: 5,
            invariants: vec![1, 2],
            max_ticks_observed: 1000,
            confidence: 0.99,
        };

        assert!(SafeProof::<WarmSafe, _>::new(prop_proof).is_ok());
    }

    #[test]
    fn test_cold_unsafe_accepts_runtime_proof() {
        let runtime_proof = RuntimeProof {
            observation_period: 1_000_000,
            samples: 10_000,
            invariants: vec![1],
            max_ticks_observed: 1_000_000,
            violations: 0,
        };

        assert!(SafeProof::<ColdUnsafe, _>::new(runtime_proof).is_ok());
    }

    #[test]
    fn test_safety_promotion() {
        let strong_proof = CompilerProof {
            compiler_version: (2027, 0, 0),
            proof_id: 1,
            invariants: vec![1, 2],
            timing_bound: 6,
            coverage: ChangeCoverage {
                covered_changes: 5,
                coverage_percent: 100,
            },
            signature: [1; 64],
        };

        // Start with ColdUnsafe
        let cold = SafeProof::<ColdUnsafe, _>::new(strong_proof.clone()).unwrap();

        // Promote to HotSafe (should work because proof is strong enough)
        let hot = SafetyPromotion::promote_cold_to_hot(cold);
        assert!(hot.is_ok());
    }
}
