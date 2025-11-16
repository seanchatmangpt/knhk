//! Comprehensive test suite for Multiple Instance patterns (12-15)
//!
//! Tests all 4 MI patterns with:
//! - Basic functionality
//! - Synchronization correctness
//! - Work-stealing executor integration
//! - RDF instance tracking
//! - Edge cases

#![cfg(all(feature = "async-v2", feature = "rdf"))]

use knhk_workflow_engine::case::CaseId;
use knhk_workflow_engine::concurrency::WorkStealingExecutor;
use knhk_workflow_engine::patterns::mi::{
    InstanceStatus, InstanceTracker, SyncGate, SyncGateStatus,
};
use knhk_workflow_engine::patterns::multiple_instance_v2::{
    MIExecutionContext, MultipleInstanceDesignTimeV2, MultipleInstanceDynamicV2,
    MultipleInstanceRuntimeV2, MultipleInstanceWithoutSyncV2,
};
use knhk_workflow_engine::patterns::{PatternExecutionContext, PatternId};
use knhk_workflow_engine::parser::WorkflowSpecId;
use oxigraph::store::Store;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Helper to create test context
fn create_base_context(case_id: &str) -> PatternExecutionContext {
    PatternExecutionContext {
        case_id: CaseId::from(case_id),
        workflow_id: WorkflowSpecId::from("test-workflow"),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: "test-scope".to_string(),
    }
}

/// Helper to create MI execution context
fn create_mi_context(
    base: PatternExecutionContext,
    executor: Arc<WorkStealingExecutor>,
    rdf_store: Arc<RwLock<Store>>,
) -> MIExecutionContext {
    MIExecutionContext {
        base,
        executor,
        rdf_store,
    }
}

// ============================================================================
// Pattern 12: MI Without Synchronization Tests
// ============================================================================

#[tokio::test]
async fn test_pattern_12_basic_execution() {
    let executor = Arc::new(WorkStealingExecutor::new(2));
    let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));

    let mut base_ctx = create_base_context("case-p12-basic");
    base_ctx
        .variables
        .insert("instance_count".to_string(), "5".to_string());

    let mi_ctx = create_mi_context(base_ctx, executor.clone(), rdf_store.clone());

    let pattern = MultipleInstanceWithoutSyncV2;
    let result = pattern.execute_async(&mi_ctx).await.unwrap();

    assert!(result.success);
    assert_eq!(
        result.variables.get("instances_spawned").unwrap(),
        "5"
    );

    // Verify instance set in RDF
    let tracker = InstanceTracker::new(rdf_store.clone());
    let instance_set_id = result.variables.get("instance_set_id").unwrap();
    let count = tracker.get_instance_count(instance_set_id).await.unwrap();
    assert_eq!(count, 5);

    // Wait for instances to potentially complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    executor.shutdown().await;
}

#[tokio::test]
async fn test_pattern_12_no_synchronization() {
    let executor = Arc::new(WorkStealingExecutor::new(4));
    let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));

    let mut base_ctx = create_base_context("case-p12-nosync");
    base_ctx
        .variables
        .insert("instance_count".to_string(), "10".to_string());

    let mi_ctx = create_mi_context(base_ctx, executor.clone(), rdf_store.clone());

    let pattern = MultipleInstanceWithoutSyncV2;
    let result = pattern.execute_async(&mi_ctx).await.unwrap();

    // Pattern 12 returns immediately without synchronization
    assert!(result.success);
    assert!(result.next_activities.is_empty()); // No blocking on completion

    tokio::time::sleep(Duration::from_millis(50)).await;
    executor.shutdown().await;
}

// ============================================================================
// Pattern 13: MI Design-Time Knowledge Tests
// ============================================================================

#[tokio::test]
async fn test_pattern_13_synchronization() {
    let executor = Arc::new(WorkStealingExecutor::new(2));
    let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));

    let mut base_ctx = create_base_context("case-p13-sync");
    base_ctx
        .variables
        .insert("instance_count".to_string(), "3".to_string());

    let mi_ctx = create_mi_context(base_ctx, executor.clone(), rdf_store.clone());

    let pattern = MultipleInstanceDesignTimeV2;
    let result = pattern.execute_async(&mi_ctx).await.unwrap();

    assert!(result.success);

    // Verify sync gate created
    let sync_gate_id = result.variables.get("sync_gate_id").unwrap();
    let gate = SyncGate::new(rdf_store.clone());

    // Initially waiting
    let status = gate.get_status(sync_gate_id).await.unwrap();
    assert_eq!(status, SyncGateStatus::Waiting);

    // Check progress
    let (completed, target) = gate.get_progress(sync_gate_id).await.unwrap();
    assert_eq!(target, 3);

    // Wait for instances to complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Should be completed now
    let (completed_final, _) = gate.get_progress(sync_gate_id).await.unwrap();
    assert_eq!(completed_final, 3);

    let status_final = gate.get_status(sync_gate_id).await.unwrap();
    assert_eq!(status_final, SyncGateStatus::Completed);

    executor.shutdown().await;
}

#[tokio::test]
async fn test_pattern_13_all_instances_complete() {
    let executor = Arc::new(WorkStealingExecutor::new(4));
    let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));

    let mut base_ctx = create_base_context("case-p13-complete");
    base_ctx
        .variables
        .insert("instance_count".to_string(), "8".to_string());

    let mi_ctx = create_mi_context(base_ctx, executor.clone(), rdf_store.clone());

    let pattern = MultipleInstanceDesignTimeV2;
    let result = pattern.execute_async(&mi_ctx).await.unwrap();

    let sync_gate_id = result.variables.get("sync_gate_id").unwrap();
    let gate = SyncGate::new(rdf_store.clone());

    // Wait for all instances
    tokio::time::sleep(Duration::from_millis(150)).await;

    let (completed, target) = gate.get_progress(sync_gate_id).await.unwrap();
    assert_eq!(completed, target);
    assert_eq!(target, 8);

    executor.shutdown().await;
}

// ============================================================================
// Pattern 14: MI Runtime Knowledge Tests
// ============================================================================

#[tokio::test]
async fn test_pattern_14_runtime_array() {
    let executor = Arc::new(WorkStealingExecutor::new(2));
    let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));

    let mut base_ctx = create_base_context("case-p14-array");

    // Provide runtime instance data as JSON array
    let runtime_data = serde_json::json!([
        {"id": 1, "data": "item1"},
        {"id": 2, "data": "item2"},
        {"id": 3, "data": "item3"}
    ]);
    base_ctx.variables.insert(
        "runtime_instance_data".to_string(),
        runtime_data.to_string(),
    );

    let mi_ctx = create_mi_context(base_ctx, executor.clone(), rdf_store.clone());

    let pattern = MultipleInstanceRuntimeV2;
    let result = pattern.execute_async(&mi_ctx).await.unwrap();

    assert!(result.success);
    assert_eq!(
        result.variables.get("instances_spawned").unwrap(),
        "3"
    );

    // Verify instance count from runtime data
    let tracker = InstanceTracker::new(rdf_store.clone());
    let instance_set_id = result.variables.get("instance_set_id").unwrap();
    let count = tracker.get_instance_count(instance_set_id).await.unwrap();
    assert_eq!(count, 3);

    tokio::time::sleep(Duration::from_millis(100)).await;
    executor.shutdown().await;
}

#[tokio::test]
async fn test_pattern_14_large_array() {
    let executor = Arc::new(WorkStealingExecutor::new(4));
    let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));

    let mut base_ctx = create_base_context("case-p14-large");

    // Create large array (100 items)
    let runtime_data: Vec<serde_json::Value> = (0..100)
        .map(|i| serde_json::json!({"id": i, "value": i * 2}))
        .collect();

    base_ctx.variables.insert(
        "runtime_instance_data".to_string(),
        serde_json::to_string(&runtime_data).unwrap(),
    );

    let mi_ctx = create_mi_context(base_ctx, executor.clone(), rdf_store.clone());

    let pattern = MultipleInstanceRuntimeV2;
    let result = pattern.execute_async(&mi_ctx).await.unwrap();

    assert!(result.success);
    assert_eq!(
        result.variables.get("instances_spawned").unwrap(),
        "100"
    );

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify all completed
    let sync_gate_id = result.variables.get("sync_gate_id").unwrap();
    let gate = SyncGate::new(rdf_store.clone());
    let (completed, target) = gate.get_progress(sync_gate_id).await.unwrap();
    assert_eq!(completed, target);
    assert_eq!(target, 100);

    executor.shutdown().await;
}

// ============================================================================
// Pattern 15: MI Dynamic Tests
// ============================================================================

#[tokio::test]
async fn test_pattern_15_initial_instances() {
    let executor = Arc::new(WorkStealingExecutor::new(2));
    let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));

    let mut base_ctx = create_base_context("case-p15-initial");
    base_ctx
        .variables
        .insert("initial_instance_count".to_string(), "2".to_string());
    base_ctx
        .variables
        .insert("allow_dynamic_spawning".to_string(), "true".to_string());

    let mi_ctx = create_mi_context(base_ctx, executor.clone(), rdf_store.clone());

    let pattern = MultipleInstanceDynamicV2;
    let result = pattern.execute_async(&mi_ctx).await.unwrap();

    assert!(result.success);
    assert_eq!(
        result.variables.get("instances_spawned").unwrap(),
        "2"
    );
    assert_eq!(
        result.variables.get("allow_dynamic_spawning").unwrap(),
        "true"
    );

    tokio::time::sleep(Duration::from_millis(100)).await;
    executor.shutdown().await;
}

#[tokio::test]
async fn test_pattern_15_zero_initial() {
    let executor = Arc::new(WorkStealingExecutor::new(2));
    let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));

    let mut base_ctx = create_base_context("case-p15-zero");
    base_ctx
        .variables
        .insert("initial_instance_count".to_string(), "0".to_string());

    let mi_ctx = create_mi_context(base_ctx, executor.clone(), rdf_store.clone());

    let pattern = MultipleInstanceDynamicV2;
    let result = pattern.execute_async(&mi_ctx).await.unwrap();

    assert!(result.success);
    assert_eq!(
        result.variables.get("instances_spawned").unwrap(),
        "0"
    );

    executor.shutdown().await;
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[tokio::test]
async fn test_single_instance() {
    let executor = Arc::new(WorkStealingExecutor::new(1));
    let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));

    let mut base_ctx = create_base_context("case-single");
    base_ctx
        .variables
        .insert("instance_count".to_string(), "1".to_string());

    let mi_ctx = create_mi_context(base_ctx, executor.clone(), rdf_store.clone());

    let pattern = MultipleInstanceDesignTimeV2;
    let result = pattern.execute_async(&mi_ctx).await.unwrap();

    assert!(result.success);
    assert_eq!(
        result.variables.get("instances_spawned").unwrap(),
        "1"
    );

    tokio::time::sleep(Duration::from_millis(50)).await;

    let sync_gate_id = result.variables.get("sync_gate_id").unwrap();
    let gate = SyncGate::new(rdf_store.clone());
    let status = gate.get_status(sync_gate_id).await.unwrap();
    assert_eq!(status, SyncGateStatus::Completed);

    executor.shutdown().await;
}

#[tokio::test]
async fn test_executor_metrics() {
    let executor = Arc::new(WorkStealingExecutor::new(4));
    let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));

    let mut base_ctx = create_base_context("case-metrics");
    base_ctx
        .variables
        .insert("instance_count".to_string(), "50".to_string());

    let mi_ctx = create_mi_context(base_ctx, executor.clone(), rdf_store.clone());

    let pattern = MultipleInstanceWithoutSyncV2;
    let _result = pattern.execute_async(&mi_ctx).await.unwrap();

    tokio::time::sleep(Duration::from_millis(150)).await;

    let metrics = executor.metrics();
    assert!(metrics.tasks_spawned.load(std::sync::atomic::Ordering::Relaxed) >= 50);

    executor.shutdown().await;
}

// ============================================================================
// Performance and Concurrency Tests
// ============================================================================

#[tokio::test]
async fn test_high_concurrency() {
    let executor = Arc::new(WorkStealingExecutor::new(8));
    let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));

    let mut base_ctx = create_base_context("case-concurrent");
    base_ctx
        .variables
        .insert("instance_count".to_string(), "1000".to_string());

    let mi_ctx = create_mi_context(base_ctx, executor.clone(), rdf_store.clone());

    let pattern = MultipleInstanceWithoutSyncV2;
    let result = pattern.execute_async(&mi_ctx).await.unwrap();

    assert!(result.success);
    assert_eq!(
        result.variables.get("instances_spawned").unwrap(),
        "1000"
    );

    tokio::time::sleep(Duration::from_millis(500)).await;

    let metrics = executor.metrics();
    let spawned = metrics.tasks_spawned.load(std::sync::atomic::Ordering::Relaxed);
    let completed = metrics
        .tasks_completed
        .load(std::sync::atomic::Ordering::Relaxed);

    assert!(spawned >= 1000);
    assert!(completed >= 900); // Most should complete

    executor.shutdown().await;
}
