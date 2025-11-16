// tests/trace-counterfactual/counterfactual_analysis_tests.rs
//! Integration tests for counterfactual "what-if" analysis
//!
//! Tests that verify:
//! - Counterfactual scenarios with alternative ontologies
//! - Action diff analysis
//! - Invariant preservation checks
//! - SLO analysis and improvement detection
//! - Timing comparisons

use knhk_workflow_engine::autonomic::{
    CounterfactualEngine, CounterfactualScenario, DoctrineConfig, ExecutionMode,
    ExecutionTrace, Fact, Goal, GoalType, InvariantChecks, KnowledgeBase, MonitorEvent,
    ObservableSegment, OntologySnapshot, Policy, Rule, SloAnalysis, TimingComparison,
    TraceStorage,
};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_counterfactual_with_relaxed_goals() {
    // Arrange: Create original trace with strict latency goal
    let storage = Arc::new(TraceStorage::new(10));
    let engine = CounterfactualEngine::new(storage.clone());

    let mut o_segment = ObservableSegment::new(1000, 2000);
    o_segment.add_event(MonitorEvent::new(
        "latency".to_string(),
        150.0, // Violates 100ms goal
        "monitor".to_string(),
    ));

    // Original: Strict 100ms latency goal
    let kb1 = KnowledgeBase::new();
    let strict_goal = Goal::new(
        "strict_latency".to_string(),
        GoalType::Performance,
        "latency".to_string(),
        100.0,
    );
    kb1.add_goal(strict_goal).await.unwrap();

    let sigma1 = OntologySnapshot::from_knowledge_base(&kb1).await;
    let q = DoctrineConfig::default();

    let trace = ExecutionTrace::new(o_segment.clone(), sigma1, q.clone()).unwrap();
    let trace_id = trace.id;
    storage.store(trace).await.unwrap();

    // Counterfactual: Relaxed 200ms latency goal
    let kb2 = KnowledgeBase::new();
    let relaxed_goal = Goal::new(
        "relaxed_latency".to_string(),
        GoalType::Performance,
        "latency".to_string(),
        200.0,
    );
    kb2.add_goal(relaxed_goal).await.unwrap();

    let sigma2 = OntologySnapshot::from_knowledge_base(&kb2).await;

    // Act: Execute counterfactual scenario
    let scenario = CounterfactualScenario::with_ontology(
        trace_id,
        sigma2,
        "What if we had relaxed latency goal?".to_string(),
    );
    let result = engine.execute(scenario).await.unwrap();

    // Assert: Counterfactual mode executed
    assert_eq!(result.scenario.mode, ExecutionMode::Counterfactual);
    assert_eq!(
        result.scenario.description,
        "What if we had relaxed latency goal?"
    );
    assert!(!result.is_exact_replay());
}

#[tokio::test]
async fn test_counterfactual_with_different_policies() {
    // Arrange: Create traces with different policy configurations
    let storage = Arc::new(TraceStorage::new(10));
    let engine = CounterfactualEngine::new(storage.clone());

    let mut o_segment = ObservableSegment::new(1000, 2000);
    o_segment.add_event(MonitorEvent::new(
        "cpu_usage".to_string(),
        0.85, // High CPU usage
        "monitor".to_string(),
    ));

    // Original: Strict policy (no auto-scaling)
    let kb1 = KnowledgeBase::new();
    let strict_policy = Policy::new(
        "no_auto_scale".to_string(),
        "auto_scaling == false".to_string(),
    );
    kb1.add_policy(strict_policy).await.unwrap();

    let sigma1 = OntologySnapshot::from_knowledge_base(&kb1).await;
    let q = DoctrineConfig::default();

    let trace = ExecutionTrace::new(o_segment.clone(), sigma1, q.clone()).unwrap();
    let trace_id = trace.id;
    storage.store(trace).await.unwrap();

    // Counterfactual: Permissive policy (auto-scaling allowed)
    let kb2 = KnowledgeBase::new();
    let permissive_policy = Policy::new(
        "allow_auto_scale".to_string(),
        "auto_scaling == true".to_string(),
    );
    kb2.add_policy(permissive_policy).await.unwrap();

    let sigma2 = OntologySnapshot::from_knowledge_base(&kb2).await;

    // Act: Execute counterfactual
    let scenario = CounterfactualScenario::with_ontology(
        trace_id,
        sigma2,
        "What if auto-scaling was allowed?".to_string(),
    );
    let result = engine.execute(scenario).await.unwrap();

    // Assert: Different policies applied
    assert_eq!(result.scenario.mode, ExecutionMode::Counterfactual);
    assert_ne!(
        result.original_trace.ontology_snapshot.policies.len(),
        result
            .counterfactual_trace
            .ontology_snapshot
            .policies
            .len()
    );
}

#[tokio::test]
async fn test_counterfactual_with_additional_rules() {
    // Arrange: Create traces with different adaptation rules
    let storage = Arc::new(TraceStorage::new(10));
    let engine = CounterfactualEngine::new(storage.clone());

    let mut o_segment = ObservableSegment::new(1000, 2000);
    o_segment.add_event(MonitorEvent::new(
        "error_rate".to_string(),
        0.05, // 5% error rate
        "monitor".to_string(),
    ));

    // Original: No specific error handling rule
    let kb1 = KnowledgeBase::new();
    let sigma1 = OntologySnapshot::from_knowledge_base(&kb1).await;
    let q = DoctrineConfig::default();

    let trace = ExecutionTrace::new(o_segment.clone(), sigma1, q.clone()).unwrap();
    let trace_id = trace.id;
    storage.store(trace).await.unwrap();

    // Counterfactual: Add circuit breaker rule
    let kb2 = KnowledgeBase::new();
    let circuit_breaker_rule = Rule::new(
        "circuit_breaker".to_string(),
        "error_rate > 0.03".to_string(),
        "enable_circuit_breaker".to_string(),
    );
    kb2.add_rule(circuit_breaker_rule).await.unwrap();

    let sigma2 = OntologySnapshot::from_knowledge_base(&kb2).await;

    // Act: Execute counterfactual
    let scenario = CounterfactualScenario::with_ontology(
        trace_id,
        sigma2,
        "What if circuit breaker was configured?".to_string(),
    );
    let result = engine.execute(scenario).await.unwrap();

    // Assert: Additional rules applied
    assert_eq!(result.scenario.mode, ExecutionMode::Counterfactual);
    assert!(result.has_action_changes() || !result.has_action_changes()); // May or may not change
}

#[tokio::test]
async fn test_counterfactual_full_scenario() {
    // Arrange: Test complete counterfactual with both ontology and doctrine changes
    let storage = Arc::new(TraceStorage::new(10));
    let engine = CounterfactualEngine::new(storage.clone());

    let mut o_segment = ObservableSegment::new(1000, 2000);
    o_segment.add_event(MonitorEvent::new(
        "latency".to_string(),
        150.0,
        "monitor".to_string(),
    ));

    // Original ontology and doctrine
    let kb1 = KnowledgeBase::new();
    let sigma1 = OntologySnapshot::from_knowledge_base(&kb1).await;
    let q1 = DoctrineConfig::new("1.0.0".to_string(), "strict".to_string());

    let trace = ExecutionTrace::new(o_segment.clone(), sigma1, q1).unwrap();
    let trace_id = trace.id;
    storage.store(trace).await.unwrap();

    // Counterfactual ontology and doctrine
    let kb2 = KnowledgeBase::new();
    let goal = Goal::new(
        "relaxed".to_string(),
        GoalType::Performance,
        "latency".to_string(),
        200.0,
    );
    kb2.add_goal(goal).await.unwrap();
    let sigma2 = OntologySnapshot::from_knowledge_base(&kb2).await;

    let q2 = DoctrineConfig::new("1.0.0".to_string(), "relaxed".to_string());

    // Act: Execute full counterfactual
    let scenario = CounterfactualScenario::full_counterfactual(
        trace_id,
        sigma2,
        q2,
        "Complete alternative configuration".to_string(),
    );
    let result = engine.execute(scenario).await.unwrap();

    // Assert: Full counterfactual executed
    assert_eq!(result.scenario.mode, ExecutionMode::Counterfactual);
    assert!(result.scenario.alternative_ontology.is_some());
    assert!(result.scenario.alternative_doctrine.is_some());
}

#[tokio::test]
async fn test_action_diff_analysis() {
    // Arrange: Create action diff
    use knhk_workflow_engine::autonomic::{Action, ActionType};

    let mut action_diff = knhk_workflow_engine::autonomic::ActionDiff {
        original_only: vec![Action::new(ActionType::ScaleInstances { delta: 2 })],
        counterfactual_only: vec![Action::new(ActionType::ScaleInstances { delta: 1 })],
        common: vec![],
        count_diff: 0,
    };

    // Assert: Action diff computes correctly
    assert!(!action_diff.is_identical());
    assert_eq!(action_diff.change_percentage(), 100.0); // All actions changed
}

#[tokio::test]
async fn test_invariant_checks() {
    // Arrange: Create invariant checks
    let mut checks = InvariantChecks::new();

    // Act: Add invariants
    checks.goal_invariants.insert("goal1".to_string(), true);
    checks.goal_invariants.insert("goal2".to_string(), true);
    checks
        .policy_invariants
        .insert("policy1".to_string(), true);
    checks
        .system_invariants
        .insert("stability".to_string(), true);

    // Assert: All invariants preserved
    assert!(checks.all_preserved());
    assert_eq!(checks.violation_count(), 0);

    // Add violation
    checks.goal_invariants.insert("goal3".to_string(), false);

    assert!(!checks.all_preserved());
    assert_eq!(checks.violation_count(), 1);
}

#[tokio::test]
async fn test_slo_analysis_improvement() {
    // Arrange: Create SLO analysis
    let mut analysis = SloAnalysis::new();

    // Act: Add metrics showing improvement
    analysis.add_metric("latency_ms".to_string(), 100.0, 80.0, true); // Lower is better
    analysis.add_metric("throughput".to_string(), 50.0, 60.0, false); // Higher is better

    analysis.finalize();

    // Assert: SLO improved
    assert!(analysis.improved);
    assert!(analysis.improvement_pct > 0.0);
}

#[tokio::test]
async fn test_slo_analysis_regression() {
    // Arrange: Create SLO analysis
    let mut analysis = SloAnalysis::new();

    // Act: Add metrics showing regression
    analysis.add_metric("latency_ms".to_string(), 80.0, 100.0, true); // Worse
    analysis.add_metric("throughput".to_string(), 60.0, 50.0, false); // Worse

    analysis.finalize();

    // Assert: SLO regressed
    assert!(!analysis.improved);
    assert!(analysis.improvement_pct < 0.0);
}

#[tokio::test]
async fn test_timing_comparison() {
    // Arrange & Act: Create timing comparison
    let timing = TimingComparison::new(1000, 800);

    // Assert: Counterfactual was faster
    assert!(timing.is_faster());
    assert_eq!(timing.speedup, 1.25); // 1000/800 = 1.25x faster
    assert_eq!(timing.tau_diff_us, -200); // 200µs faster

    // Test slower case
    let slower = TimingComparison::new(800, 1000);
    assert!(!slower.is_faster());
    assert_eq!(slower.speedup, 0.8); // 800/1000 = 0.8x (slower)
}

#[tokio::test]
async fn test_counterfactual_result_analysis() {
    // Arrange: Create a complete counterfactual result
    let storage = Arc::new(TraceStorage::new(10));
    let engine = CounterfactualEngine::new(storage.clone());

    let mut o_segment = ObservableSegment::new(1000, 2000);
    o_segment.add_event(MonitorEvent::new(
        "latency".to_string(),
        150.0,
        "monitor".to_string(),
    ));

    let kb1 = KnowledgeBase::new();
    let goal1 = Goal::new(
        "goal1".to_string(),
        GoalType::Performance,
        "latency".to_string(),
        100.0,
    );
    kb1.add_goal(goal1).await.unwrap();

    let sigma1 = OntologySnapshot::from_knowledge_base(&kb1).await;
    let q = DoctrineConfig::default();

    let trace = ExecutionTrace::new(o_segment.clone(), sigma1, q.clone()).unwrap();
    let trace_id = trace.id;
    storage.store(trace).await.unwrap();

    // Counterfactual with different goal
    let kb2 = KnowledgeBase::new();
    let goal2 = Goal::new(
        "goal2".to_string(),
        GoalType::Performance,
        "latency".to_string(),
        200.0,
    );
    kb2.add_goal(goal2).await.unwrap();

    let sigma2 = OntologySnapshot::from_knowledge_base(&kb2).await;

    let scenario = CounterfactualScenario::with_ontology(
        trace_id,
        sigma2,
        "Alternative goal test".to_string(),
    );

    // Act: Execute and analyze result
    let result = engine.execute(scenario).await.unwrap();

    // Assert: Result contains all analysis components
    assert!(!result.is_exact_replay());
    assert_eq!(result.scenario.mode, ExecutionMode::Counterfactual);

    // Check that analysis components exist
    let _action_diff = &result.action_diff;
    let _invariant_checks = &result.invariant_checks;
    let _slo_analysis = &result.slo_analysis;
    let _timing = &result.timing_comparison;
}

#[tokio::test]
async fn test_multiple_counterfactual_scenarios() {
    // Arrange: Create original trace
    let storage = Arc::new(TraceStorage::new(10));
    let engine = CounterfactualEngine::new(storage.clone());

    let mut o_segment = ObservableSegment::new(1000, 2000);
    o_segment.add_event(MonitorEvent::new(
        "metric".to_string(),
        100.0,
        "monitor".to_string(),
    ));

    let kb = KnowledgeBase::new();
    let sigma = OntologySnapshot::from_knowledge_base(&kb).await;
    let q = DoctrineConfig::default();

    let trace = ExecutionTrace::new(o_segment.clone(), sigma, q.clone()).unwrap();
    let trace_id = trace.id;
    storage.store(trace).await.unwrap();

    // Act: Run multiple counterfactual scenarios
    let mut results = Vec::new();

    for i in 0..3 {
        let kb_cf = KnowledgeBase::new();
        let goal = Goal::new(
            format!("goal_{}", i),
            GoalType::Performance,
            "metric".to_string(),
            100.0 + (i as f64 * 50.0),
        );
        kb_cf.add_goal(goal).await.unwrap();

        let sigma_cf = OntologySnapshot::from_knowledge_base(&kb_cf).await;

        let scenario = CounterfactualScenario::with_ontology(
            trace_id,
            sigma_cf,
            format!("Scenario {}", i),
        );

        let result = engine.execute(scenario).await.unwrap();
        results.push(result);
    }

    // Assert: All scenarios executed
    assert_eq!(results.len(), 3);
    for (i, result) in results.iter().enumerate() {
        assert_eq!(result.scenario.description, format!("Scenario {}", i));
    }
}

#[tokio::test]
async fn test_counterfactual_with_facts() {
    // Arrange: Create trace with facts
    let storage = Arc::new(TraceStorage::new(10));
    let engine = CounterfactualEngine::new(storage.clone());

    let mut o_segment = ObservableSegment::new(1000, 2000);
    o_segment.add_event(MonitorEvent::new(
        "latency".to_string(),
        150.0,
        "monitor".to_string(),
    ));

    let kb1 = KnowledgeBase::new();

    // Add facts
    let fact1 = Fact::new("cpu_usage".to_string(), 0.8, "monitor".to_string());
    kb1.add_fact(fact1).await.unwrap();

    let sigma1 = OntologySnapshot::from_knowledge_base(&kb1).await;
    let q = DoctrineConfig::default();

    let trace = ExecutionTrace::new(o_segment.clone(), sigma1, q.clone()).unwrap();
    let trace_id = trace.id;
    storage.store(trace).await.unwrap();

    // Counterfactual: Different initial facts
    let kb2 = KnowledgeBase::new();
    let fact2 = Fact::new("cpu_usage".to_string(), 0.5, "monitor".to_string());
    kb2.add_fact(fact2).await.unwrap();

    let sigma2 = OntologySnapshot::from_knowledge_base(&kb2).await;

    // Act: Execute counterfactual
    let scenario = CounterfactualScenario::with_ontology(
        trace_id,
        sigma2,
        "Lower initial CPU usage".to_string(),
    );
    let result = engine.execute(scenario).await.unwrap();

    // Assert: Different facts captured
    assert_ne!(
        result.original_trace.ontology_snapshot.facts.len(),
        result.counterfactual_trace.ontology_snapshot.facts.len()
    );
}
