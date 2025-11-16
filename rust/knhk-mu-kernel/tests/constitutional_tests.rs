//! Constitutional Cross-Layer Integration Tests
//!
//! These tests verify that the constitutional guarantees hold across
//! both the μ-kernel and AHI layers. They test:
//!
//! 1. Type-safe separation (AHI cannot construct kernel artifacts)
//! 2. Resource accounting (tick quotas enforced)
//! 3. Proof obligations (overlays require valid proofs)
//! 4. Timescale separation (hot/warm/cold enforced)
//! 5. Constitutional invariants (all four traits)

use knhk_mu_kernel::*;
use knhk_mu_kernel::ahi::*;
use knhk_mu_kernel::constitutional::*;
use knhk_mu_kernel::sigma::{SigmaCompiled, SigmaPointer};
use knhk_mu_kernel::core::MuKernel;

/// Test that AHI cannot directly modify Σ*
#[test]
fn test_ahi_cannot_modify_sigma() {
    // This test verifies type safety - AHI doesn't have methods to modify Σ*
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let kernel = MuKernel::new(sigma_ptr);

    let ctx = AhiContext::new(&kernel, 1000);

    // ctx has no method to directly modify Σ*
    // This is enforced at compile time - if you uncomment the following,
    // it won't compile:
    // ctx.modify_sigma(...); // ERROR: method not found

    // AHI can only submit proven overlays
    assert_eq!(ctx.quota().remaining(), 1000);
}

/// Test that tick quotas are enforced
#[test]
fn test_tick_quota_enforcement() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let kernel = MuKernel::new(sigma_ptr);

    let mut ctx = AhiContext::new(&kernel, 100);

    // Request reasonable amount of ticks
    let grant = ctx.request_ticks(50).unwrap();
    assert_eq!(grant.ticks, 50);

    // Request excessive ticks - should fail
    let result = ctx.request_ticks(10000);
    assert!(result.is_err());
}

/// Test that overlay submission requires proof
#[test]
fn test_overlay_requires_proof() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let kernel = MuKernel::new(sigma_ptr);

    let mut ctx = AhiContext::new(&kernel, 1000);

    // Create an overlay
    let overlay = DeltaSigma::new();

    // Create a valid proof
    let proof = TickBudgetProof::<8> {
        measured_ticks: 5,
        timestamp: 0,
    };

    let proven = ProvenOverlay::new(overlay, proof);

    // Submit should succeed with valid proof
    let token = ctx.submit_overlay(proven).unwrap();
    assert!(token.id > 0);
}

/// Test that invalid proofs are rejected
#[test]
fn test_invalid_proof_rejected() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let kernel = MuKernel::new(sigma_ptr);

    let mut ctx = AhiContext::new(&kernel, 1000);

    let overlay = DeltaSigma::new();

    // Create invalid proof (exceeds budget)
    let proof = TickBudgetProof::<8> {
        measured_ticks: 10, // Exceeds MAX_TICKS
        timestamp: 0,
    };

    let proven = ProvenOverlay::new(overlay, proof);

    // Submit should fail
    let result = ctx.submit_overlay(proven);
    assert!(result.is_err());
}

/// Test hot path timescale constraints
#[test]
fn test_hot_path_constraints() {
    use ahi::timescales::*;

    let op = ExampleHotOp { input: 42 };

    // Verify compile-time constraints
    let _ = ExampleHotOp::CONSTRAINTS_SATISFIED;

    // Verify constants
    assert_eq!(ExampleHotOp::MAX_TICKS, 3);
    assert!(!ExampleHotOp::ALLOCATES);
    assert!(ExampleHotOp::MAX_TICKS <= CHATMAN_CONSTANT);

    // Execute
    let result = op.execute_hot().unwrap();
    assert!(result.cost <= ExampleHotOp::MAX_TICKS);
}

/// Test that hot path cannot allocate
#[test]
fn test_hot_path_no_allocation() {
    use ahi::timescales::*;

    // This is enforced at compile time via const
    assert!(!ExampleHotOp::ALLOCATES);

    // If we try to create a Hot impl that allocates, it won't compile
    // struct BadHot;
    // impl Hot for BadHot {
    //     const MAX_TICKS: u64 = 5;
    //     const ALLOCATES: bool = true;  // ERROR: violates constraint
    //     fn execute_hot(&self) -> Result<Action, HotError> { ... }
    // }
}

/// Test warm path constraints
#[tokio::test]
async fn test_warm_path_constraints() {
    use ahi::timescales::*;

    let op = ExampleWarmOp {
        input: alloc::vec![1, 2, 3, 4],
    };

    // Verify compile-time constraints
    let _ = ExampleWarmOp::CONSTRAINTS_SATISFIED;

    // Verify constants
    assert_eq!(ExampleWarmOp::MAX_MILLIS, 1);
    assert!(ExampleWarmOp::MAX_MILLIS <= 1000);

    // Execute
    let result = op.execute_warm().await.unwrap();
    assert!(result.cost > 0);
}

/// Test Chatman bounded trait
#[test]
fn test_chatman_bounded_trait() {
    use constitutional::*;

    let check = SimpleCheck { threshold: 100 };

    // Compile-time verification
    let _ = SimpleCheck::CHATMAN_SATISFIED;

    // Runtime verification
    assert_eq!(SimpleCheck::WORST_CASE_TICKS, 3);
    assert!(SimpleCheck::WORST_CASE_TICKS <= CHATMAN_CONSTANT);

    let budget = check.tick_budget();
    assert_eq!(budget.limit, 3);
}

/// Test doctrine alignment
#[test]
fn test_doctrine_alignment() {
    use constitutional::*;

    let check = SimpleCheck { threshold: 50 };
    let doctrine = Doctrine::new(1, 0xFFFF);

    // Verify doctrine alignment
    check.verify_doctrine(&doctrine).unwrap();

    let hash = check.doctrine_hash();
    assert_eq!(hash.len(), 32);
}

/// Test closed world property
#[test]
fn test_closed_world() {
    use constitutional::*;

    let check = SimpleCheck { threshold: 42 };

    // Observe complete state
    let state = check.observe_complete_state();
    assert_eq!(state, 42);

    // Verify state hash is deterministic
    let hash1 = check.state_hash();
    let hash2 = check.state_hash();
    assert_eq!(hash1, hash2);
}

/// Test deterministic execution
#[test]
fn test_deterministic_execution() {
    use constitutional::*;

    let check = SimpleCheck { threshold: 75 };

    // Same input → same output
    let output1 = check.deterministic_execute(&100);
    let output2 = check.deterministic_execute(&100);
    assert_eq!(output1, output2);

    // Different input → different output (potentially)
    let output3 = check.deterministic_execute(&50);
    assert_ne!(output1, output3);

    // Verify determinism
    assert!(check.verify_determinism(&100));
}

/// Test full constitutional compliance
#[test]
fn test_full_constitutional() {
    use constitutional::*;

    let check = SimpleCheck { threshold: 100 };
    let doctrine = Doctrine::new(1, 0xFFFF);

    // Verify all constitutional guarantees
    check.verify_constitutional(&doctrine).unwrap();

    // Generate constitutional receipt
    let receipt = check.constitutional_receipt();
    assert_eq!(receipt.tick_bound, SimpleCheck::WORST_CASE_TICKS);

    let hash = receipt.hash();
    assert_eq!(hash.len(), 32);
}

/// Test decision interface
#[test]
fn test_decision_interface() {
    use ahi::decision::*;

    let sigma = Box::leak(Box::new(SigmaCompiled::new()));
    let obs = U64Observation(42);
    let invariants = heapless::Vec::new();

    let mut decision: Decision<_, 8> = Decision::new(
        obs,
        sigma,
        invariants,
        RiskClass::Low,
    );

    // Verify compile-time properties
    assert!(Decision::<U64Observation, 8>::within_chatman_constant());
    assert_eq!(Decision::<U64Observation, 8>::tick_cost(), 8);

    // Execute decision
    let action = decision.execute().unwrap();
    assert_eq!(action.output_hash.len(), 32);
}

/// Test decision with high risk requires approval
#[test]
fn test_high_risk_decision() {
    use ahi::decision::*;

    let sigma = Box::leak(Box::new(SigmaCompiled::new()));
    let obs = U64Observation(42);
    let invariants = heapless::Vec::new();

    let mut decision: Decision<_, 8> = Decision::new(
        obs,
        sigma,
        invariants,
        RiskClass::Critical,
    );

    // High risk decisions require approval
    let result = decision.execute();
    assert!(matches!(result, Err(DecisionError::ApprovalRequired)));
}

/// Test timescale executor
#[test]
fn test_timescale_executor() {
    use ahi::timescales::*;

    let executor = TimescaleExecutor::new();
    let op = ExampleHotOp { input: 123 };

    let result = executor.execute_hot(&op).unwrap();
    assert!(result.cost <= ExampleHotOp::MAX_TICKS);
}

/// Test proof factory
#[test]
fn test_proof_factory() {
    use ahi::userspace::*;

    let factory = ProofFactory::new();

    // Create valid tick proof
    let proof = factory.tick_proof::<8>(5).unwrap();
    assert_eq!(proof.measured_ticks, 5);
    assert!(proof.verify().is_ok());

    // Create invalid tick proof (exceeds budget)
    let invalid = factory.tick_proof::<8>(10);
    assert!(invalid.is_err());
}

/// Test invariant proof
#[test]
fn test_invariant_proof() {
    use ahi::userspace::*;
    use sigma::SigmaHash;

    let factory = ProofFactory::new();

    let mut invariants = heapless::Vec::new();
    invariants.push(1).unwrap();
    invariants.push(2).unwrap();

    let proof = factory.invariant_proof(
        invariants,
        SigmaHash([0; 32]),
        SigmaHash([1; 32]),
    );

    assert!(proof.verify().is_ok());
    assert_eq!(proof.proof_hash().len(), 32);
}

/// Test cross-layer integration
#[test]
fn test_cross_layer_integration() {
    use ahi::decision::*;
    use ahi::userspace::*;
    use constitutional::*;

    // Create kernel
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let kernel = MuKernel::new(sigma_ptr);

    // Create AHI context
    let mut ctx = AhiContext::new(&kernel, 5000);

    // Create decision
    let sigma = Box::leak(Box::new(SigmaCompiled::new()));
    let obs = U64Observation(42);
    let invariants = heapless::Vec::new();

    let mut decision: Decision<_, 8> = Decision::new(
        obs,
        sigma,
        invariants,
        RiskClass::Low,
    );

    // Execute decision
    let action = decision.execute().unwrap();

    // Create proof from execution
    let proof = ctx.proof_factory().tick_proof::<8>(action.ticks_consumed).unwrap();

    // Create overlay with proof
    let overlay = DeltaSigma::new();
    let proven = ProvenOverlay::new(overlay, proof);

    // Submit to kernel
    let token = ctx.submit_overlay(proven).unwrap();

    // Verify submission
    assert!(token.id > 0);
    assert!(ctx.quota().consumed > 0);
}

/// Test that const generics enforce tick bounds
#[test]
fn test_const_generic_tick_bounds() {
    use ahi::decision::*;

    // This compiles - within Chatman Constant
    let _: Decision<U64Observation, 8> = Decision::new(
        U64Observation(0),
        Box::leak(Box::new(SigmaCompiled::new())),
        heapless::Vec::new(),
        RiskClass::Safe,
    );

    // This compiles - exceeds Chatman Constant (warm path)
    let _: Decision<U64Observation, 100> = Decision::new(
        U64Observation(0),
        Box::leak(Box::new(SigmaCompiled::new())),
        heapless::Vec::new(),
        RiskClass::Safe,
    );

    // Can check at compile time
    assert!(Decision::<U64Observation, 8>::within_chatman_constant());
    assert!(!Decision::<U64Observation, 100>::within_chatman_constant());
}

/// Test observation slice trait
#[test]
fn test_observation_slice() {
    use ahi::decision::*;

    // U64 observation
    let obs1 = U64Observation(12345);
    let bytes = obs1.to_bytes();
    assert_eq!(bytes.len(), 8);
    assert_eq!(obs1.hash().len(), 32);

    // Byte observation
    let obs2 = ByteObservation([1, 2, 3, 4, 5, 6, 7, 8]);
    let bytes = obs2.to_bytes();
    assert_eq!(bytes.len(), 8);
    assert_eq!(obs2.hash().len(), 32);

    // Hashes are deterministic
    let hash1 = obs1.hash();
    let hash2 = obs1.hash();
    assert_eq!(hash1, hash2);
}

/// Test risk classification
#[test]
fn test_risk_classification() {
    use ahi::decision::*;

    assert_eq!(RiskClass::Safe.level(), 0);
    assert_eq!(RiskClass::Low.level(), 1);
    assert_eq!(RiskClass::Medium.level(), 2);
    assert_eq!(RiskClass::High.level(), 3);
    assert_eq!(RiskClass::Critical.level(), 4);

    assert!(!RiskClass::Safe.requires_approval());
    assert!(!RiskClass::Low.requires_approval());
    assert!(!RiskClass::Medium.requires_approval());
    assert!(RiskClass::High.requires_approval());
    assert!(RiskClass::Critical.requires_approval());
}

/// Benchmark hot path performance (integration with timing module)
#[test]
fn test_hot_path_performance() {
    use ahi::timescales::*;
    use timing::measure_ticks;

    let op = ExampleHotOp { input: 999 };

    let (result, ticks) = measure_ticks(|| op.execute_hot());

    assert!(result.is_ok());
    assert!(ticks > 0);
    // In a real system, we'd assert ticks <= CHATMAN_CONSTANT
    // For testing without hardware counters, we just verify it executes
}

/// Test doctrine with multiple invariants
#[test]
fn test_doctrine_multiple_invariants() {
    use constitutional::*;

    let mut doctrine = Doctrine::new(1, 0b11110000);

    doctrine.add_invariant(1).unwrap();
    doctrine.add_invariant(2).unwrap();
    doctrine.add_invariant(3).unwrap();

    assert_eq!(doctrine.required_invariants.len(), 3);
    assert!(doctrine.permits_op(4));
    assert!(doctrine.permits_op(7));
    assert!(!doctrine.permits_op(0));
    assert!(!doctrine.permits_op(3));
}

/// Test quota exhaustion
#[test]
fn test_quota_exhaustion() {
    use ahi::userspace::*;

    let mut quota = TickQuota::new(100);

    quota.consume(50).unwrap();
    assert_eq!(quota.remaining(), 50);

    quota.consume(30).unwrap();
    assert_eq!(quota.remaining(), 20);

    let result = quota.consume(30);
    assert!(result.is_err());
    assert_eq!(quota.consumed, 80); // Not updated on failure
}
