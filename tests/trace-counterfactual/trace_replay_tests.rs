// tests/trace-counterfactual/trace_replay_tests.rs
//! Integration tests for trace indexing and deterministic replay
//!
//! Tests that verify:
//! - TraceId generation is deterministic
//! - Replay produces identical results
//! - Trace storage and retrieval works correctly
//! - Observable segments capture events properly

use knhk_workflow_engine::autonomic::{
    CounterfactualEngine, CounterfactualScenario, DoctrineConfig, ExecutionMode,
    ExecutionTrace, Goal, GoalType, KnowledgeBase, MonitorEvent, ObservableSegment,
    OntologySnapshot, TraceId, TraceStorage,
};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_trace_id_deterministic() {
    // Arrange: Create identical observable segments and ontology snapshots
    let o1 = ObservableSegment::new(1000, 2000);
    let o2 = ObservableSegment::new(1000, 2000);

    let sigma1 = OntologySnapshot {
        goals: Vec::new(),
        rules: Vec::new(),
        facts: HashMap::new(),
        policies: Vec::new(),
        timestamp_ms: 1500,
    };
    let sigma2 = sigma1.clone();

    let q1 = DoctrineConfig::default();
    let q2 = DoctrineConfig::default();

    // Act: Generate trace IDs
    let trace_id1 = TraceId::new(&o1, &sigma1, &q1).unwrap();
    let trace_id2 = TraceId::new(&o2, &sigma2, &q2).unwrap();

    // Assert: Trace IDs should be identical for identical inputs
    assert_eq!(trace_id1, trace_id2);
    assert_eq!(trace_id1.to_hex(), trace_id2.to_hex());
}

#[tokio::test]
async fn test_trace_id_sensitivity() {
    // Arrange: Create slightly different inputs
    let o1 = ObservableSegment::new(1000, 2000);
    let mut o2 = ObservableSegment::new(1000, 2000);
    o2.start_time_ms = 1001; // Small change

    let sigma = OntologySnapshot {
        goals: Vec::new(),
        rules: Vec::new(),
        facts: HashMap::new(),
        policies: Vec::new(),
        timestamp_ms: 1500,
    };

    let q = DoctrineConfig::default();

    // Act: Generate trace IDs
    let trace_id1 = TraceId::new(&o1, &sigma, &q).unwrap();
    let trace_id2 = TraceId::new(&o2, &sigma, &q).unwrap();

    // Assert: Different inputs should produce different trace IDs
    assert_ne!(trace_id1, trace_id2);
}

#[tokio::test]
async fn test_observable_segment_events() {
    // Arrange: Create observable segment
    let mut segment = ObservableSegment::new(1000, 5000);

    // Act: Add events
    segment.add_event(MonitorEvent::new(
        "latency".to_string(),
        100.0,
        "monitor".to_string(),
    ));
    segment.add_event(MonitorEvent::new(
        "throughput".to_string(),
        50.0,
        "monitor".to_string(),
    ));

    // Assert: Events captured correctly
    assert_eq!(segment.events.len(), 2);
    assert_eq!(segment.duration_ms(), 4000);
    assert!(segment.contains_time(3000));
    assert!(!segment.contains_time(6000));
}

#[tokio::test]
async fn test_ontology_snapshot_roundtrip() {
    // Arrange: Create knowledge base with goals, rules, facts
    let kb = KnowledgeBase::new();

    let goal = Goal::new(
        "latency_goal".to_string(),
        GoalType::Performance,
        "latency".to_string(),
        100.0,
    );
    kb.add_goal(goal).await.unwrap();

    // Act: Create snapshot and restore to new KB
    let snapshot = OntologySnapshot::from_knowledge_base(&kb).await;
    let kb2 = KnowledgeBase::new();
    snapshot.restore_to_knowledge_base(&kb2).await.unwrap();

    // Assert: Restored knowledge matches original
    let goals1 = kb.get_goals().await;
    let goals2 = kb2.get_goals().await;

    assert_eq!(goals1.len(), goals2.len());
    assert_eq!(goals1[0].name, goals2[0].name);
    assert_eq!(goals1[0].target, goals2[0].target);
}

#[tokio::test]
async fn test_trace_storage_operations() {
    // Arrange: Create trace storage
    let storage = TraceStorage::new(10);

    let o_segment = ObservableSegment::new(1000, 2000);
    let sigma = OntologySnapshot {
        goals: Vec::new(),
        rules: Vec::new(),
        facts: HashMap::new(),
        policies: Vec::new(),
        timestamp_ms: 1500,
    };
    let q = DoctrineConfig::default();

    let trace = ExecutionTrace::new(o_segment, sigma, q).unwrap();
    let trace_id = trace.id;

    // Act: Store trace
    storage.store(trace).await.unwrap();

    // Assert: Trace can be retrieved
    assert!(storage.contains(&trace_id).await);

    let retrieved = storage.retrieve(&trace_id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, trace_id);
}

#[tokio::test]
async fn test_trace_storage_lru_eviction() {
    // Arrange: Create trace storage with capacity 3
    let storage = TraceStorage::new(3);

    let mut trace_ids = Vec::new();

    // Act: Store 5 traces
    for i in 0..5 {
        let mut o_segment = ObservableSegment::new(1000 + i * 1000, 2000 + i * 1000);
        o_segment
            .metadata
            .insert("index".to_string(), i.to_string());

        let sigma = OntologySnapshot {
            goals: Vec::new(),
            rules: Vec::new(),
            facts: HashMap::new(),
            policies: Vec::new(),
            timestamp_ms: 1500 + i * 1000,
        };
        let q = DoctrineConfig::default();

        let trace = ExecutionTrace::new(o_segment, sigma, q).unwrap();
        trace_ids.push(trace.id);
        storage.store(trace).await.unwrap();
    }

    // Assert: Only last 3 traces remain (LRU eviction)
    let stats = storage.stats().await;
    assert_eq!(stats.total_traces, 3);

    // First 2 traces should be evicted
    assert!(!storage.contains(&trace_ids[0]).await);
    assert!(!storage.contains(&trace_ids[1]).await);

    // Last 3 traces should remain
    assert!(storage.contains(&trace_ids[2]).await);
    assert!(storage.contains(&trace_ids[3]).await);
    assert!(storage.contains(&trace_ids[4]).await);
}

#[tokio::test]
async fn test_deterministic_replay() {
    // Arrange: Create original trace with monitor events
    let storage = Arc::new(TraceStorage::new(10));
    let engine = CounterfactualEngine::new(storage.clone());

    let mut o_segment = ObservableSegment::new(1000, 2000);
    o_segment.add_event(MonitorEvent::new(
        "latency".to_string(),
        150.0,
        "monitor".to_string(),
    ));
    o_segment.add_event(MonitorEvent::new(
        "throughput".to_string(),
        50.0,
        "monitor".to_string(),
    ));

    let kb = KnowledgeBase::new();
    let goal = Goal::new(
        "latency_goal".to_string(),
        GoalType::Performance,
        "latency".to_string(),
        100.0,
    );
    kb.add_goal(goal).await.unwrap();

    let sigma = OntologySnapshot::from_knowledge_base(&kb).await;
    let q = DoctrineConfig::default();

    let trace = ExecutionTrace::new(o_segment, sigma, q).unwrap();
    let trace_id = trace.id;
    storage.store(trace).await.unwrap();

    // Act: Execute replay scenario
    let scenario = CounterfactualScenario::replay(trace_id);
    let result = engine.execute(scenario).await.unwrap();

    // Assert: Replay should be in replay mode
    assert_eq!(result.scenario.mode, ExecutionMode::Replay);
    assert_eq!(result.original_trace.id, trace_id);
}

#[tokio::test]
async fn test_trace_hex_encoding() {
    // Arrange: Create trace ID
    let o_segment = ObservableSegment::new(1000, 2000);
    let sigma = OntologySnapshot {
        goals: Vec::new(),
        rules: Vec::new(),
        facts: HashMap::new(),
        policies: Vec::new(),
        timestamp_ms: 1500,
    };
    let q = DoctrineConfig::default();

    let trace_id = TraceId::new(&o_segment, &sigma, &q).unwrap();

    // Act: Convert to hex and back
    let hex = trace_id.to_hex();
    let parsed = TraceId::from_hex(&hex).unwrap();

    // Assert: Round-trip should preserve ID
    assert_eq!(trace_id, parsed);
    assert_eq!(hex.len(), 64); // 32 bytes * 2 hex chars
}

#[tokio::test]
async fn test_doctrine_config() {
    // Arrange: Create different doctrine configurations
    let mut q1 = DoctrineConfig::new("1.0.0".to_string(), "strict".to_string());
    q1.set_config("max_instances".to_string(), serde_json::json!(10));
    q1.set_feature("auto_scaling".to_string(), true);

    let mut q2 = DoctrineConfig::new("1.0.0".to_string(), "relaxed".to_string());
    q2.set_config("max_instances".to_string(), serde_json::json!(20));
    q2.set_feature("auto_scaling".to_string(), false);

    let o_segment = ObservableSegment::new(1000, 2000);
    let sigma = OntologySnapshot {
        goals: Vec::new(),
        rules: Vec::new(),
        facts: HashMap::new(),
        policies: Vec::new(),
        timestamp_ms: 1500,
    };

    // Act: Generate trace IDs with different doctrines
    let trace_id1 = TraceId::new(&o_segment, &sigma, &q1).unwrap();
    let trace_id2 = TraceId::new(&o_segment, &sigma, &q2).unwrap();

    // Assert: Different doctrines produce different trace IDs
    assert_ne!(trace_id1, trace_id2);
}

#[tokio::test]
async fn test_execution_trace_records() {
    // Arrange: Create execution trace
    let o_segment = ObservableSegment::new(1000, 2000);
    let sigma = OntologySnapshot {
        goals: Vec::new(),
        rules: Vec::new(),
        facts: HashMap::new(),
        policies: Vec::new(),
        timestamp_ms: 1500,
    };
    let q = DoctrineConfig::default();

    let mut trace = ExecutionTrace::new(o_segment, sigma, q).unwrap();

    // Act: Add execution records
    use knhk_workflow_engine::autonomic::{ActionResult, ExecutionRecord};
    trace.add_execution_record(ExecutionRecord {
        timestamp_ms: 1100,
        action_type: "scale_up".to_string(),
        params: HashMap::new(),
        result: ActionResult::Success {
            details: "Scaled to 3 instances".to_string(),
        },
        duration_us: 500,
    });

    trace.add_execution_record(ExecutionRecord {
        timestamp_ms: 1200,
        action_type: "optimize".to_string(),
        params: HashMap::new(),
        result: ActionResult::Success {
            details: "Optimized query path".to_string(),
        },
        duration_us: 200,
    });

    // Assert: Execution records captured
    assert_eq!(trace.execution_results.len(), 2);
    assert_eq!(trace.execution_results[0].action_type, "scale_up");
    assert_eq!(trace.execution_results[1].action_type, "optimize");
}
