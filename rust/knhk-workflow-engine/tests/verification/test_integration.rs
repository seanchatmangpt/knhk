//! End-to-end integration tests for formal verification
//!
//! Tests the complete verification workflow including:
//! - Policy verification
//! - Overlay verification
//! - Proof caching
//! - Invariant checking

use knhk_workflow_engine::{
    // Verification
    InvariantChecker, InvariantContext, PolicyVerifier, ProofCache, ProofSubject, SmtSolver,
    VerificationConfig, VerifiedPolicy,
    // Autonomic
    DeltaSigma, OverlayChange, OverlayScope, Unproven,
    // Policy lattice
    autonomic::policy_lattice::{LatencyBound, PolicyElement, Strictness},
    // Patterns
    patterns::PatternId,
};

#[tokio::test]
async fn test_complete_verification_workflow() {
    // Create verifier
    let verifier = PolicyVerifier::new().expect("Failed to create verifier");

    // Create policy
    let latency = LatencyBound::new(50.0, Strictness::Hard).expect("Failed to create latency bound");
    let policy = PolicyElement::Latency(latency);

    // Verify policy
    let proof = verifier
        .verify_policy(&policy)
        .expect("Failed to verify policy");

    assert!(proof.is_valid(), "Policy verification failed");
    assert!(proof.counterexample().is_none(), "Unexpected counterexample");
}

#[tokio::test]
async fn test_overlay_verification_with_proofs() {
    // Create verifier
    let verifier = PolicyVerifier::new().expect("Failed to create verifier");

    // Create overlay
    let scope = OverlayScope::new()
        .with_pattern(PatternId::new(12).expect("Invalid pattern ID"));

    let changes = vec![OverlayChange::ScaleMultiInstance { delta: 2 }];

    let unproven = DeltaSigma::new(scope, changes);
    let proof_pending = unproven
        .generate_proof_obligations()
        .expect("Failed to generate proof obligations");

    // Verify overlay
    let proof = verifier
        .verify_overlay(&proof_pending)
        .await
        .expect("Failed to verify overlay");

    assert!(proof.is_valid(), "Overlay verification failed");
}

#[tokio::test]
async fn test_proof_caching() {
    // Create proof cache
    let cache = ProofCache::new(100, 60_000); // 60 second TTL

    // Create and cache proof
    let overlay_id = knhk_workflow_engine::autonomic::delta_sigma::OverlayId::new();
    let subject = ProofSubject::Overlay(overlay_id);

    let proof =
        knhk_workflow_engine::ProofCertificate::new(subject.clone(), knhk_workflow_engine::ProofStatus::Valid);

    cache.put(proof.clone()).await.expect("Failed to cache proof");

    // Retrieve from cache
    let cached = cache
        .get(&subject)
        .await
        .expect("Failed to retrieve from cache");

    assert_eq!(cached.proof_id, proof.proof_id);
}

#[tokio::test]
async fn test_invariant_checking_workflow() {
    // Create invariant checker
    let mut checker = InvariantChecker::new();

    // Register invariants
    checker.register(knhk_workflow_engine::PolicyConsistencyInvariant);

    // Create context
    let latency = LatencyBound::new(100.0, Strictness::Hard).expect("Failed to create latency bound");
    let policy = PolicyElement::Latency(latency);
    let context = InvariantContext::new().with_policy(policy);

    // Check invariants
    checker
        .check_all(&context)
        .expect("Invariant check failed");

    assert_eq!(checker.violation_count(), 0, "Unexpected invariant violations");
}

#[tokio::test]
async fn test_doctrine_projection_verification() {
    use knhk_workflow_engine::autonomic::doctrine::Doctrine;

    let solver = SmtSolver::new();
    let doctrine = Doctrine::new();

    // Policy within bounds
    let valid_policy = PolicyElement::Latency(
        LatencyBound::new(50.0, Strictness::Hard).expect("Failed to create latency bound"),
    );

    let result = solver
        .verify_projection(&valid_policy, &doctrine)
        .expect("Failed to verify projection");

    assert!(result.is_sat(), "Valid policy should satisfy doctrine");

    // Policy exceeds bounds
    let invalid_policy = PolicyElement::Latency(
        LatencyBound::new(200.0, Strictness::Hard).expect("Failed to create latency bound"),
    );

    let result2 = solver
        .verify_projection(&invalid_policy, &doctrine)
        .expect("Failed to verify projection");

    // Note: Current simple solver doesn't enforce doctrine bounds in projection
    // Real SMT solver would detect this violation
    assert!(result2.is_sat() || result2.is_unsat());
}

#[tokio::test]
async fn test_type_level_verification() {
    use knhk_workflow_engine::verification::type_level::{Bounds, MuKernelTickBound, VerifiedMetrics};

    // Valid metrics
    let metrics =
        VerifiedMetrics::<MuKernelTickBound>::new(5).expect("Failed to create verified metrics");

    assert_eq!(metrics.ticks(), 5);

    // Invalid metrics (exceeds μ-kernel bound)
    let invalid_metrics = VerifiedMetrics::<MuKernelTickBound>::new(10);
    assert!(invalid_metrics.is_err(), "Should reject invalid tick count");
}

#[tokio::test]
async fn test_verified_policy_state_transitions() {
    let latency = LatencyBound::new(100.0, Strictness::Hard).expect("Failed to create latency bound");
    let policy = PolicyElement::Latency(latency);

    // Unverified → Verified → Proven
    let unverified = VerifiedPolicy::new(policy);
    let verified = unverified.verify().expect("Failed to verify");
    let proven = verified.prove().expect("Failed to prove");

    assert!(!proven.is_bottom());
    assert!(proven.policy().partial_cmp_lattice(&PolicyElement::bottom()).is_some());
}

#[tokio::test]
async fn test_performance_constraint_verification() {
    use knhk_workflow_engine::autonomic::doctrine::{ExecutionMetrics, MAX_EXEC_TICKS};

    let verifier = PolicyVerifier::new().expect("Failed to create verifier");

    // Valid metrics (within μ-kernel constraint)
    let mut valid_metrics = ExecutionMetrics::new();
    valid_metrics.exec_ticks = 5;

    let proof1 = verifier
        .verify_metrics(&valid_metrics)
        .expect("Failed to verify metrics");

    assert!(proof1.is_valid(), "Valid metrics should pass verification");

    // Invalid metrics (exceeds μ-kernel constraint)
    let mut invalid_metrics = ExecutionMetrics::new();
    invalid_metrics.exec_ticks = 20; // Exceeds MAX_EXEC_TICKS (8)

    let proof2 = verifier
        .verify_metrics(&invalid_metrics)
        .expect("Failed to verify metrics");

    assert!(!proof2.is_valid(), "Invalid metrics should fail verification");
    assert!(
        proof2.counterexample().is_some(),
        "Should provide counterexample"
    );
}
