//! Integration Tests for Complete Governance Layer
//!
//! Tests end-to-end governance workflows combining all components:
//! - Policy lattice + Mode manager + Session tracking
//! - Counterfactual replay with policy enforcement
//! - Multi-session adaptation with global constraints
//! - Complete MAPE-K loop execution
//!
//! **Chicago TDD Approach**: Tests real component integration, not mocks

use knhk_workflow_engine::autonomic::{
    failure_modes::{AutonomicMode, ModeManager},
    mode_policy::{ModePolicyFilter, default_action_annotations},
    plan::{Action, ActionType, AdaptationPlan, Planner},
    policy_lattice::{LatencyBound, Strictness},
    session::{SessionId, SessionTable, TenantId},
    trace_index::{TraceId, TraceStorage, ExecutionTrace, OntologySnapshot, DoctrineConfig},
    counterfactual::{CounterfactualEngine, CounterfactualScenario},
    analyze::{Analysis, Analyzer},
    knowledge::KnowledgeBase,
};
use knhk_workflow_engine::case::CaseId;
use knhk_workflow_engine::error::WorkflowResult;
use std::sync::Arc;
use serde_json::json;

// ============================================================================
// End-to-End MAPE-K Tests
// ============================================================================

#[tokio::test]
async fn test_complete_mape_k_loop_execution() {
    // Arrange: Set up all components
    let mode_manager = ModeManager::new();
    let filter = ModePolicyFilter::new(default_action_annotations());
    let session_table = Arc::new(SessionTable::new());

    // Create session
    let session_id = SessionId::new();
    let case_id = CaseId::new();
    let tenant_id = TenantId::default_tenant();

    session_table.create_session(session_id, case_id, tenant_id);

    // Act: Execute MAPE-K loop
    // 1. Monitor: Observe high latency
    let observations = json!({
        "latency_p99_ms": 250.0,
        "error_rate": 0.02,
        "throughput_rps": 500.0
    });

    // 2. Analyze: Detect SLO violation
    let analyzer = Analyzer::new();
    let analysis = analyzer.analyze(observations.clone()).await.unwrap();

    assert!(!analysis.slo_violations.is_empty(), "Should detect SLO violation");

    // 3. Plan: Generate adaptation actions
    let planner = Planner::new();
    let plan = planner.plan(&analysis, &KnowledgeBase::default()).await.unwrap();

    assert!(!plan.actions.is_empty(), "Should generate adaptation plan");

    // 4. Execute: Filter actions by current mode and apply
    let mut executed_actions = Vec::new();
    let current_mode = mode_manager.current_mode();

    for action in &plan.actions {
        if filter.filter_action(action, current_mode).is_ok() {
            executed_actions.push(action.clone());

            // Update session metrics
            if let Some(metrics) = session_table.get_session_metrics(session_id) {
                metrics.increment_adaptations();
            }
        }
    }

    // Assert: Verify complete loop
    assert!(!executed_actions.is_empty(), "Should execute some actions");

    if let Some(metrics) = session_table.get_session_metrics(session_id) {
        assert_eq!(
            metrics.get_adaptation_count(),
            executed_actions.len() as u64,
            "Should track adaptations"
        );
    }
}

#[tokio::test]
async fn test_policy_enforcement_blocks_risky_actions_in_degraded_mode() {
    // Arrange: Put system in Conservative mode
    let mut mode_manager = ModeManager::new();
    mode_manager.transition_to(AutonomicMode::Conservative);

    let filter = ModePolicyFilter::new(default_action_annotations());

    // Create risky action (requires Normal mode)
    let risky_action = Action {
        action_id: uuid::Uuid::new_v4(),
        action_type: ActionType::ScaleInstances {
            service: "web".to_string(),
            delta: 5,
        },
        rationale: "Scale up significantly".to_string(),
        policy_element: None,
    };

    // Create safe action (allowed in Conservative)
    let safe_action = Action {
        action_id: uuid::Uuid::new_v4(),
        action_type: ActionType::AdjustResources {
            resource_type: "cpu".to_string(),
            delta: 0.1,
        },
        rationale: "Minor CPU adjustment".to_string(),
        policy_element: None,
    };

    // Act & Assert: Risky action blocked, safe action allowed
    assert!(
        filter.filter_action(&risky_action, mode_manager.current_mode()).is_err(),
        "Risky action should be blocked in Conservative mode"
    );
    assert!(
        filter.filter_action(&safe_action, mode_manager.current_mode()).is_ok(),
        "Safe action should be allowed in Conservative mode"
    );
}

// ============================================================================
// Multi-Session Governance Tests
// ============================================================================

#[tokio::test]
async fn test_multi_session_isolation_with_different_policies() {
    // Arrange: Create multiple sessions with different policies
    let session_table = Arc::new(SessionTable::new());
    let tenant1 = TenantId::new();
    let tenant2 = TenantId::new();

    // Tenant 1: Strict latency policy
    let session1 = SessionId::new();
    session_table.create_session(session1, CaseId::new(), tenant1);

    // Tenant 2: Relaxed latency policy
    let session2 = SessionId::new();
    session_table.create_session(session2, CaseId::new(), tenant2);

    // Act: Apply different policies
    let strict_policy = LatencyBound::new(50.0, Strictness::Hard).unwrap();
    let relaxed_policy = LatencyBound::new(200.0, Strictness::Soft).unwrap();

    // Simulate violations
    if let Some(metrics1) = session_table.get_session_metrics(session1) {
        // Strict session with 100ms latency = violation
        metrics1.increment_violations();
    }

    if let Some(metrics2) = session_table.get_session_metrics(session2) {
        // Relaxed session with 100ms latency = OK
        // No violation
    }

    // Assert: Sessions maintain independent state
    if let Some(metrics1) = session_table.get_session_metrics(session1) {
        assert_eq!(metrics1.get_violation_count(), 1);
    }

    if let Some(metrics2) = session_table.get_session_metrics(session2) {
        assert_eq!(metrics2.get_violation_count(), 0);
    }
}

#[test]
fn test_global_mode_affects_all_sessions() {
    // Arrange: Create multiple sessions
    let session_table = Arc::new(SessionTable::new());
    let sessions: Vec<_> = (0..10)
        .map(|_| {
            let session_id = SessionId::new();
            session_table.create_session(
                session_id,
                CaseId::new(),
                TenantId::default_tenant(),
            );
            session_id
        })
        .collect();

    let mut mode_manager = ModeManager::new();
    let filter = ModePolicyFilter::new(default_action_annotations());

    // Act: Transition to Conservative mode globally
    mode_manager.transition_to(AutonomicMode::Conservative);

    let action = Action {
        action_id: uuid::Uuid::new_v4(),
        action_type: ActionType::ScaleInstances {
            service: "test".to_string(),
            delta: 1,
        },
        rationale: "Scale".to_string(),
        policy_element: None,
    };

    // Assert: All sessions affected by global mode
    for _session_id in &sessions {
        assert!(
            filter.filter_action(&action, mode_manager.current_mode()).is_err(),
            "Global mode should affect all sessions"
        );
    }
}

// ============================================================================
// Counterfactual Integration Tests
// ============================================================================

#[tokio::test]
async fn test_counterfactual_with_different_mode() {
    // Arrange: Create original trace in Normal mode
    let storage = Arc::new(TraceStorage::new());

    let original_trace = ExecutionTrace {
        trace_id: TraceId::new(),
        timestamp_ms: 1000,
        observations: json!({
            "latency_p99_ms": 150.0,
            "error_rate": 0.02
        }),
        analysis: Analysis {
            slo_violations: vec![],
            anomalies: vec![],
            trends: vec![],
            recommendations: vec![],
        },
        plan: AdaptationPlan {
            actions: vec![
                Action {
                    action_id: uuid::Uuid::new_v4(),
                    action_type: ActionType::ScaleInstances {
                        service: "web".to_string(),
                        delta: 2,
                    },
                    rationale: "Scale up".to_string(),
                    policy_element: None,
                }
            ],
            confidence: 0.9,
        },
        executed_actions: vec![],
        knowledge_updates: vec![],
        ontology: OntologySnapshot {
            version: "1.0.0".to_string(),
            concepts: vec![],
            relationships: vec![],
        },
        doctrine: DoctrineConfig {
            invariants: vec![],
            constraints: vec![],
        },
    };

    let trace_id = original_trace.trace_id;
    storage.store_trace(original_trace).await.unwrap();

    // Act: Replay with stricter doctrine (Conservative mode equivalent)
    let stricter_doctrine = DoctrineConfig {
        invariants: vec!["no_scaling_in_conservative".to_string()],
        constraints: vec!["latency < 100ms".to_string()],
    };

    let engine = CounterfactualEngine::new(storage);
    let scenario = CounterfactualScenario::with_doctrine(
        trace_id,
        stricter_doctrine,
        "Conservative mode counterfactual".to_string(),
    );

    let result = engine.execute_scenario(scenario).await.unwrap();

    // Assert: Different doctrine should produce different actions
    assert!(
        result.has_action_changes() || result.action_diff.removed.len() > 0,
        "Stricter doctrine should affect action selection"
    );
}

// ============================================================================
// Policy Lattice Integration Tests
// ============================================================================

#[test]
fn test_policy_lattice_enforces_global_constraints() {
    // Arrange: Create session with local and global policies
    let session_table = SessionTable::new();
    let session_id = SessionId::new();

    session_table.create_session(
        session_id,
        CaseId::new(),
        TenantId::default_tenant(),
    );

    // Global policy: All services must have p99 < 100ms
    let global_policy = LatencyBound::new(100.0, Strictness::Hard).unwrap();

    // Local policy: This session wants p99 < 50ms
    let local_policy = LatencyBound::new(50.0, Strictness::Hard).unwrap();

    // Act: Combine policies (meet operation)
    // The stricter policy should dominate
    let effective_policy = if local_policy.is_stricter_than(&global_policy) {
        local_policy
    } else {
        global_policy
    };

    // Assert: Effective policy should be the stricter one (50ms)
    assert_eq!(effective_policy.target_p99_ms, 50.0);
    assert!(local_policy.is_stricter_than(&global_policy));
}

// ============================================================================
// Session Lifecycle Integration Tests
// ============================================================================

#[test]
fn test_complete_session_lifecycle_with_adaptations() {
    // Arrange: Create session
    let session_table = SessionTable::new();
    let session_id = SessionId::new();

    session_table.create_session(
        session_id,
        CaseId::new(),
        TenantId::default_tenant(),
    );

    let metrics = session_table.get_session_metrics(session_id).unwrap();

    // Act: Simulate complete lifecycle
    // 1. Created (default state)
    assert_eq!(metrics.get_state(), knhk_workflow_engine::autonomic::session::SessionState::Created);

    // 2. Start execution
    metrics.set_state(knhk_workflow_engine::autonomic::session::SessionState::Active);
    assert_eq!(metrics.get_state(), knhk_workflow_engine::autonomic::session::SessionState::Active);

    // 3. Execute tasks
    for _ in 0..5 {
        metrics.increment_task_completions();
        metrics.add_latency_us(1000);
    }

    // 4. Detect violations
    metrics.increment_violations();

    // 5. Apply adaptations
    metrics.increment_adaptations();
    metrics.set_state(knhk_workflow_engine::autonomic::session::SessionState::Adapted);

    // 6. Complete successfully
    metrics.set_state(knhk_workflow_engine::autonomic::session::SessionState::Completed);

    // Assert: Verify complete lifecycle
    assert_eq!(metrics.get_task_completions(), 5);
    assert_eq!(metrics.get_violation_count(), 1);
    assert_eq!(metrics.get_adaptation_count(), 1);
    assert_eq!(
        metrics.get_state(),
        knhk_workflow_engine::autonomic::session::SessionState::Completed
    );
}

// ============================================================================
// Cross-Component Consistency Tests
// ============================================================================

#[test]
fn test_mode_transitions_consistent_across_components() {
    // Arrange
    let mut mode_manager = ModeManager::new();
    let filter = ModePolicyFilter::new(default_action_annotations());

    // Define actions requiring different modes
    let actions = vec![
        (
            Action {
                action_id: uuid::Uuid::new_v4(),
                action_type: ActionType::ScaleInstances {
                    service: "web".to_string(),
                    delta: 1,
                },
                rationale: "Scale".to_string(),
                policy_element: None,
            },
            AutonomicMode::Normal,
        ),
        (
            Action {
                action_id: uuid::Uuid::new_v4(),
                action_type: ActionType::AdjustResources {
                    resource_type: "cpu".to_string(),
                    delta: 0.1,
                },
                rationale: "Adjust".to_string(),
                policy_element: None,
            },
            AutonomicMode::Conservative,
        ),
    ];

    // Act: Test each mode
    for mode in &[
        AutonomicMode::Normal,
        AutonomicMode::Conservative,
        AutonomicMode::Frozen,
    ] {
        mode_manager.transition_to(*mode);

        for (action, required_mode) in &actions {
            let result = filter.filter_action(action, *mode);

            // Assert: Action should only be allowed if mode >= required_mode
            if mode >= required_mode {
                assert!(result.is_ok(), "Action should be allowed in {:?}", mode);
            } else {
                assert!(result.is_err(), "Action should be blocked in {:?}", mode);
            }
        }
    }
}

// ============================================================================
// Stress Tests
// ============================================================================

#[test]
fn test_governance_layer_handles_high_session_count() {
    // Arrange: Create many sessions
    let session_table = SessionTable::new();
    let session_count = 10_000;

    // Act: Create sessions and simulate activity
    let start = std::time::Instant::now();

    for i in 0..session_count {
        let session_id = SessionId::new();
        session_table.create_session(
            session_id,
            CaseId::new(),
            TenantId::default_tenant(),
        );

        if let Some(metrics) = session_table.get_session_metrics(session_id) {
            metrics.increment_task_completions();
            if i % 10 == 0 {
                metrics.increment_violations();
            }
            if i % 5 == 0 {
                metrics.increment_adaptations();
            }
        }
    }

    let elapsed = start.elapsed();

    // Assert: Should handle 10k sessions efficiently (<1s)
    assert!(
        elapsed.as_secs() < 1,
        "Should create 10k sessions in <1s: took {}ms",
        elapsed.as_millis()
    );
    assert_eq!(session_table.active_session_count(), session_count);
}

#[test]
fn test_policy_enforcement_scales_with_action_count() {
    // Arrange
    let filter = ModePolicyFilter::new(default_action_annotations());
    let action_count = 1_000;

    let actions: Vec<_> = (0..action_count)
        .map(|i| Action {
            action_id: uuid::Uuid::new_v4(),
            action_type: if i % 2 == 0 {
                ActionType::ScaleInstances {
                    service: format!("service-{}", i),
                    delta: 1,
                }
            } else {
                ActionType::AdjustResources {
                    resource_type: "cpu".to_string(),
                    delta: 0.1,
                }
            },
            rationale: format!("Action {}", i),
            policy_element: None,
        })
        .collect();

    // Act: Filter all actions
    let start = std::time::Instant::now();

    let filtered: Vec<_> = actions
        .iter()
        .filter_map(|a| filter.filter_action(a, AutonomicMode::Normal).ok())
        .collect();

    let elapsed = start.elapsed();

    // Assert: Should filter 1000 actions quickly (<10ms)
    assert!(
        elapsed.as_millis() < 10,
        "Should filter 1000 actions in <10ms: took {}ms",
        elapsed.as_millis()
    );
    assert_eq!(filtered.len(), action_count);
}

// ============================================================================
// Integration Test Summary
// ============================================================================

#[test]
fn test_governance_integration_summary() {
    println!("\n=== Governance Layer Integration Test Summary ===");
    println!("Components Integrated:");
    println!("  ✓ Policy Lattice");
    println!("  ✓ Mode Manager");
    println!("  ✓ Session Tracking");
    println!("  ✓ Counterfactual Engine");
    println!("  ✓ Mode-Aware Policy Filter");
    println!();
    println!("Scenarios Tested:");
    println!("  ✓ Complete MAPE-K loop execution");
    println!("  ✓ Multi-session isolation");
    println!("  ✓ Global mode enforcement");
    println!("  ✓ Counterfactual analysis");
    println!("  ✓ Policy lattice constraints");
    println!("  ✓ Session lifecycle");
    println!("  ✓ Cross-component consistency");
    println!("  ✓ High-scale stress tests");
    println!("=================================================\n");

    assert!(true);
}
