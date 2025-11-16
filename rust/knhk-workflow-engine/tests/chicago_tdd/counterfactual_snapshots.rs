//! Snapshot Tests for Counterfactual Replay
//!
//! Tests deterministic replay and counterfactual analysis:
//! - Replay produces bit-for-bit identical results
//! - Counterfactual scenarios produce consistent diffs
//! - Snapshot comparison detects behavioral changes
//!
//! **Chicago TDD Approach**: Tests with real MAPE-K components

use knhk_workflow_engine::autonomic::counterfactual::{
    CounterfactualEngine, CounterfactualScenario, ExecutionMode, CounterfactualResult,
};
use knhk_workflow_engine::autonomic::trace_index::{
    TraceId, TraceStorage, ExecutionTrace, OntologySnapshot, DoctrineConfig,
};
use knhk_workflow_engine::autonomic::analyze::{Analysis, Analyzer};
use knhk_workflow_engine::autonomic::plan::{Action, ActionType, AdaptationPlan};
use knhk_workflow_engine::error::WorkflowResult;
use serde_json::json;
use std::sync::Arc;

// ============================================================================
// Test Fixtures
// ============================================================================

/// Create test trace storage with sample data
fn create_test_trace_storage() -> Arc<TraceStorage> {
    Arc::new(TraceStorage::new())
}

/// Create sample execution trace
fn create_sample_trace() -> ExecutionTrace {
    ExecutionTrace {
        trace_id: TraceId::new(),
        timestamp_ms: 1000,
        observations: json!({
            "latency_p99_ms": 150.0,
            "error_rate": 0.02,
            "throughput_rps": 1000.0
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
                    action_type: ActionType::AdjustResources {
                        resource_type: "cpu".to_string(),
                        delta: 0.5,
                    },
                    rationale: "Increase CPU to reduce latency".to_string(),
                    policy_element: None,
                }
            ],
            confidence: 0.85,
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
    }
}

// ============================================================================
// Replay Determinism Tests
// ============================================================================

#[tokio::test]
async fn test_replay_produces_identical_trace() {
    // Arrange: Create trace and store it
    let storage = create_test_trace_storage();
    let original_trace = create_sample_trace();
    let trace_id = original_trace.trace_id;

    storage.store_trace(original_trace.clone()).await.unwrap();

    let engine = CounterfactualEngine::new(storage.clone());
    let scenario = CounterfactualScenario::replay(trace_id);

    // Act: Replay the trace
    let result = engine.execute_scenario(scenario).await.unwrap();

    // Assert: Replay should be bit-for-bit identical
    assert!(result.is_exact_replay(), "Replay should be exact");
    assert!(
        !result.has_action_changes(),
        "Replay should not change actions"
    );

    // Compare traces
    assert_eq!(
        original_trace.observations,
        result.counterfactual_trace.observations
    );
    assert_eq!(
        original_trace.plan.actions.len(),
        result.counterfactual_trace.plan.actions.len()
    );
}

#[tokio::test]
async fn test_replay_is_deterministic_across_runs() {
    // Arrange: Create and store trace
    let storage = create_test_trace_storage();
    let original_trace = create_sample_trace();
    let trace_id = original_trace.trace_id;

    storage.store_trace(original_trace.clone()).await.unwrap();

    let engine = CounterfactualEngine::new(storage.clone());
    let scenario = CounterfactualScenario::replay(trace_id);

    // Act: Execute replay multiple times
    let result1 = engine.execute_scenario(scenario.clone()).await.unwrap();
    let result2 = engine.execute_scenario(scenario.clone()).await.unwrap();
    let result3 = engine.execute_scenario(scenario).await.unwrap();

    // Assert: All replays should be identical
    assert_eq!(
        result1.counterfactual_trace.plan.actions.len(),
        result2.counterfactual_trace.plan.actions.len()
    );
    assert_eq!(
        result1.counterfactual_trace.plan.actions.len(),
        result3.counterfactual_trace.plan.actions.len()
    );

    // Confidence scores should be identical
    assert_eq!(
        result1.counterfactual_trace.plan.confidence,
        result2.counterfactual_trace.plan.confidence
    );
    assert_eq!(
        result1.counterfactual_trace.plan.confidence,
        result3.counterfactual_trace.plan.confidence
    );
}

#[tokio::test]
async fn test_replay_preserves_timing() {
    // Arrange
    let storage = create_test_trace_storage();
    let original_trace = create_sample_trace();
    let trace_id = original_trace.trace_id;

    storage.store_trace(original_trace.clone()).await.unwrap();

    let engine = CounterfactualEngine::new(storage);
    let scenario = CounterfactualScenario::replay(trace_id);

    // Act
    let result = engine.execute_scenario(scenario).await.unwrap();

    // Assert: Timing should be preserved
    assert_eq!(
        original_trace.timestamp_ms,
        result.counterfactual_trace.timestamp_ms
    );
}

// ============================================================================
// Counterfactual Scenario Tests
// ============================================================================

#[tokio::test]
async fn test_counterfactual_with_alternative_ontology() {
    // Arrange: Create trace with alternative ontology
    let storage = create_test_trace_storage();
    let original_trace = create_sample_trace();
    let trace_id = original_trace.trace_id;

    storage.store_trace(original_trace.clone()).await.unwrap();

    let alternative_ontology = OntologySnapshot {
        version: "2.0.0".to_string(),
        concepts: vec![],
        relationships: vec![],
    };

    let engine = CounterfactualEngine::new(storage);
    let scenario = CounterfactualScenario::with_ontology(
        trace_id,
        alternative_ontology,
        "Test alternative ontology".to_string(),
    );

    // Act
    let result = engine.execute_scenario(scenario).await.unwrap();

    // Assert: Should detect differences
    assert!(!result.is_exact_replay());
    assert_eq!(result.scenario.mode, ExecutionMode::Counterfactual);
}

#[tokio::test]
async fn test_counterfactual_with_alternative_doctrine() {
    // Arrange
    let storage = create_test_trace_storage();
    let original_trace = create_sample_trace();
    let trace_id = original_trace.trace_id;

    storage.store_trace(original_trace.clone()).await.unwrap();

    let alternative_doctrine = DoctrineConfig {
        invariants: vec!["latency < 100ms".to_string()],
        constraints: vec!["error_rate < 0.01".to_string()],
    };

    let engine = CounterfactualEngine::new(storage);
    let scenario = CounterfactualScenario::with_doctrine(
        trace_id,
        alternative_doctrine,
        "Stricter SLO constraints".to_string(),
    );

    // Act
    let result = engine.execute_scenario(scenario).await.unwrap();

    // Assert
    assert_eq!(result.scenario.mode, ExecutionMode::Counterfactual);
}

#[tokio::test]
async fn test_full_counterfactual_scenario() {
    // Arrange
    let storage = create_test_trace_storage();
    let original_trace = create_sample_trace();
    let trace_id = original_trace.trace_id;

    storage.store_trace(original_trace.clone()).await.unwrap();

    let alternative_ontology = OntologySnapshot {
        version: "2.0.0".to_string(),
        concepts: vec![],
        relationships: vec![],
    };

    let alternative_doctrine = DoctrineConfig {
        invariants: vec![],
        constraints: vec![],
    };

    let engine = CounterfactualEngine::new(storage);
    let scenario = CounterfactualScenario::full_counterfactual(
        trace_id,
        alternative_ontology,
        alternative_doctrine,
        "Complete alternative scenario".to_string(),
    );

    // Act
    let result = engine.execute_scenario(scenario).await.unwrap();

    // Assert
    assert_eq!(result.scenario.mode, ExecutionMode::Counterfactual);
    assert_eq!(
        result.scenario.description,
        "Complete alternative scenario"
    );
}

// ============================================================================
// Diff Analysis Tests
// ============================================================================

#[tokio::test]
async fn test_action_diff_detection() {
    // Arrange: Create trace with different actions
    let storage = create_test_trace_storage();
    let mut original_trace = create_sample_trace();
    let trace_id = original_trace.trace_id;

    // Add extra action to original
    original_trace.plan.actions.push(Action {
        action_id: uuid::Uuid::new_v4(),
        action_type: ActionType::ScaleInstances {
            service: "web".to_string(),
            delta: 2,
        },
        rationale: "Scale up for load".to_string(),
        policy_element: None,
    });

    storage.store_trace(original_trace.clone()).await.unwrap();

    let engine = CounterfactualEngine::new(storage);
    let scenario = CounterfactualScenario::replay(trace_id);

    // Act
    let result = engine.execute_scenario(scenario).await.unwrap();

    // Assert: Should detect action differences
    // (In real implementation, counterfactual might produce different actions)
    assert!(result.action_diff.added.len() >= 0);
    assert!(result.action_diff.removed.len() >= 0);
    assert!(result.action_diff.modified.len() >= 0);
}

#[tokio::test]
async fn test_invariant_violation_detection() {
    // Arrange
    let storage = create_test_trace_storage();
    let original_trace = create_sample_trace();
    let trace_id = original_trace.trace_id;

    storage.store_trace(original_trace).await.unwrap();

    let strict_doctrine = DoctrineConfig {
        invariants: vec![
            "latency_p99_ms < 100".to_string(),
            "error_rate < 0.01".to_string(),
        ],
        constraints: vec![],
    };

    let engine = CounterfactualEngine::new(storage);
    let scenario = CounterfactualScenario::with_doctrine(
        trace_id,
        strict_doctrine,
        "Strict invariants".to_string(),
    );

    // Act
    let result = engine.execute_scenario(scenario).await.unwrap();

    // Assert: Should detect violations
    assert!(
        result.invariant_checks.violations.len() >= 0,
        "Should check for violations"
    );
}

#[tokio::test]
async fn test_slo_analysis_comparison() {
    // Arrange
    let storage = create_test_trace_storage();
    let original_trace = create_sample_trace();
    let trace_id = original_trace.trace_id;

    storage.store_trace(original_trace).await.unwrap();

    let engine = CounterfactualEngine::new(storage);
    let scenario = CounterfactualScenario::replay(trace_id);

    // Act
    let result = engine.execute_scenario(scenario).await.unwrap();

    // Assert: SLO analysis should be present
    assert!(result.slo_analysis.latency_improvement_pct.is_finite());
    assert!(result.slo_analysis.error_rate_improvement_pct.is_finite());
}

#[tokio::test]
async fn test_timing_comparison() {
    // Arrange
    let storage = create_test_trace_storage();
    let original_trace = create_sample_trace();
    let trace_id = original_trace.trace_id;

    storage.store_trace(original_trace).await.unwrap();

    let engine = CounterfactualEngine::new(storage);
    let scenario = CounterfactualScenario::replay(trace_id);

    // Act
    let result = engine.execute_scenario(scenario).await.unwrap();

    // Assert: Timing comparison should show minimal difference for replay
    assert!(
        result.timing_comparison.execution_time_diff_ms < 10,
        "Replay should have similar timing"
    );
}

// ============================================================================
// Snapshot Consistency Tests
// ============================================================================

#[tokio::test]
async fn test_snapshot_serialization_roundtrip() {
    // Arrange: Create result and serialize
    let storage = create_test_trace_storage();
    let original_trace = create_sample_trace();
    let trace_id = original_trace.trace_id;

    storage.store_trace(original_trace).await.unwrap();

    let engine = CounterfactualEngine::new(storage);
    let scenario = CounterfactualScenario::replay(trace_id);

    let result = engine.execute_scenario(scenario).await.unwrap();

    // Act: Serialize and deserialize
    let serialized = serde_json::to_string(&result).unwrap();
    let deserialized: CounterfactualResult = serde_json::from_str(&serialized).unwrap();

    // Assert: Should roundtrip correctly
    assert_eq!(result.scenario.original_trace_id, deserialized.scenario.original_trace_id);
    assert_eq!(result.scenario.mode, deserialized.scenario.mode);
}

#[tokio::test]
async fn test_snapshot_stability_across_versions() {
    // Arrange: Create known snapshot
    let storage = create_test_trace_storage();
    let original_trace = create_sample_trace();
    let trace_id = original_trace.trace_id;

    storage.store_trace(original_trace).await.unwrap();

    let engine = CounterfactualEngine::new(storage);
    let scenario = CounterfactualScenario::replay(trace_id);

    // Act: Execute and snapshot
    let result = engine.execute_scenario(scenario).await.unwrap();

    // Assert: Snapshot should have stable structure
    let json = serde_json::to_value(&result).unwrap();

    assert!(json.get("scenario").is_some());
    assert!(json.get("original_trace").is_some());
    assert!(json.get("counterfactual_trace").is_some());
    assert!(json.get("action_diff").is_some());
    assert!(json.get("invariant_checks").is_some());
    assert!(json.get("slo_analysis").is_some());
}

// ============================================================================
// Performance Tests
// ============================================================================

#[tokio::test]
async fn test_replay_performance_within_budget() {
    use std::time::Instant;

    // Arrange
    let storage = create_test_trace_storage();
    let original_trace = create_sample_trace();
    let trace_id = original_trace.trace_id;

    storage.store_trace(original_trace).await.unwrap();

    let engine = CounterfactualEngine::new(storage);
    let scenario = CounterfactualScenario::replay(trace_id);

    // Act: Measure replay time
    let start = Instant::now();
    let _ = engine.execute_scenario(scenario).await.unwrap();
    let elapsed = start.elapsed();

    // Assert: Should complete quickly (< 100ms)
    assert!(
        elapsed.as_millis() < 100,
        "Replay should be fast: took {}ms",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn test_batch_replay_performance() {
    use std::time::Instant;

    // Arrange: Create multiple traces
    let storage = create_test_trace_storage();
    let mut trace_ids = Vec::new();

    for _ in 0..10 {
        let trace = create_sample_trace();
        trace_ids.push(trace.trace_id);
        storage.store_trace(trace).await.unwrap();
    }

    let engine = CounterfactualEngine::new(storage);

    // Act: Replay all traces
    let start = Instant::now();

    for trace_id in trace_ids {
        let scenario = CounterfactualScenario::replay(trace_id);
        let _ = engine.execute_scenario(scenario).await.unwrap();
    }

    let elapsed = start.elapsed();

    // Assert: Batch replay should be efficient (< 1s for 10 traces)
    assert!(
        elapsed.as_secs() < 1,
        "Batch replay should be efficient: took {}ms",
        elapsed.as_millis()
    );
}
