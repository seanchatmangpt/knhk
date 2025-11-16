//! # Integration Tests for MAPE-K Autonomic System
//!
//! **Covenant 3**: Feedback Loops Run at Machine Speed
//!
//! These tests verify the complete MAPE-K feedback loop behavior,
//! including detection, analysis, planning, execution, and learning.

use knhk_autonomic::{
    AutonomicController, Config,
    types::{Action, ActionType, RiskLevel, MetricType, RuleType},
};
use std::time::Duration;
use tempfile::tempdir;
use uuid::Uuid;

/// Test complete MAPE-K cycle with failure injection and recovery
#[tokio::test]
async fn test_complete_mape_k_cycle() {
    // Create temporary knowledge database
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_knowledge.db");

    let config = Config::default()
        .with_loop_frequency(Duration::from_millis(100))
        .with_knowledge_path(db_path.to_str().unwrap());

    let mut controller = AutonomicController::new(config).await.unwrap();

    // === SETUP MONITORING ===
    {
        let mut monitor = controller.monitor().write().await;
        monitor.register_metric(
            "Error Count",
            MetricType::Reliability,
            0.0,
            5.0,
            "count",
        ).await.unwrap();
    }

    // === SETUP ANALYSIS ===
    {
        let mut analyzer = controller.analyzer().write().await;
        analyzer.register_rule(
            "High Error Rate",
            RuleType::HighErrorRate,
            ""
        ).await.unwrap();
    }

    // === SETUP PLANNING ===
    let action_id = {
        let mut planner = controller.planner().write().await;

        let action = Action {
            id: Uuid::new_v4(),
            action_type: ActionType::Heal,
            description: "Retry operation".to_string(),
            target: "test_task".to_string(),
            implementation: "retry_handler".to_string(),
            estimated_impact: "Recovers 70% of failures".to_string(),
            risk_level: RiskLevel::LowRisk,
        };

        let action_id = action.id;
        planner.register_action(action).await.unwrap();

        planner.register_policy(
            "Retry on Failure",
            "HighErrorRate",
            vec![action_id],
            100,
        ).await.unwrap();

        action_id
    };

    // Start controller in background
    let controller_handle = tokio::spawn({
        let mut controller_clone = controller.clone();
        async move {
            controller_clone.start().await
        }
    });

    // Wait for initialization
    tokio::time::sleep(Duration::from_millis(50)).await;

    // === INJECT FAILURE ===
    {
        let mut monitor = controller.monitor().write().await;
        monitor.update_metric("Error Count", 10.0).await.unwrap();
    }

    // Wait for MAPE-K cycles to process
    tokio::time::sleep(Duration::from_millis(300)).await;

    // === VERIFY RESULTS ===
    {
        let kb = controller.knowledge().read().await;

        // Check cycles were executed
        let cycles = kb.get_cycles().await.unwrap();
        assert!(!cycles.is_empty(), "Expected MAPE-K cycles to be recorded");

        // Check patterns were learned
        let patterns = kb.get_patterns().await.unwrap();
        assert!(!patterns.is_empty(), "Expected patterns to be learned");

        // Check success rate was tracked
        let rate = kb.get_success_rate(&action_id).await.unwrap();
        assert!(rate > 0.0, "Expected success rate to be tracked");
    }

    // Stop controller
    controller.stop().await;
    controller_handle.abort();
}

/// Test monitor component metrics and anomaly detection
#[tokio::test]
async fn test_monitor_component() {
    use knhk_autonomic::monitor::MonitoringComponent;

    let mut monitor = MonitoringComponent::new();

    // Register metric
    monitor.register_metric(
        "Test Metric",
        MetricType::Performance,
        100.0,
        150.0,
        "ms",
    ).await.unwrap();

    // Update with normal value
    monitor.update_metric("Test Metric", 120.0).await.unwrap();

    let metrics = monitor.collect_metrics().await.unwrap();
    assert_eq!(metrics.len(), 1);
    assert!(!metrics[0].is_anomalous);

    // Update with anomalous value
    monitor.update_metric("Test Metric", 200.0).await.unwrap();

    let metrics = monitor.collect_metrics().await.unwrap();
    assert!(metrics[0].is_anomalous);

    let anomalies = monitor.detect_anomalies(&metrics).await.unwrap();
    assert_eq!(anomalies.len(), 1);
}

/// Test analyze component rule matching
#[tokio::test]
async fn test_analyze_component() {
    use knhk_autonomic::analyze::AnalysisComponent;
    use knhk_autonomic::types::{Metric, TrendDirection};
    use chrono::Utc;

    let mut analyzer = AnalysisComponent::new();

    analyzer.register_rule(
        "High Error Rate",
        RuleType::HighErrorRate,
        ""
    ).await.unwrap();

    let metrics = vec![
        Metric {
            id: Uuid::new_v4(),
            name: "Error Count".to_string(),
            metric_type: MetricType::Reliability,
            current_value: 10.0,
            expected_value: 1.0,
            unit: "count".to_string(),
            anomaly_threshold: 5.0,
            is_anomalous: true,
            trend: TrendDirection::Degrading,
            timestamp: Utc::now(),
        }
    ];

    let analyses = analyzer.analyze(&[], &metrics).await.unwrap();
    assert_eq!(analyses.len(), 1);
    assert_eq!(analyses[0].rule_type, RuleType::HighErrorRate);
}

/// Test planner component policy evaluation
#[tokio::test]
async fn test_planner_component() {
    use knhk_autonomic::planner::PlanningComponent;
    use knhk_autonomic::types::Analysis;
    use chrono::Utc;
    use std::collections::HashMap;

    let mut planner = PlanningComponent::new();

    // Setup action
    let action = Action {
        id: Uuid::new_v4(),
        action_type: ActionType::Heal,
        description: "Test action".to_string(),
        target: "test".to_string(),
        implementation: "test_handler".to_string(),
        estimated_impact: "Test".to_string(),
        risk_level: RiskLevel::LowRisk,
    };

    let action_id = action.id;
    planner.register_action(action).await.unwrap();

    // Setup policy
    planner.register_policy(
        "Test Policy",
        "HighErrorRate",
        vec![action_id],
        100,
    ).await.unwrap();

    // Create analysis
    let analysis = Analysis {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        problem: "High error rate".to_string(),
        root_cause: "Test".to_string(),
        affected_elements: vec![],
        recommended_actions: vec![],
        confidence: 0.9,
        rule_type: RuleType::HighErrorRate,
    };

    // Create plan
    let mut success_rates = HashMap::new();
    success_rates.insert(action_id, 0.9);

    let plan = planner.create_plan(&analysis, &success_rates).await.unwrap();
    assert!(plan.is_some());

    let plan = plan.unwrap();
    assert_eq!(plan.actions.len(), 1);
    assert_eq!(plan.actions[0], action_id);
}

/// Test knowledge base persistence and learning
#[tokio::test]
async fn test_knowledge_persistence() {
    use knhk_autonomic::knowledge::KnowledgeBase;

    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_kb.db");

    let action_id = Uuid::new_v4();

    // Create knowledge base and record data
    {
        let mut kb = KnowledgeBase::new(db_path.to_str().unwrap()).await.unwrap();

        // Record pattern
        kb.record_pattern("Test pattern", vec![action_id]).await.unwrap();

        // Record successes
        kb.record_success("Test situation", action_id, true).await.unwrap();
        kb.record_success("Test situation", action_id, true).await.unwrap();
        kb.record_success("Test situation", action_id, false).await.unwrap();
    }

    // Reopen knowledge base and verify persistence
    {
        let kb = KnowledgeBase::new(db_path.to_str().unwrap()).await.unwrap();

        let patterns = kb.get_patterns().await.unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].description, "Test pattern");

        let memories = kb.get_memories().await.unwrap();
        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].attempts, 3);
        assert_eq!(memories[0].successes, 2);

        let rate = kb.get_success_rate(&action_id).await.unwrap();
        assert!((rate - 0.666).abs() < 0.01); // 2/3 = 0.666...
    }
}

/// Test hooks execution
#[tokio::test]
async fn test_hooks_system() {
    use knhk_autonomic::hooks::{HookRegistry, HookType, HookContext};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let mut registry = HookRegistry::new();

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);

    // Register hook
    registry.register(HookType::PostMonitor, move |_ctx| {
        let counter = Arc::clone(&counter_clone);
        async move {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }).await;

    // Execute hook
    let ctx = HookContext::new();
    registry.execute(HookType::PostMonitor, &ctx).await.unwrap();

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}
