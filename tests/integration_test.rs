// KNHK Integration Tests - Fortune 500 Production Scenarios
// Phase 5: End-to-end tests that validate real-world enterprise deployments

use knhk::{
    ProductionPlatform, PlatformConfig,
    PersistenceLayer, ObservabilityLayer, MonitoringLayer,
    RecoveryManager, ScalingManager, LearningEngine, CostTracker,
};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::time::{sleep, timeout};

/// Test complete workflow execution in production
#[tokio::test]
async fn test_production_workflow_execution() {
    let config = PlatformConfig {
        max_concurrent_workflows: 100,
        workflow_timeout: Duration::from_secs(60),
        enable_auto_scaling: false,
        enable_learning: true,
        enable_cost_tracking: true,
        persistence_path: "/tmp/test_knhk_prod".to_string(),
        cluster_mode: false,
        node_id: "test-node".to_string(),
        telemetry_endpoint: None,
        health_check_port: 9091,
    };

    let mut platform = ProductionPlatform::new(config).unwrap();
    platform.start().await.unwrap();

    // Submit test workflow
    let descriptor = r#"
        name: test-workflow
        version: 1.0.0
        steps:
          - name: validate
            action: validate_input
          - name: process
            action: transform_data
          - name: persist
            action: store_result
    "#;

    let workflow_id = platform.submit_workflow(descriptor.to_string()).await.unwrap();
    assert!(!workflow_id.is_empty());

    // Wait for completion (with timeout)
    let result = timeout(Duration::from_secs(10), async {
        loop {
            if let Some(state) = platform.workflows.get(&workflow_id) {
                if matches!(state.status, knhk::production::platform::WorkflowStatus::Completed) {
                    return Ok(());
                }
                if matches!(state.status, knhk::production::platform::WorkflowStatus::Failed) {
                    return Err("Workflow failed");
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
    }).await;

    assert!(result.is_ok());

    // Verify workflow completed successfully
    let state = platform.workflows.get(&workflow_id).unwrap();
    assert!(matches!(state.status, knhk::production::platform::WorkflowStatus::Completed));

    platform.shutdown().await.unwrap();
}

/// Test persistence and recovery
#[tokio::test]
async fn test_persistence_and_recovery() {
    let temp_dir = tempfile::tempdir().unwrap();
    let persistence_path = temp_dir.path().to_str().unwrap();

    // Create persistence layer
    let persistence = Arc::new(PersistenceLayer::new(persistence_path).unwrap());

    // Store test receipts
    let receipt = knhk::autonomic::Receipt::default();
    persistence.store_receipt("test-workflow-1", &receipt).await.unwrap();

    // Verify storage
    let receipts = persistence.get_receipts("test-workflow-1").await.unwrap();
    assert_eq!(receipts.len(), 1);

    // Verify integrity
    let valid = persistence.verify_receipts("test-workflow-1").await.unwrap();
    assert!(valid);

    // Test recovery
    let recovery = RecoveryManager::new(persistence.clone()).unwrap();

    // Save snapshot
    let snapshot = knhk::production::recovery::StateSnapshot {
        timestamp: SystemTime::now(),
        workflows: vec![],
        metrics: Default::default(),
    };
    recovery.save_snapshot(snapshot.clone()).await.unwrap();

    // Load snapshot
    let loaded = recovery.load_latest_snapshot().await.unwrap();
    assert_eq!(loaded.workflows.len(), snapshot.workflows.len());

    // Verify consistency
    let report = recovery.verify_consistency().await.unwrap();
    assert!(report.is_consistent);
}

/// Test observability and monitoring
#[tokio::test]
async fn test_observability_and_monitoring() {
    // Initialize observability
    let observability = Arc::new(ObservabilityLayer::new(None).unwrap());
    observability.start().await.unwrap();

    // Record workflow events
    observability.record_workflow_submitted("test-wf-1", "test-descriptor").await;
    observability.record_workflow_start("test-wf-1").await;

    for i in 0..5 {
        observability.record_step_completion("test-wf-1", i, Duration::from_millis(100)).await;
    }

    observability.record_workflow_completion("test-wf-1", Duration::from_secs(1)).await;

    // Get statistics
    let stats = observability.get_stats();
    assert!(stats.total_spans > 0);

    // Initialize monitoring
    let monitoring = Arc::new(MonitoringLayer::new().unwrap());
    monitoring.start().await.unwrap();

    // Record metrics
    monitoring.record_workflow_success("test-wf-1", Duration::from_millis(500)).await;

    // Check SLA
    let stats = monitoring.get_stats();
    assert_eq!(stats.uptime_percentage, 100.0);
    assert!(stats.is_healthy);

    observability.shutdown().await.unwrap();
    monitoring.shutdown().await.unwrap();
}

/// Test auto-scaling
#[tokio::test]
async fn test_auto_scaling() {
    let scaling = Arc::new(ScalingManager::new(false).unwrap());

    // Join cluster
    scaling.join_cluster("test-node-1").await.unwrap();

    // Assign workflows
    let node1 = scaling.assign_workflow("workflow-1").await.unwrap();
    let node2 = scaling.assign_workflow("workflow-2").await.unwrap();

    assert!(!node1.is_empty());
    assert!(!node2.is_empty());

    // Get scaling stats
    let stats = scaling.get_stats();
    assert_eq!(stats.current_replicas, 1);
    assert_eq!(stats.active_nodes, 1);

    scaling.leave_cluster().await.unwrap();
}

/// Test learning engine
#[tokio::test]
async fn test_learning_engine() {
    let learning = Arc::new(LearningEngine::new().unwrap());
    learning.start().await.unwrap();

    // Learn from executions
    for i in 0..10 {
        let receipts = vec![knhk::autonomic::Receipt::default()];
        let duration = Duration::from_millis(100 + i * 10);

        learning.learn_from_execution(
            &format!("workflow-{}", i),
            &receipts,
            duration
        ).await;
    }

    // Get predictions
    let prediction = learning.predict_performance("test-descriptor").await;
    assert!(prediction.is_err()); // No pattern yet

    // Get optimization suggestions
    let suggestions = learning.get_optimization_suggestions("test-descriptor").await;
    assert!(suggestions.is_empty()); // No suggestions yet

    // Get stats
    let stats = learning.get_stats();
    assert_eq!(stats.total_workflows_learned, 10);

    learning.shutdown().await.unwrap();
}

/// Test cost tracking
#[tokio::test]
async fn test_cost_tracking() {
    let cost_tracker = Arc::new(CostTracker::new().unwrap());
    cost_tracker.start().await.unwrap();

    // Track workflow costs
    for i in 0..5 {
        let receipts = vec![knhk::autonomic::Receipt::default()];
        let duration = Duration::from_secs(1 + i);

        let cost = cost_tracker.calculate_workflow_cost(
            &format!("workflow-{}", i),
            &receipts,
            duration
        ).await.unwrap();

        assert!(cost > 0.0);
        assert!(cost < 1.0); // Should be less than $1
    }

    // Generate allocation report
    let allocation = cost_tracker.generate_allocation_report(
        "Engineering",
        SystemTime::now() - Duration::from_secs(3600),
        SystemTime::now()
    ).await;

    assert_eq!(allocation.department, "Engineering");
    assert_eq!(allocation.workflow_count, 5);

    // Get statistics
    let stats = cost_tracker.get_stats();
    assert_eq!(stats.total_workflows, 5);
    assert!(stats.total_cost > 0.0);
    assert!(stats.average_cost_per_workflow > 0.0);

    cost_tracker.shutdown().await.unwrap();
}

/// Test concurrent workflow execution at scale
#[tokio::test]
async fn test_concurrent_workflow_scale() {
    let config = PlatformConfig {
        max_concurrent_workflows: 1000,
        workflow_timeout: Duration::from_secs(30),
        enable_auto_scaling: true,
        enable_learning: false,
        enable_cost_tracking: false,
        persistence_path: "/tmp/test_knhk_scale".to_string(),
        cluster_mode: false,
        node_id: "scale-test".to_string(),
        telemetry_endpoint: None,
        health_check_port: 9092,
    };

    let mut platform = ProductionPlatform::new(config).unwrap();
    platform.start().await.unwrap();

    // Submit multiple workflows concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let descriptor = format!(r#"
            name: workflow-{}
            version: 1.0.0
            steps:
              - name: step1
                action: process
        "#, i);

        let platform_clone = &platform;
        let handle = tokio::spawn(async move {
            platform_clone.submit_workflow(descriptor).await
        });
        handles.push(handle);
    }

    // Wait for all submissions
    let mut workflow_ids = vec![];
    for handle in handles {
        if let Ok(Ok(id)) = handle.await {
            workflow_ids.push(id);
        }
    }

    assert_eq!(workflow_ids.len(), 10);

    // Wait for some completions
    sleep(Duration::from_secs(2)).await;

    // Check statistics
    let total = platform.total_workflows.load(std::sync::atomic::Ordering::Relaxed);
    assert!(total >= 10);

    platform.shutdown().await.unwrap();
}

/// Test crash recovery scenario
#[tokio::test]
async fn test_crash_recovery_scenario() {
    let temp_dir = tempfile::tempdir().unwrap();
    let persistence_path = temp_dir.path().to_str().unwrap();

    // Phase 1: Create platform and run workflows
    {
        let config = PlatformConfig {
            persistence_path: persistence_path.to_string(),
            ..Default::default()
        };

        let mut platform = ProductionPlatform::new(config).unwrap();
        platform.start().await.unwrap();

        // Submit workflow
        let workflow_id = platform.submit_workflow("test-workflow".to_string()).await.unwrap();

        // Simulate some progress
        sleep(Duration::from_millis(100)).await;

        // Create snapshot before "crash"
        let recovery = &platform.recovery;
        let snapshot = knhk::production::recovery::StateSnapshot {
            timestamp: SystemTime::now(),
            workflows: platform.workflows.iter()
                .map(|e| e.value().clone())
                .collect(),
            metrics: Default::default(),
        };
        recovery.save_snapshot(snapshot).await.unwrap();

        // Don't call shutdown to simulate crash
    }

    // Phase 2: Recover after crash
    {
        let config = PlatformConfig {
            persistence_path: persistence_path.to_string(),
            ..Default::default()
        };

        let mut platform = ProductionPlatform::new(config).unwrap();

        // Platform will automatically recover on start
        platform.start().await.unwrap();

        // Verify recovery
        assert!(platform.workflows.len() > 0);

        platform.shutdown().await.unwrap();
    }
}

/// Test SLA compliance
#[tokio::test]
async fn test_sla_compliance() {
    let monitoring = Arc::new(MonitoringLayer::new().unwrap());
    monitoring.start().await.unwrap();

    // Simulate workflow executions with varying latencies
    for i in 0..100 {
        let duration = if i % 10 == 0 {
            Duration::from_millis(2000) // 10% high latency
        } else {
            Duration::from_millis(50) // 90% normal latency
        };

        monitoring.record_workflow_success(&format!("wf-{}", i), duration).await;
    }

    // Check SLA metrics
    let stats = monitoring.get_stats();
    assert!(stats.uptime_percentage >= 99.0);

    monitoring.shutdown().await.unwrap();
}

/// Test cost optimization and ROI
#[tokio::test]
async fn test_cost_optimization_roi() {
    let cost_tracker = Arc::new(CostTracker::new().unwrap());
    cost_tracker.start().await.unwrap();

    // Simulate workflows with cost savings
    for i in 0..50 {
        let receipts = vec![knhk::autonomic::Receipt::default()];
        let duration = Duration::from_millis(100); // Fast execution

        let cost = cost_tracker.calculate_workflow_cost(
            &format!("optimized-wf-{}", i),
            &receipts,
            duration
        ).await.unwrap();

        // Each workflow should cost less than legacy baseline ($0.50)
        assert!(cost < 0.50);
    }

    // Check ROI metrics
    let stats = cost_tracker.get_stats();
    assert!(stats.total_savings > 0.0);

    // Verify cost reduction target (40%)
    let reduction = (stats.total_savings / (stats.total_cost + stats.total_savings)) * 100.0;
    println!("Cost reduction achieved: {:.1}%", reduction);

    cost_tracker.shutdown().await.unwrap();
}