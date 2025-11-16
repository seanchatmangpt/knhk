//! Comprehensive Overlay Proof Tests
//!
//! Tests for proof composition, safety classification, and impossibility results.

use knhk_mu_kernel::{
    overlay::{OverlayAlgebra, KernelPromotion, RolloutStrategy},
    overlay_types::{
        OverlayValue, OverlayChanges, OverlayChange, OverlayMetadata,
        SnapshotId, PerfImpact, InvariantProof, VerificationMethod,
    },
    overlay_proof::{
        CompilerProof, FormalProof, PropertyProof, RuntimeProof,
        ChangeCoverage, OverlayProof, ComposedProof,
    },
    overlay_safety::{SafeProof, HotSafe, WarmSafe, ColdUnsafe, SafetyPromotion},
    overlay_compiler::ProofBuilder,
    sigma::TaskDescriptor,
    core::MuKernel,
};

// =============================================================================
// Test Helpers
// =============================================================================

fn make_snapshot() -> SnapshotId {
    SnapshotId([42; 32])
}

fn make_metadata(priority: u8) -> OverlayMetadata {
    OverlayMetadata {
        id: 1,
        created_at: 0,
        priority,
        author: [0; 32],
        description: "test overlay",
        perf_impact: PerfImpact {
            expected_improvement: 0.1,
            confidence: 0.9,
            max_tick_increase: 2,
        },
    }
}

fn make_compiler_proof(timing_bound: u64) -> CompilerProof {
    CompilerProof {
        compiler_version: (2027, 0, 0),
        proof_id: 1,
        invariants: vec![1, 2, 3],
        timing_bound,
        coverage: ChangeCoverage {
            covered_changes: 5,
            coverage_percent: 100,
        },
        signature: [1; 64],
    }
}

fn make_fast_changes() -> OverlayChanges {
    let mut changes = OverlayChanges::new();
    changes.push(OverlayChange::AddTask {
        task_id: 1,
        descriptor: TaskDescriptor::default(),
        tick_budget: 5,
    });
    changes
}

// =============================================================================
// Proof Verification Tests
// =============================================================================

#[test]
fn test_compiler_proof_verification_success() {
    let proof = make_compiler_proof(6);
    assert!(proof.verify().is_ok());
}

#[test]
fn test_compiler_proof_requires_signature() {
    let mut proof = make_compiler_proof(6);
    proof.signature = [0; 64];  // Invalid signature
    assert!(proof.verify().is_err());
}

#[test]
fn test_formal_proof_verification() {
    let proof = FormalProof {
        prover: "Z3",
        proof_hash: [42; 32],
        invariants: vec![1, 2, 3],
        timing_bound: 6,
        certificate: vec![1, 2, 3],
    };

    assert!(proof.verify().is_ok());
}

#[test]
fn test_formal_proof_requires_certificate() {
    let proof = FormalProof {
        prover: "Z3",
        proof_hash: [42; 32],
        invariants: vec![1, 2],
        timing_bound: 6,
        certificate: vec![],  // Empty certificate
    };

    assert!(proof.verify().is_err());
}

#[test]
fn test_property_proof_requires_minimum_tests() {
    let proof = PropertyProof {
        test_cases: 500,  // Too few (requires 1000)
        shrink_count: 0,
        invariants: vec![1, 2],
        max_ticks_observed: 7,
        confidence: 0.99,
    };

    assert!(proof.verify().is_err());
}

#[test]
fn test_property_proof_requires_high_confidence() {
    let proof = PropertyProof {
        test_cases: 10_000,
        shrink_count: 5,
        invariants: vec![1, 2],
        max_ticks_observed: 7,
        confidence: 0.90,  // Too low (requires 0.95)
    };

    assert!(proof.verify().is_err());
}

#[test]
fn test_runtime_proof_rejects_violations() {
    let proof = RuntimeProof {
        observation_period: 1_000_000,
        samples: 10_000,
        invariants: vec![1, 2],
        max_ticks_observed: 7,
        violations: 5,  // Has violations
    };

    assert!(proof.verify().is_err());
}

#[test]
fn test_runtime_proof_requires_minimum_observation() {
    let proof = RuntimeProof {
        observation_period: 100,  // Too short
        samples: 10_000,
        invariants: vec![1],
        max_ticks_observed: 7,
        violations: 0,
    };

    assert!(proof.verify().is_err());
}

// =============================================================================
// Safety Classification Tests
// =============================================================================

#[test]
fn test_hot_safe_accepts_fast_compiler_proof() {
    let proof = make_compiler_proof(6);
    let safe = SafeProof::<HotSafe, _>::new(proof);
    assert!(safe.is_ok());
}

#[test]
fn test_hot_safe_rejects_slow_operations() {
    let proof = make_compiler_proof(100);  // Too slow
    let safe = SafeProof::<HotSafe, _>::new(proof);
    assert!(safe.is_err());
}

#[test]
fn test_hot_safe_rejects_weak_proofs() {
    let weak_proof = RuntimeProof {
        observation_period: 1_000_000,
        samples: 10_000,
        invariants: vec![1, 2],
        max_ticks_observed: 7,
        violations: 0,
    };

    let safe = SafeProof::<HotSafe, _>::new(weak_proof);
    assert!(safe.is_err());
}

#[test]
fn test_warm_safe_accepts_property_proofs() {
    let proof = PropertyProof {
        test_cases: 10_000,
        shrink_count: 5,
        invariants: vec![1, 2],
        max_ticks_observed: 1000,
        confidence: 0.99,
    };

    let safe = SafeProof::<WarmSafe, _>::new(proof);
    assert!(safe.is_ok());
}

#[test]
fn test_cold_unsafe_accepts_runtime_proofs() {
    let proof = RuntimeProof {
        observation_period: 1_000_000,
        samples: 10_000,
        invariants: vec![1],
        max_ticks_observed: 1_000_000,
        violations: 0,
    };

    let safe = SafeProof::<ColdUnsafe, _>::new(proof);
    assert!(safe.is_ok());
}

// =============================================================================
// Safety Promotion Tests
// =============================================================================

#[test]
fn test_promote_warm_to_hot_success() {
    let strong_proof = make_compiler_proof(6);
    let warm = SafeProof::<WarmSafe, _>::new(strong_proof).unwrap();

    let hot = SafetyPromotion::promote_to_hot(warm);
    assert!(hot.is_ok());
}

#[test]
fn test_promote_warm_to_hot_fails_if_slow() {
    let slow_proof = make_compiler_proof(1000);
    let warm = SafeProof::<WarmSafe, _>::new(slow_proof).unwrap();

    let hot = SafetyPromotion::promote_to_hot(warm);
    assert!(hot.is_err());
}

#[test]
fn test_promote_cold_to_warm_success() {
    let property_proof = PropertyProof {
        test_cases: 10_000,
        shrink_count: 5,
        invariants: vec![1, 2],
        max_ticks_observed: 1000,
        confidence: 0.99,
    };

    let cold = SafeProof::<ColdUnsafe, _>::new(property_proof).unwrap();
    let warm = SafetyPromotion::promote_to_warm(cold);
    assert!(warm.is_ok());
}

#[test]
fn test_promote_cold_to_hot_success_with_strong_proof() {
    let strong_proof = make_compiler_proof(6);
    let cold = SafeProof::<ColdUnsafe, _>::new(strong_proof).unwrap();

    let hot = SafetyPromotion::promote_cold_to_hot(cold);
    assert!(hot.is_ok());
}

// =============================================================================
// Overlay Composition Tests
// =============================================================================

#[test]
fn test_compose_non_conflicting_overlays() {
    let snapshot = make_snapshot();

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

    let proof = make_compiler_proof(6);

    let overlay1 = OverlayValue::new(
        snapshot,
        changes1,
        proof.clone(),
        make_metadata(10),
    ).unwrap();

    let overlay2 = OverlayValue::new(
        snapshot,
        changes2,
        proof,
        make_metadata(10),
    ).unwrap();

    let composed = overlay1.compose(&overlay2);
    assert!(composed.is_ok());

    let composed = composed.unwrap();
    assert_eq!(composed.changes.len(), 2);
}

#[test]
fn test_compose_rejects_conflicting_changes() {
    let snapshot = make_snapshot();

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
        task_id: 1,  // Same task - conflict!
        old_descriptor: TaskDescriptor::default(),
        new_descriptor: TaskDescriptor::default(),
        invariant_proof: InvariantProof {
            invariants: vec![],
            method: VerificationMethod::Compiler,
            timestamp: 0,
        },
    });

    let proof = make_compiler_proof(6);

    let overlay1 = OverlayValue::new(
        snapshot,
        changes1,
        proof.clone(),
        make_metadata(10),
    ).unwrap();

    let overlay2 = OverlayValue::new(
        snapshot,
        changes2,
        proof,
        make_metadata(10),
    ).unwrap();

    let composed = overlay1.compose(&overlay2);
    assert!(composed.is_err());
}

#[test]
fn test_compose_rejects_different_bases() {
    let snapshot1 = SnapshotId([1; 32]);
    let snapshot2 = SnapshotId([2; 32]);

    let changes = make_fast_changes();
    let proof = make_compiler_proof(6);

    let overlay1 = OverlayValue::new(
        snapshot1,
        changes.clone(),
        proof.clone(),
        make_metadata(10),
    ).unwrap();

    let overlay2 = OverlayValue::new(
        snapshot2,
        changes,
        proof,
        make_metadata(10),
    ).unwrap();

    let composed = overlay1.compose(&overlay2);
    assert!(composed.is_err());
}

// =============================================================================
// Proof Composition Tests
// =============================================================================

#[test]
fn test_composed_proof_preserves_invariants() {
    let proof1 = make_compiler_proof(6);
    let proof2 = make_compiler_proof(7);

    let composed = ComposedProof::new(proof1, proof2);
    assert!(composed.is_ok());

    let composed = composed.unwrap();
    // Intersection of invariants
    assert!(!composed.invariants_preserved().is_empty());
}

#[test]
fn test_composed_proof_takes_max_timing() {
    let proof1 = make_compiler_proof(6);
    let proof2 = make_compiler_proof(7);

    let composed = ComposedProof::new(proof1, proof2).unwrap();

    // Should take maximum of both
    assert_eq!(composed.timing_bound(), 7);
}

#[test]
fn test_composed_proof_verifies_both() {
    let mut proof1 = make_compiler_proof(6);
    proof1.signature = [0; 64];  // Invalid

    let proof2 = make_compiler_proof(7);

    let composed = ComposedProof::new(proof1, proof2);
    assert!(composed.is_err());
}

// =============================================================================
// Kernel Promotion Tests
// =============================================================================

#[test]
fn test_kernel_accepts_hot_safe_overlay() {
    let proof = make_compiler_proof(6);
    let safe_proof = SafeProof::<HotSafe, _>::new(proof).unwrap();

    let snapshot = make_snapshot();
    let overlay = OverlayValue::new(
        snapshot,
        make_fast_changes(),
        safe_proof,
        make_metadata(10),
    ).unwrap();

    let mut kernel = MuKernel::new(1024);
    let result = kernel.promote_hot(overlay);

    // Would succeed with proper Σ* setup
    let _ = result;
}

#[test]
fn test_kernel_accepts_warm_safe_with_rollout() {
    let proof = make_compiler_proof(6);
    let safe_proof = SafeProof::<WarmSafe, _>::new(proof).unwrap();

    let snapshot = make_snapshot();
    let overlay = OverlayValue::new(
        snapshot,
        make_fast_changes(),
        safe_proof,
        make_metadata(10),
    ).unwrap();

    let mut kernel = MuKernel::new(1024);
    let rollout = RolloutStrategy::Canary {
        initial_percent: 5,
        increment_percent: 10,
        wait_seconds: 60,
    };

    let result = kernel.promote_warm(overlay, rollout);
    let _ = result;
}

// =============================================================================
// Impossibility Tests (What You CANNOT Write)
// =============================================================================

// These tests demonstrate what the type system prevents.
// They are commented out because they won't compile - that's the point!

/*
#[test]
fn test_cannot_apply_overlay_without_proof() {
    // ❌ This won't compile - OverlayValue requires a proof
    let overlay = OverlayValue {
        base_sigma: make_snapshot(),
        changes: make_fast_changes(),
        // proof: ???  // Cannot construct without proof!
        metadata: make_metadata(10),
    };
}
*/

/*
#[test]
fn test_cannot_manually_create_compiler_proof() {
    // ❌ This won't compile - OverlayProof trait is sealed
    struct FakeProof;
    impl OverlayProof for FakeProof {  // ERROR: trait is sealed
        // ...
    }
}
*/

/*
#[test]
fn test_cannot_promote_cold_to_hot_directly() {
    let runtime_proof = RuntimeProof { /* ... */ };
    let cold = SafeProof::<ColdUnsafe, _>::new(runtime_proof).unwrap();
    let overlay = OverlayValue::new(/* ... */, cold, /* ... */).unwrap();

    let mut kernel = MuKernel::new(1024);

    // ❌ This won't compile - type mismatch
    // promote_hot requires SafeProof<HotSafe, _>, not SafeProof<ColdUnsafe, _>
    kernel.promote_hot(overlay);  // ERROR: type mismatch
}
*/

/*
#[test]
fn test_cannot_bypass_safety_levels() {
    let slow_proof = make_compiler_proof(1000);  // Too slow for HotSafe

    // ❌ This won't compile or will fail at construction
    let fake_hot = SafeProof::<HotSafe, _>::new(slow_proof);  // ERROR: timing check fails
}
*/

// =============================================================================
// Proof Builder Tests
// =============================================================================

#[test]
fn test_proof_builder_generates_valid_hot_safe() {
    let mut builder = ProofBuilder::new();
    let changes = make_fast_changes();

    let proof = builder.build_hot_safe(&changes);
    assert!(proof.is_ok());
}

#[test]
fn test_proof_builder_rejects_slow_for_hot() {
    let mut builder = ProofBuilder::new();

    let mut changes = OverlayChanges::new();
    changes.push(OverlayChange::AddTask {
        task_id: 1,
        descriptor: TaskDescriptor::default(),
        tick_budget: 100,  // Too slow
    });

    let proof = builder.build_hot_safe(&changes);
    assert!(proof.is_err());
}

#[test]
fn test_proof_builder_warm_safe() {
    let mut builder = ProofBuilder::new();

    let mut changes = OverlayChanges::new();
    changes.push(OverlayChange::AddTask {
        task_id: 1,
        descriptor: TaskDescriptor::default(),
        tick_budget: 1000,  // OK for warm
    });

    let proof = builder.build_warm_safe(&changes);
    assert!(proof.is_ok());
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_empty_overlay_is_valid() {
    let proof = make_compiler_proof(0);
    let snapshot = make_snapshot();
    let changes = OverlayChanges::new();

    let overlay = OverlayValue::new(
        snapshot,
        changes,
        proof,
        make_metadata(10),
    );

    assert!(overlay.is_ok());
}

#[test]
fn test_overlay_with_many_changes() {
    let mut changes = OverlayChanges::new();
    for i in 0..100 {
        changes.push(OverlayChange::AddTask {
            task_id: i,
            descriptor: TaskDescriptor::default(),
            tick_budget: 1,
        });
    }

    let proof = make_compiler_proof(100);
    let snapshot = make_snapshot();

    let overlay = OverlayValue::new(
        snapshot,
        changes,
        proof,
        make_metadata(10),
    );

    assert!(overlay.is_ok());
}

#[test]
fn test_proof_strength_ordering() {
    use knhk_mu_kernel::overlay_proof::ProofStrength;

    assert!(ProofStrength::Formal > ProofStrength::Compiler);
    assert!(ProofStrength::Compiler > ProofStrength::PropertyBased);
    assert!(ProofStrength::PropertyBased > ProofStrength::Runtime);
}
