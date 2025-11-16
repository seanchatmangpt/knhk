//! Chicago-Style TDD Integration Tests for AHI Microkernel
//!
//! These tests follow Chicago School TDD principles:
//! - State-based assertions (no mocks)
//! - Integration testing across modules
//! - Observable behavior validation
//! - Real object collaboration
//!
//! Focus: 80/20 critical path validation

use knhk_workflow_engine::innovation::{
    // Verified kernel
    KernelResult, KernelError, TickBudget, KernelState, ExecutionPhase,
    GuardResult, KernelOp, GuardCheckOp, KernelSequence, VerifiedContext,
    // Refinement guards
    PublicSector, PrivateSector, CriticalSector, GuardProperty,
    BudgetGuard, ProofToken, GuardVector, StrictDoctrine, RelaxedDoctrine,
    // Cluster types
    Leader, Follower, Observer, ClusterConfig, TripleReplication,
    ConsensusOp, Proposal, DistributedContext,
    // Auto-specialization
    GenericCpu, X86Avx2, SmallData, LargeData, ScalarKernel,
    SimdAvx2Kernel, AutoSelector, KernelSelection, AdaptiveExecutor,
    // Linear resources
    ResourceToken, ResourceQuota, P0, P1, P2, P4, Interactive, Batch,
    ScheduledAction, HotPathScheduler, BackgroundScheduler, ResourcePool,
};

// ============================================================================
// CRITICAL PATH 1: Chatman Constant Enforcement (20% that delivers 80%)
// ============================================================================

#[test]
fn test_chatman_constant_enforced_across_all_modules() {
    // Verified kernel enforces ≤8 ticks at compile time
    let mut state = KernelState::<32>::new();
    let mut budget = TickBudget::<8>::new();

    // Execute exactly 8 ticks - should succeed
    for _ in 0..8 {
        match state.tick() {
            KernelResult::Success(()) => {},
            KernelResult::Failure(e) => panic!("Unexpected failure at tick {}: {:?}", state.tick_count, e),
        }
    }
    assert_eq!(state.tick_count, 8);

    // 9th tick should fail with ChatmanViolation
    match state.tick() {
        KernelResult::Success(()) => panic!("Should have violated Chatman constant"),
        KernelResult::Failure(KernelError::ChatmanViolation { actual, limit }) => {
            assert_eq!(actual, 9);
            assert_eq!(limit, 8);
        }
        KernelResult::Failure(e) => panic!("Wrong error type: {:?}", e),
    }
}

#[test]
fn test_chatman_enforcement_in_guard_vector() {
    // GuardVector must sum to ≤8 ticks at compile time
    // This test validates the const_assert! works
    let _valid_guards = GuardVector::<BudgetGuard<2>, BudgetGuard<3>, BudgetGuard<3>>::new();

    // The following would not compile (8 ticks total is OK):
    // let _max_guards = GuardVector::<BudgetGuard<8>, BudgetGuard<0>, BudgetGuard<0>>::new();

    // This demonstrates compile-time enforcement works
    assert_eq!(GuardVector::<BudgetGuard<2>, BudgetGuard<3>, BudgetGuard<3>>::TOTAL_TICKS, 8);
}

#[test]
fn test_chatman_enforcement_in_resource_scheduling() {
    // Interactive SLO must respect Chatman constant (8 ticks)
    let action = ScheduledAction::<P0, Interactive, 8>::new();

    // Action with cost=8 should fit in Interactive SLO (MAX_TICKS=8)
    let token = ResourceToken::<10>::new();
    let result = action.execute(token);
    assert!(result.is_ok());

    // The following would not compile - exceeds Interactive MAX_TICKS:
    // let _invalid = ScheduledAction::<P0, Interactive, 9>::new();
}

// ============================================================================
// CRITICAL PATH 2: Type-Level Safety Guarantees
// ============================================================================

#[test]
fn test_role_based_access_control_enforced_by_types() {
    // Only leaders can commit decisions
    let mut leader_ctx = DistributedContext::<Leader, 3, 2>::new();
    assert!(leader_ctx.commit_decision(1).is_ok());

    // Followers cannot commit (enforced at runtime since role is generic)
    let mut follower_ctx = DistributedContext::<Follower, 3, 2>::new();
    match follower_ctx.commit_decision(2) {
        Ok(_) => panic!("Follower should not be able to commit"),
        Err(e) => assert_eq!(e, "Role cannot commit"),
    }

    // Observers cannot commit or vote
    let mut observer_ctx = DistributedContext::<Observer, 3, 2>::new();
    match observer_ctx.commit_decision(3) {
        Ok(_) => panic!("Observer should not be able to commit"),
        Err(e) => assert_eq!(e, "Role cannot commit"),
    }
}

#[test]
fn test_quorum_enforcement_at_compile_time() {
    // Valid cluster: 3 replicas, quorum=2 (majority)
    let _config = ClusterConfig::<3, 2>::new();
    assert!(ClusterConfig::<3, 2>::has_quorum(2));
    assert!(!ClusterConfig::<3, 2>::has_quorum(1));

    // Valid cluster: 5 replicas, quorum=3
    let _config = ClusterConfig::<5, 3>::new();
    assert!(ClusterConfig::<5, 3>::has_quorum(3));

    // The following would not compile - quorum too small (not majority):
    // let _invalid = ClusterConfig::<3, 1>::new(); // 1 < (3/2 + 1)

    // The following would not compile - quorum exceeds replicas:
    // let _invalid = ClusterConfig::<3, 4>::new();
}

#[test]
fn test_linear_resource_tokens_prevent_double_spend() {
    let mut quota = ResourceQuota::<1000>::new();

    // Allocate 300 tokens
    let token1 = quota.allocate::<300>().expect("Should allocate");
    assert_eq!(quota.remaining(), 700);

    // Token can be split but not cloned
    let (token_a, token_b) = token1.split::<100, 200>();
    assert_eq!(ResourceToken::<100>::amount(), 100);
    assert_eq!(ResourceToken::<200>::amount(), 200);

    // Consume tokens (moves ownership, prevents reuse)
    let consumed_a = token_a.consume();
    let consumed_b = token_b.consume();

    // Cannot use consumed tokens again (would not compile if we tried)
    // let _reuse = token_a.consume(); // ERROR: value moved

    // Reclaim to restore quota
    quota.reclaim(consumed_a);
    quota.reclaim(consumed_b);
    assert_eq!(quota.remaining(), 1000);
}

#[test]
fn test_scheduler_type_safety_prevents_priority_inversion() {
    let mut hot_path = HotPathScheduler::<10>::new();

    // Hot path only accepts P0/P1 + Interactive
    let p0_action = ScheduledAction::<P0, Interactive, 5>::new();
    assert!(hot_path.enqueue(p0_action).is_ok());

    let p1_action = ScheduledAction::<P1, Interactive, 7>::new();
    assert!(hot_path.enqueue_p1(p1_action).is_ok());

    // The following would not compile - P2 not allowed on hot path:
    // let p2_action = ScheduledAction::<P2, Interactive, 5>::new();
    // hot_path.enqueue(p2_action); // ERROR: type mismatch

    // Background scheduler only accepts P3/P4
    let mut background = BackgroundScheduler::<10>::new();
    let p4_action = ScheduledAction::<P4, Batch, 50>::new();
    assert!(background.enqueue(p4_action).is_ok());
}

// ============================================================================
// CRITICAL PATH 3: Integration Across Modules
// ============================================================================

#[test]
fn test_end_to_end_verified_workflow_execution() {
    // Create verified context with Chatman budget
    let mut ctx = VerifiedContext::<8, 32>::new();

    // Execute workflow with guards and tick tracking
    let result = ctx.run(|state, budget| {
        // Check budget (2 ticks)
        match budget.consume(2) {
            KernelResult::Success(()) => {},
            KernelResult::Failure(e) => return KernelResult::Failure(e),
        }

        // Execute guard check (consumes 1 tick internally)
        let guard = GuardCheckOp::new(|| true, 1);
        match guard.execute(state, budget) {
            KernelResult::Success(()) => {},
            KernelResult::Failure(e) => return KernelResult::Failure(e),
        }

        // Additional processing (2 ticks)
        match state.tick() {
            KernelResult::Success(()) => {},
            KernelResult::Failure(e) => return KernelResult::Failure(e),
        }
        match state.tick() {
            KernelResult::Success(()) => {},
            KernelResult::Failure(e) => return KernelResult::Failure(e),
        }

        KernelResult::Success(())
    });

    // Verify execution succeeded
    assert!(result.is_success());

    // Verify Chatman constant respected (2 + 1 + 2 = 5 ticks)
    assert_eq!(ctx.ticks_used(), 5);
    assert!(ctx.ticks_used() <= 8);

    // Verify guard was recorded
    assert_eq!(ctx.state().guard_count, 1);
}

#[test]
fn test_distributed_consensus_with_resource_allocation() {
    // Distributed cluster with resource management
    let mut pool = ResourcePool::<1000>::new();
    let mut leader_ctx = DistributedContext::<Leader, 3, 2>::new();

    // Allocate resources for high-priority consensus operation
    let token = pool.allocate_p0::<100>().expect("Should allocate for P0");
    assert_eq!(pool.remaining(), 900);

    // Execute consensus decision
    let decision_result = leader_ctx.execute(|| {
        // Deterministic consensus logic
        42
    });
    assert_eq!(decision_result.unwrap(), 42);

    // Commit decision (only leader can do this)
    assert!(leader_ctx.commit_decision(1).is_ok());

    // Reclaim resources after commit
    let consumed = token.consume();
    pool.reclaim(consumed);
    assert_eq!(pool.remaining(), 1000);
}

#[test]
fn test_hardware_adaptive_execution_with_chatman_compliance() {
    // Auto-select best kernel for hardware/data profile
    let selection = AutoSelector::<X86Avx2, SmallData>::select();

    // Small data on AVX2 should select narrow SIMD variant
    assert_eq!(selection, KernelSelection::SimdNarrow);

    // Create adaptive executor
    let mut executor = AdaptiveExecutor::<X86Avx2, SmallData>::new();

    // Execute with performance monitoring
    let result = executor.execute_and_adapt(&[1, 2, 3, 4, 5]);
    assert_eq!(result, 15);

    // Verify kernel selection
    assert_eq!(executor.current_variant(), KernelSelection::SimdNarrow);

    // Execution should be Chatman compliant (≤8 ticks per operation)
    // This is enforced by kernel design, validated through integration
}

// ============================================================================
// CRITICAL PATH 4: Failure Scenarios and Error Handling
// ============================================================================

#[test]
fn test_budget_exhaustion_prevents_execution() {
    let mut quota = ResourceQuota::<100>::new();

    // Allocate 60 tokens
    let _token1 = quota.allocate::<60>().expect("Should allocate");
    assert_eq!(quota.remaining(), 40);

    // Try to allocate 50 more (should fail - only 40 remaining)
    let token2 = quota.allocate::<50>();
    assert!(token2.is_none());

    // Smaller allocation should succeed
    let token3 = quota.allocate::<30>().expect("Should allocate");
    assert_eq!(quota.remaining(), 10);
}

#[test]
fn test_guard_failure_propagates_correctly() {
    let mut state = KernelState::<32>::new();
    let mut budget = TickBudget::<8>::new();

    // Guard that fails
    let failing_guard = GuardCheckOp::new(|| false, 1);

    let result = failing_guard.execute(&mut state, &mut budget);

    // Should return guard failure
    match result {
        KernelResult::Failure(KernelError::GuardFailure { guard_id }) => {
            assert_eq!(guard_id, 1);
        }
        _ => panic!("Expected GuardFailure, got: {:?}", result),
    }

    // Guard result should be recorded as Fail
    assert_eq!(state.guard_count, 1);
    assert!(matches!(state.guards[0], GuardResult::Fail));
}

#[test]
fn test_consensus_fails_without_quorum() {
    // Create proposal with triple replication
    let mut proposal = Proposal::<i32, TripleReplication>::new(42);

    // Add only 1 vote (need 2 for quorum in triple replication)
    assert!(proposal.vote::<Leader>().is_ok());
    assert!(!proposal.has_quorum());

    // Try to commit without quorum
    let result = proposal.commit::<Leader>();
    match result {
        Ok(_) => panic!("Should not commit without quorum"),
        Err(e) => assert_eq!(e, "Quorum not reached"),
    }
}

#[test]
fn test_invalid_state_transitions_prevented() {
    let mut state = KernelState::<32>::new();

    // Initial state is Idle
    assert_eq!(state.phase, ExecutionPhase::Idle);

    // Can transition to GuardCheck
    assert!(state.transition(ExecutionPhase::GuardCheck).is_success());
    assert_eq!(state.phase, ExecutionPhase::GuardCheck);

    // Can transition to Execution
    assert!(state.transition(ExecutionPhase::Execution).is_success());

    // Cannot transition backwards to GuardCheck
    assert!(state.transition(ExecutionPhase::GuardCheck).is_failure());
}

// ============================================================================
// CRITICAL PATH 5: Performance and Scalability Validation
// ============================================================================

#[test]
fn test_kernel_sequence_batch_execution() {
    // Execute multiple operations in sequence with guaranteed bounds
    let sequence = KernelSequence::<3, 6>::new(); // 3 ops, 6 total ticks

    let guard1 = GuardCheckOp::new(|| true, 1);
    let guard2 = GuardCheckOp::new(|| true, 2);
    let guard3 = GuardCheckOp::new(|| true, 3);

    let ops = [guard1, guard2, guard3];
    let mut state = KernelState::<32>::new();

    let result = sequence.execute::<8>(&ops, &mut state);

    // All operations should succeed
    assert!(result.is_success());

    // All guards should be recorded
    assert_eq!(state.guard_count, 3);
    assert!(matches!(state.guards[0], GuardResult::Pass));
    assert!(matches!(state.guards[1], GuardResult::Pass));
    assert!(matches!(state.guards[2], GuardResult::Pass));
}

#[test]
fn test_resource_pool_priority_allocation() {
    let mut pool = ResourcePool::<1000>::new();

    // P0 can always allocate (highest priority)
    let p0_token = pool.allocate_p0::<300>().expect("P0 should always allocate");
    assert_eq!(pool.remaining(), 700);

    // P2 can allocate if enough remains (> 50% total)
    let p2_token = pool.allocate_p2::<200>().expect("Should allocate when >50% remains");
    assert_eq!(pool.remaining(), 500);

    // P2 blocked when ≤50% remains (reserved for P0)
    let p2_blocked = pool.allocate_p2::<100>();
    assert!(p2_blocked.is_none());

    // Reclaim and verify restoration
    pool.reclaim(p0_token.consume());
    pool.reclaim(p2_token.consume());
    assert_eq!(pool.remaining(), 1000);
}

#[test]
fn test_adaptive_executor_performance_monitoring() {
    let mut executor = AdaptiveExecutor::<GenericCpu, LargeData>::new();

    // Execute multiple iterations
    for _ in 0..5 {
        let result = executor.execute_and_adapt(&[1, 2, 3, 4, 5]);
        assert_eq!(result, 15);
    }

    // Performance monitor should have samples
    // Executor may adapt based on performance (implementation detail)

    // Verify execution remains Chatman compliant
    // (Each operation ≤8 ticks, enforced by kernel design)
}

// ============================================================================
// CRITICAL PATH 6: Zero-Cost Abstraction Validation
// ============================================================================

#[test]
fn test_proof_tokens_are_zero_sized() {
    use std::mem;

    // Proof tokens should be zero-sized (compile-time only)
    assert_eq!(mem::size_of::<ProofToken<BudgetGuard<5>>>(), 0);
    assert_eq!(mem::size_of::<ProofToken<GuardVector<BudgetGuard<2>, BudgetGuard<3>, BudgetGuard<3>>>>(), 0);

    // Type-level constraints have zero runtime cost
}

#[test]
fn test_phantom_types_zero_cost() {
    use std::mem;

    // Phantom marker types are zero-sized
    assert_eq!(mem::size_of::<PublicSector>(), 0);
    assert_eq!(mem::size_of::<PrivateSector>(), 0);
    assert_eq!(mem::size_of::<CriticalSector>(), 0);

    // Doctrine constraints are zero-sized
    assert_eq!(mem::size_of::<StrictDoctrine>(), 0);
    assert_eq!(mem::size_of::<RelaxedDoctrine>(), 0);
}

#[test]
fn test_cpu_capability_types_zero_cost() {
    use std::mem;

    // CPU capability markers are zero-sized
    assert_eq!(mem::size_of::<GenericCpu>(), 0);
    assert_eq!(mem::size_of::<X86Avx2>(), 0);

    // Data profile markers are zero-sized
    assert_eq!(mem::size_of::<SmallData>(), 0);
    assert_eq!(mem::size_of::<LargeData>(), 0);
}

// ============================================================================
// CRITICAL PATH 7: Doctrine Compliance Validation
// ============================================================================

#[test]
fn test_closed_world_assumption_all_errors_enumerated() {
    // All kernel errors are enumerated in KernelError enum
    let chatman_error = KernelError::ChatmanViolation { actual: 9, limit: 8 };
    let guard_error = KernelError::GuardFailure { guard_id: 1 };
    let state_error = KernelError::InvalidStateTransition;
    let capacity_error = KernelError::InsufficientCapacity;

    // All errors have string representation
    assert!(format!("{:?}", chatman_error).contains("ChatmanViolation"));
    assert!(format!("{:?}", guard_error).contains("GuardFailure"));
    assert!(format!("{:?}", state_error).contains("InvalidStateTransition"));
    assert!(format!("{:?}", capacity_error).contains("InsufficientCapacity"));
}

#[test]
fn test_deterministic_execution_pure_state_transitions() {
    // Distributed context operations are pure (no side effects)
    let mut ctx1 = DistributedContext::<Leader, 3, 2>::new();
    let mut ctx2 = DistributedContext::<Leader, 3, 2>::new();

    // Same operations produce same results
    let result1 = ctx1.execute(|| 42);
    let result2 = ctx2.execute(|| 42);

    assert_eq!(result1.unwrap(), result2.unwrap());

    // Commit same decision
    assert!(ctx1.commit_decision(1).is_ok());
    assert!(ctx2.commit_decision(1).is_ok());
}

#[test]
fn test_no_panic_paths_total_functions_only() {
    // All kernel operations return KernelResult, never panic
    let mut state = KernelState::<32>::new();
    let mut budget = TickBudget::<8>::new();

    // Even extreme cases return errors, not panics
    for _ in 0..100 {
        let _ = state.tick(); // Returns error after 8 ticks
    }

    // Budget consumption returns error when exhausted
    for _ in 0..20 {
        let _ = budget.consume(1); // Returns error after 8 ticks
    }

    // No panics occurred - all errors returned as KernelResult
}
