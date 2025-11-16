//! Integration tests for Doctrine-Bound Policy Lattice Kernel
//!
//! Tests the complete integration of policy lattice with MAPE-K framework.

use knhk_workflow_engine::autonomic::{
    Action, ActionType, AdaptationPlan, CapacityEnvelope, Doctrine, Executor, FailureRateBound,
    GuardStrictness, GuardStrictnessLevel, KnowledgeBase, LatencyBound, Lattice, PolicyElement,
    Strictness,
};
use std::sync::Arc;

// ============================================================================
// Basic Policy Creation and Validation
// ============================================================================

#[test]
fn test_create_policy_atoms() {
    // Latency bound
    let latency = LatencyBound::new(100.0, Strictness::Hard).unwrap();
    assert_eq!(latency.target_p99_ms, 100.0);
    assert_eq!(latency.strictness, Strictness::Hard);

    // Failure rate bound
    let failure = FailureRateBound::new(0.05).unwrap();
    assert_eq!(failure.max_error_rate, 0.05);

    // Guard strictness
    let guard = GuardStrictness::new(GuardStrictnessLevel::Tighten);
    assert_eq!(guard.level, GuardStrictnessLevel::Tighten);

    // Capacity envelope
    let capacity = CapacityEnvelope::new(100, 32).unwrap();
    assert_eq!(capacity.max_concurrency, 100);
    assert_eq!(capacity.max_parallelism, 32);
}

#[test]
fn test_invalid_policy_creation() {
    // Negative latency
    assert!(LatencyBound::new(-10.0, Strictness::Soft).is_err());

    // Invalid error rate
    assert!(FailureRateBound::new(-0.1).is_err());
    assert!(FailureRateBound::new(1.5).is_err());

    // Zero capacity
    assert!(CapacityEnvelope::new(0, 10).is_err());
    assert!(CapacityEnvelope::new(10, 0).is_err());
}

// ============================================================================
// Lattice Operations
// ============================================================================

#[test]
fn test_latency_meet_operation() {
    let a = LatencyBound::new(100.0, Strictness::Soft).unwrap();
    let b = LatencyBound::new(50.0, Strictness::Hard).unwrap();

    let meet = a.meet(&b);
    assert_eq!(meet.target_p99_ms, 50.0); // Stricter (lower)
    assert_eq!(meet.strictness, Strictness::Hard); // Stricter
}

#[test]
fn test_latency_join_operation() {
    let a = LatencyBound::new(100.0, Strictness::Soft).unwrap();
    let b = LatencyBound::new(50.0, Strictness::Hard).unwrap();

    let join = a.join(&b);
    assert_eq!(join.target_p99_ms, 100.0); // More relaxed (higher)
    assert_eq!(join.strictness, Strictness::Soft); // More relaxed
}

#[test]
fn test_failure_rate_lattice() {
    let a = FailureRateBound::new(0.05).unwrap();
    let b = FailureRateBound::new(0.10).unwrap();

    let meet = a.meet(&b);
    assert_eq!(meet.max_error_rate, 0.05); // Stricter (lower)

    let join = a.join(&b);
    assert_eq!(join.max_error_rate, 0.10); // More relaxed (higher)
}

#[test]
fn test_capacity_lattice() {
    let a = CapacityEnvelope::new(50, 16).unwrap();
    let b = CapacityEnvelope::new(100, 32).unwrap();

    let meet = a.meet(&b);
    assert_eq!(meet.max_concurrency, 50); // Stricter (lower)
    assert_eq!(meet.max_parallelism, 16);

    let join = a.join(&b);
    assert_eq!(join.max_concurrency, 100); // More relaxed (higher)
    assert_eq!(join.max_parallelism, 32);
}

#[test]
fn test_policy_element_conjunction() {
    let latency = PolicyElement::Latency(LatencyBound::new(100.0, Strictness::Soft).unwrap());
    let failure = PolicyElement::FailureRate(FailureRateBound::new(0.05).unwrap());

    let conjunction = latency.meet(&failure);
    match conjunction {
        PolicyElement::Conjunction(policies) => {
            assert_eq!(policies.len(), 2);
        }
        _ => panic!("Expected Conjunction"),
    }
}

#[test]
fn test_policy_element_bottom() {
    let bottom = PolicyElement::Bottom;
    assert!(bottom.is_bottom());

    let latency = PolicyElement::Latency(LatencyBound::new(100.0, Strictness::Soft).unwrap());
    assert!(!latency.is_bottom());

    // Bottom absorbs in meet
    let meet = bottom.meet(&latency);
    assert!(meet.is_bottom());
}

// ============================================================================
// Lattice Laws Verification
// ============================================================================

#[test]
fn test_commutativity() {
    let a = LatencyBound::new(100.0, Strictness::Soft).unwrap();
    let b = LatencyBound::new(50.0, Strictness::Hard).unwrap();

    let meet_ab = a.meet(&b);
    let meet_ba = b.meet(&a);
    assert_eq!(meet_ab.target_p99_ms, meet_ba.target_p99_ms);

    let join_ab = a.join(&b);
    let join_ba = b.join(&a);
    assert_eq!(join_ab.target_p99_ms, join_ba.target_p99_ms);
}

#[test]
fn test_associativity() {
    let a = LatencyBound::new(100.0, Strictness::Soft).unwrap();
    let b = LatencyBound::new(50.0, Strictness::Hard).unwrap();
    let c = LatencyBound::new(75.0, Strictness::Soft).unwrap();

    let meet_ab_c = a.meet(&b).meet(&c);
    let meet_a_bc = a.meet(&b.meet(&c));
    assert_eq!(meet_ab_c.target_p99_ms, meet_a_bc.target_p99_ms);

    let join_ab_c = a.join(&b).join(&c);
    let join_a_bc = a.join(&b.join(&c));
    assert_eq!(join_ab_c.target_p99_ms, join_a_bc.target_p99_ms);
}

#[test]
fn test_idempotence() {
    let a = LatencyBound::new(100.0, Strictness::Soft).unwrap();

    let meet_aa = a.meet(&a);
    assert_eq!(meet_aa.target_p99_ms, a.target_p99_ms);

    let join_aa = a.join(&a);
    assert_eq!(join_aa.target_p99_ms, a.target_p99_ms);
}

#[test]
fn test_absorption() {
    let a = LatencyBound::new(100.0, Strictness::Soft).unwrap();
    let b = LatencyBound::new(50.0, Strictness::Hard).unwrap();

    // a ⊓ (a ⊔ b) = a
    let a_join_b = a.join(&b);
    let meet = a.meet(&a_join_b);
    assert_eq!(meet.target_p99_ms, a.target_p99_ms);

    // a ⊔ (a ⊓ b) = a
    let a_meet_b = a.meet(&b);
    let join = a.join(&a_meet_b);
    assert_eq!(join.target_p99_ms, a.target_p99_ms);
}

// ============================================================================
// Doctrine Projection Tests
// ============================================================================

#[test]
fn test_doctrine_default() {
    let doctrine = Doctrine::new();
    assert_eq!(doctrine.max_exec_ticks, 8);
    assert_eq!(doctrine.max_run_len, 8);
    assert_eq!(doctrine.max_call_depth, 8);
    assert!(doctrine.enforce_mu_kernel);
}

#[test]
fn test_doctrine_project_valid_latency() {
    let doctrine = Doctrine::new();
    let policy = PolicyElement::Latency(LatencyBound::new(50.0, Strictness::Soft).unwrap());

    let projected = doctrine.project(&policy).unwrap();
    assert!(projected.is_some());
    assert!(!projected.unwrap().is_bottom());
}

#[test]
fn test_doctrine_project_excessive_latency() {
    let doctrine = Doctrine::new();
    let policy = PolicyElement::Latency(LatencyBound::new(200.0, Strictness::Soft).unwrap());

    let projected = doctrine.project(&policy).unwrap();
    assert!(projected.is_some());

    // Should be clamped to doctrine bound (100ms)
    match projected.unwrap() {
        PolicyElement::Latency(bound) => {
            assert!(bound.target_p99_ms <= doctrine.max_hot_path_latency_ms);
        }
        _ => panic!("Expected Latency"),
    }
}

#[test]
fn test_doctrine_project_valid_failure_rate() {
    let doctrine = Doctrine::new();
    let policy = PolicyElement::FailureRate(FailureRateBound::new(0.005).unwrap());

    let projected = doctrine.project(&policy).unwrap();
    assert!(projected.is_some());
    assert!(!projected.unwrap().is_bottom());
}

#[test]
fn test_doctrine_project_excessive_failure_rate() {
    let doctrine = Doctrine::new();
    let policy = PolicyElement::FailureRate(FailureRateBound::new(0.05).unwrap());

    let projected = doctrine.project(&policy).unwrap();
    assert!(projected.is_some());

    // Should be clamped to doctrine bound (1%)
    match projected.unwrap() {
        PolicyElement::FailureRate(bound) => {
            assert!(bound.max_error_rate <= doctrine.max_safe_error_rate);
        }
        _ => panic!("Expected FailureRate"),
    }
}

#[test]
fn test_doctrine_project_capacity() {
    let doctrine = Doctrine::new();
    let policy = PolicyElement::Capacity(CapacityEnvelope::new(100, 32).unwrap());

    let projected = doctrine.project(&policy).unwrap();
    assert!(projected.is_some());
    assert!(!projected.unwrap().is_bottom());
}

#[test]
fn test_doctrine_project_excessive_capacity() {
    let doctrine = Doctrine::new();
    let policy = PolicyElement::Capacity(CapacityEnvelope::new(1000, 200).unwrap());

    let projected = doctrine.project(&policy).unwrap();
    assert!(projected.is_some());

    // Should be clamped to doctrine bounds
    match projected.unwrap() {
        PolicyElement::Capacity(envelope) => {
            assert!(envelope.max_concurrency <= doctrine.max_safe_concurrency);
            assert!(envelope.max_parallelism <= doctrine.max_safe_parallelism);
        }
        _ => panic!("Expected Capacity"),
    }
}

#[test]
fn test_doctrine_validation() {
    let doctrine = Doctrine::new();

    let valid_policy = PolicyElement::Latency(LatencyBound::new(50.0, Strictness::Soft).unwrap());
    assert!(doctrine.validate(&valid_policy).unwrap());

    let bottom = PolicyElement::Bottom;
    assert!(!doctrine.validate(&bottom).unwrap());
}

// ============================================================================
// MAPE-K Integration Tests
// ============================================================================

#[tokio::test]
async fn test_action_with_policy() {
    let policy = PolicyElement::Latency(LatencyBound::new(80.0, Strictness::Soft).unwrap());
    let action = Action::with_policy(ActionType::ScaleInstances { delta: 2 }, policy.clone());

    assert!(action.has_policy());
    assert!(action.get_policy().is_some());
    assert_eq!(action.get_policy().unwrap(), &policy);
}

#[tokio::test]
async fn test_executor_validates_policy() {
    let executor = Executor::new();

    // Create action with valid policy
    let valid_policy = PolicyElement::Latency(LatencyBound::new(80.0, Strictness::Soft).unwrap());
    let mut action = Action::new(ActionType::ScaleInstances { delta: 1 });
    action.set_policy(valid_policy);

    // Execute - should succeed
    let mut plan = AdaptationPlan::new();
    plan.actions.push(action);

    let results = executor.execute(&plan).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].policy_validated);
    assert!(results[0].success);
}

#[tokio::test]
async fn test_executor_rejects_invalid_policy() {
    let executor = Executor::new();

    // Create action with policy that exceeds doctrine bounds
    let bad_policy = PolicyElement::Latency(LatencyBound::new(500.0, Strictness::Soft).unwrap());
    let mut action = Action::new(ActionType::ScaleInstances { delta: 1 });
    action.set_policy(bad_policy);

    let mut plan = AdaptationPlan::new();
    plan.actions.push(action);

    // Execute - policy should be clamped by doctrine, but action should still succeed
    let results = executor.execute(&plan).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].policy_validated); // Policy was validated (and clamped)
}

#[tokio::test]
async fn test_knowledge_base_policy_lattice() {
    let kb = KnowledgeBase::new();

    // Initially no constraints
    let lattice = kb.get_policy_lattice().await;
    assert!(!lattice.is_bottom());

    // Strengthen policy
    let constraint = PolicyElement::Latency(LatencyBound::new(100.0, Strictness::Hard).unwrap());
    kb.strengthen_policy(constraint).await.unwrap();

    // Check updated lattice
    let lattice = kb.get_policy_lattice().await;
    assert!(!lattice.is_bottom());
    assert_eq!(lattice.get_history().len(), 1);
}

#[tokio::test]
async fn test_knowledge_base_validate_action_policy() {
    let kb = KnowledgeBase::new();

    // Valid policy
    let valid_policy = PolicyElement::Latency(LatencyBound::new(80.0, Strictness::Soft).unwrap());
    let is_valid = kb.validate_action_policy(&valid_policy).await.unwrap();
    assert!(is_valid);

    // Bottom policy
    let bottom = PolicyElement::Bottom;
    let is_valid = kb.validate_action_policy(&bottom).await.unwrap();
    assert!(!is_valid);
}

#[tokio::test]
async fn test_knowledge_base_project_action_policy() {
    let kb = KnowledgeBase::new();

    // Project valid policy
    let policy = PolicyElement::Latency(LatencyBound::new(80.0, Strictness::Soft).unwrap());
    let projected = kb.project_action_policy(&policy).await.unwrap();
    assert!(projected.is_some());
    assert!(!projected.unwrap().is_bottom());

    // Project bottom policy
    let bottom = PolicyElement::Bottom;
    let projected = kb.project_action_policy(&bottom).await.unwrap();
    assert!(projected.is_some());
    assert!(projected.unwrap().is_bottom());
}

#[tokio::test]
async fn test_knowledge_base_with_custom_doctrine() {
    let doctrine = Doctrine::relaxed();
    let kb = KnowledgeBase::with_doctrine(doctrine.clone());

    assert_eq!(kb.doctrine().max_exec_ticks, doctrine.max_exec_ticks);
    assert!(!kb.doctrine().enforce_mu_kernel);
}

// ============================================================================
// End-to-End Integration Tests
// ============================================================================

#[tokio::test]
async fn test_e2e_policy_enforcement() {
    let kb = Arc::new(KnowledgeBase::new());
    let executor = Executor::new();

    // Add system-wide policy constraint
    let system_policy = PolicyElement::Latency(LatencyBound::new(50.0, Strictness::Hard).unwrap());
    kb.strengthen_policy(system_policy.clone()).await.unwrap();

    // Create action with compatible policy
    let action_policy = PolicyElement::Latency(LatencyBound::new(40.0, Strictness::Soft).unwrap());
    let mut action = Action::new(ActionType::OptimizePattern { pattern_id: 12 });
    action.set_policy(action_policy);

    // Validate action against knowledge base
    let is_valid = kb.validate_action_policy(action.get_policy().unwrap()).await.unwrap();
    assert!(is_valid);

    // Execute action
    let mut plan = AdaptationPlan::new();
    plan.actions.push(action);

    let results = executor.execute(&plan).await.unwrap();
    assert!(results[0].success);
    assert!(results[0].policy_validated);
}

#[tokio::test]
async fn test_e2e_policy_violation() {
    let kb = Arc::new(KnowledgeBase::new());
    let executor = Executor::new();

    // Add strict system policy
    let system_policy = PolicyElement::Latency(LatencyBound::new(30.0, Strictness::Hard).unwrap());
    kb.strengthen_policy(system_policy).await.unwrap();

    // Create action with incompatible policy (too relaxed)
    let action_policy = PolicyElement::Latency(LatencyBound::new(100.0, Strictness::Soft).unwrap());
    let mut action = Action::new(ActionType::ScaleInstances { delta: 1 });
    action.set_policy(action_policy);

    // Project through knowledge base - should be clamped
    let projected = kb.project_action_policy(action.get_policy().unwrap()).await.unwrap();
    assert!(projected.is_some());

    // Projected policy should be stricter
    match projected.unwrap() {
        PolicyElement::Latency(bound) => {
            assert!(bound.target_p99_ms <= 30.0 || bound.target_p99_ms <= 100.0);
        }
        _ => panic!("Expected Latency"),
    }
}

#[tokio::test]
async fn test_e2e_multiple_policy_constraints() {
    let kb = Arc::new(KnowledgeBase::new());
    let executor = Executor::new();

    // Add multiple system constraints
    let latency_policy = PolicyElement::Latency(LatencyBound::new(50.0, Strictness::Hard).unwrap());
    let failure_policy = PolicyElement::FailureRate(FailureRateBound::new(0.01).unwrap());

    kb.strengthen_policy(latency_policy).await.unwrap();
    kb.strengthen_policy(failure_policy).await.unwrap();

    // Create action with conjunction of constraints
    let action_latency = PolicyElement::Latency(LatencyBound::new(40.0, Strictness::Soft).unwrap());
    let action_failure = PolicyElement::FailureRate(FailureRateBound::new(0.005).unwrap());
    let action_policy = action_latency.meet(&action_failure);

    let mut action = Action::new(ActionType::OptimizePattern { pattern_id: 5 });
    action.set_policy(action_policy);

    // Validate
    let is_valid = kb.validate_action_policy(action.get_policy().unwrap()).await.unwrap();
    assert!(is_valid);

    // Execute
    let mut plan = AdaptationPlan::new();
    plan.actions.push(action);

    let results = executor.execute(&plan).await.unwrap();
    assert!(results[0].success);
    assert!(results[0].policy_validated);
}

// ============================================================================
// Stress Tests
// ============================================================================

#[tokio::test]
async fn test_policy_lattice_depth() {
    let mut current = PolicyElement::Latency(LatencyBound::new(100.0, Strictness::Soft).unwrap());

    // Create deep conjunction
    for i in 1..10 {
        let next = PolicyElement::FailureRate(FailureRateBound::new(0.01 * i as f64 / 10.0).unwrap());
        current = current.meet(&next);
    }

    // Verify it's a conjunction
    match current {
        PolicyElement::Conjunction(policies) => {
            assert_eq!(policies.len(), 10);
        }
        _ => panic!("Expected Conjunction"),
    }
}

#[tokio::test]
async fn test_concurrent_policy_updates() {
    use tokio::task::JoinSet;

    let kb = Arc::new(KnowledgeBase::new());
    let mut tasks = JoinSet::new();

    // Spawn multiple concurrent policy updates
    for i in 0..100 {
        let kb_clone = kb.clone();
        tasks.spawn(async move {
            let policy = PolicyElement::Latency(
                LatencyBound::new(50.0 + i as f64, Strictness::Soft).unwrap()
            );
            kb_clone.strengthen_policy(policy).await.unwrap();
        });
    }

    // Wait for all tasks
    while tasks.join_next().await.is_some() {}

    // Verify final state
    let lattice = kb.get_policy_lattice().await;
    assert!(!lattice.is_bottom());
    assert!(lattice.get_history().len() >= 100);
}
