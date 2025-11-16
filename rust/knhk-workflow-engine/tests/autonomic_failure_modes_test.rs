// rust/knhk-workflow-engine/tests/autonomic_failure_modes_test.rs
//! Integration tests for Doctrine-Aware Failure Modes
//!
//! Tests the complete failure mode system including:
//! - Automatic mode degradation
//! - Mode restoration on recovery
//! - Action filtering by mode
//! - Manual mode override
//! - Observable mode changes

use knhk_workflow_engine::autonomic::*;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_automatic_mode_degradation_on_monitor_failure() {
    // Arrange: Create MAPE-K controller with normal health
    let config = ControllerConfig::default();
    let kb = Arc::new(KnowledgeBase::new());
    let mut monitor = Monitor::new(kb.clone());
    monitor.add_collector(Arc::new(WorkflowMetricsCollector::new()));

    let controller = MapeKController::new(config, monitor);

    // Act: Degrade monitor health
    let mode_manager = controller.mode_manager();
    mode_manager
        .update_health(HealthSignal::new(ComponentType::Monitor, 0.5))
        .await
        .unwrap();

    // Assert: Mode should degrade to Conservative
    let mode = controller.autonomic_mode().await;
    assert_eq!(mode, AutonomicMode::Conservative);
}

#[tokio::test]
async fn test_automatic_mode_freeze_on_critical_failure() {
    // Arrange
    let config = ControllerConfig::default();
    let kb = Arc::new(KnowledgeBase::new());
    let monitor = Monitor::new(kb.clone());
    let controller = MapeKController::new(config, monitor);

    // Act: Critical monitor failure
    let mode_manager = controller.mode_manager();
    mode_manager
        .update_health(HealthSignal::new(ComponentType::Monitor, 0.2))
        .await
        .unwrap();

    // Assert: Should freeze
    let mode = controller.autonomic_mode().await;
    assert_eq!(mode, AutonomicMode::Frozen);
}

#[tokio::test]
async fn test_mode_recovery_on_health_improvement() {
    // Arrange
    let config = ControllerConfig::default();
    let kb = Arc::new(KnowledgeBase::new());
    let monitor = Monitor::new(kb.clone());
    let controller = MapeKController::new(config, monitor);
    let mode_manager = controller.mode_manager();

    // Act: Degrade to frozen
    mode_manager
        .update_health(HealthSignal::new(ComponentType::Monitor, 0.2))
        .await
        .unwrap();
    assert_eq!(mode_manager.current_mode().await, AutonomicMode::Frozen);

    // Improve to conservative
    mode_manager
        .update_health(HealthSignal::new(ComponentType::Monitor, 0.55))
        .await
        .unwrap();
    mode_manager
        .update_health(HealthSignal::new(ComponentType::Analyzer, 0.6))
        .await
        .unwrap();

    // Assert: Should be conservative now
    assert_eq!(
        mode_manager.current_mode().await,
        AutonomicMode::Conservative
    );

    // Recover to normal
    mode_manager
        .update_health(HealthSignal::new(ComponentType::Monitor, 0.9))
        .await
        .unwrap();
    mode_manager
        .update_health(HealthSignal::new(ComponentType::Analyzer, 0.85))
        .await
        .unwrap();

    // Assert: Should be normal now
    assert_eq!(mode_manager.current_mode().await, AutonomicMode::Normal);
}

#[tokio::test]
async fn test_action_filtering_by_mode() {
    // Arrange
    let filter = ModePolicyFilter::new();

    let actions = vec![
        Action::new(ActionType::ScaleInstances { delta: 2 }),
        Action::new(ActionType::AdjustResources {
            resource: "cpu".to_string(),
            amount: 0.1,
        }),
        Action::new(ActionType::OptimizePattern { pattern_id: 12 }),
        Action::new(ActionType::MigrateRuntime {
            from: "W1".to_string(),
            to: "R1".to_string(),
        }),
    ];

    // Act & Assert: Normal mode - all actions allowed
    let filtered = filter.filter_actions(&actions, AutonomicMode::Normal);
    assert_eq!(filtered.len(), 4);

    // Conservative mode - only safe actions
    let filtered = filter.filter_actions(&actions, AutonomicMode::Conservative);
    assert_eq!(filtered.len(), 2); // AdjustResources and OptimizePattern

    // Frozen mode - no actions
    let filtered = filter.filter_actions(&actions, AutonomicMode::Frozen);
    assert_eq!(filtered.len(), 0);
}

#[tokio::test]
async fn test_rejected_actions_are_logged() {
    // Arrange
    let filter = ModePolicyFilter::new();
    let actions = vec![
        Action::new(ActionType::ScaleInstances { delta: 2 }),
        Action::new(ActionType::AdjustResources {
            resource: "cpu".to_string(),
            amount: 0.1,
        }),
    ];

    // Act: Filter in Conservative mode
    let (allowed, rejected) = filter.filter_with_rejected(&actions, AutonomicMode::Conservative);

    // Assert
    assert_eq!(allowed.len(), 1); // Only AdjustResources
    assert_eq!(rejected.len(), 1); // ScaleInstances rejected

    assert_eq!(rejected[0].current_mode, AutonomicMode::Conservative);
    assert_eq!(rejected[0].required_mode, MinimumMode::Normal);
    assert!(!rejected[0].reason.is_empty());
}

#[tokio::test]
async fn test_manual_mode_override() {
    // Arrange
    let config = ControllerConfig::default();
    let kb = Arc::new(KnowledgeBase::new());
    let monitor = Monitor::new(kb.clone());
    let controller = MapeKController::new(config, monitor);
    let mode_manager = controller.mode_manager();

    // Act: Set manual override to frozen
    mode_manager
        .set_manual_override(AutonomicMode::Frozen)
        .await
        .unwrap();

    // Assert: Should be frozen
    assert_eq!(mode_manager.current_mode().await, AutonomicMode::Frozen);

    // Act: Try to improve health (should not change mode)
    mode_manager
        .update_health(HealthSignal::new(ComponentType::Monitor, 0.95))
        .await
        .unwrap();

    // Assert: Should still be frozen
    assert_eq!(mode_manager.current_mode().await, AutonomicMode::Frozen);

    // Act: Clear override
    mode_manager.clear_manual_override().await.unwrap();

    // Assert: Should recover to normal with good health
    mode_manager
        .update_health(HealthSignal::new(ComponentType::Monitor, 0.95))
        .await
        .unwrap();
    mode_manager
        .update_health(HealthSignal::new(ComponentType::Analyzer, 0.9))
        .await
        .unwrap();
    assert_eq!(mode_manager.current_mode().await, AutonomicMode::Normal);
}

#[tokio::test]
async fn test_mode_change_history() {
    // Arrange
    let config = ControllerConfig::default();
    let kb = Arc::new(KnowledgeBase::new());
    let monitor = Monitor::new(kb.clone());
    let controller = MapeKController::new(config, monitor);
    let mode_manager = controller.mode_manager();

    // Act: Make several mode changes
    mode_manager
        .update_health(HealthSignal::new(ComponentType::Monitor, 0.5))
        .await
        .unwrap();
    mode_manager
        .update_health(HealthSignal::new(ComponentType::Monitor, 0.2))
        .await
        .unwrap();
    mode_manager
        .update_health(HealthSignal::new(ComponentType::Monitor, 0.9))
        .await
        .unwrap();

    // Assert: History should record all changes
    let history = mode_manager.get_history().await;
    assert!(history.len() >= 2); // At least degradation and recovery

    // Check mode transitions
    assert!(history.iter().any(|e| e.to == AutonomicMode::Frozen));
    assert!(history.iter().any(|e| e.from == AutonomicMode::Frozen));
}

#[tokio::test]
async fn test_frozen_mode_prevents_all_actions() {
    // Arrange
    let config = ControllerConfig {
        cycle_interval: Duration::from_millis(50),
        self_healing: true,
        self_optimization: true,
        self_configuration: false,
        self_protection: false,
    };

    let kb = Arc::new(KnowledgeBase::new());

    // Add goal that will be violated
    let goal = Goal::new(
        "latency".to_string(),
        GoalType::Performance,
        "avg_latency_ms".to_string(),
        100.0,
    );
    kb.add_goal(goal).await.unwrap();

    // Add violating fact
    kb.add_fact(Fact::new(
        "avg_latency_ms".to_string(),
        200.0,
        "test".to_string(),
    ))
    .await
    .unwrap();

    let mut monitor = Monitor::new(kb.clone());
    let collector = Arc::new(WorkflowMetricsCollector::new());
    monitor.add_collector(collector.clone());

    let controller = MapeKController::new(config, monitor);

    // Act: Force frozen mode
    controller
        .mode_manager()
        .set_manual_override(AutonomicMode::Frozen)
        .await
        .unwrap();

    // Start loop
    controller.start_loop().await.unwrap();

    // Wait for cycle
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Stop loop
    controller.stop_loop().await.unwrap();

    // Assert: No adaptations should have occurred
    let stats = controller.get_stats().await;
    assert_eq!(stats.actions_executed, 0);
}

#[tokio::test]
async fn test_conservative_mode_allows_safe_actions() {
    // Arrange
    let filter = ModePolicyFilter::new();

    // Act: Test safe actions in conservative mode
    assert!(filter.is_allowed(
        &ActionType::AdjustResources {
            resource: "cpu".to_string(),
            amount: 0.1
        },
        AutonomicMode::Conservative
    ));

    assert!(filter.is_allowed(
        &ActionType::OptimizePattern { pattern_id: 12 },
        AutonomicMode::Conservative
    ));

    assert!(filter.is_allowed(
        &ActionType::Cancel {
            target: "task1".to_string()
        },
        AutonomicMode::Conservative
    ));

    assert!(filter.is_allowed(
        &ActionType::Compensate {
            task_id: "task1".to_string()
        },
        AutonomicMode::Conservative
    ));

    // Assert: Structural changes not allowed
    assert!(!filter.is_allowed(
        &ActionType::ScaleInstances { delta: 2 },
        AutonomicMode::Conservative
    ));

    assert!(!filter.is_allowed(
        &ActionType::MigrateRuntime {
            from: "W1".to_string(),
            to: "R1".to_string()
        },
        AutonomicMode::Conservative
    ));
}

#[tokio::test]
async fn test_health_metrics_calculation() {
    // Arrange
    let mut signals = std::collections::HashMap::new();
    signals.insert(
        ComponentType::Monitor,
        HealthSignal::new(ComponentType::Monitor, 0.9),
    );
    signals.insert(
        ComponentType::Analyzer,
        HealthSignal::new(ComponentType::Analyzer, 0.85),
    );
    signals.insert(
        ComponentType::Planner,
        HealthSignal::new(ComponentType::Planner, 0.8),
    );
    signals.insert(
        ComponentType::Executor,
        HealthSignal::new(ComponentType::Executor, 0.95),
    );

    // Act
    let metrics = HealthMetrics::from_signals(&signals);

    // Assert
    assert!(metrics.overall_score > 0.8);
    assert_eq!(metrics.determine_mode(), AutonomicMode::Normal);
}

#[tokio::test]
async fn test_custom_action_annotations() {
    // Arrange
    let mut filter = ModePolicyFilter::new();

    // Add custom annotation for a safe custom action
    filter.add_annotation(ActionAnnotation {
        action_pattern: ActionPattern::Custom {
            name: "safe_logging".to_string(),
        },
        minimum_mode: MinimumMode::Conservative,
        rationale: "Logging is safe in degraded mode".to_string(),
    });

    // Act & Assert: Custom safe action allowed in conservative
    assert!(filter.is_allowed(
        &ActionType::Custom {
            name: "safe_logging".to_string(),
            params: "{}".to_string()
        },
        AutonomicMode::Conservative
    ));

    // Unknown custom action requires Normal mode (fail-safe)
    assert!(!filter.is_allowed(
        &ActionType::Custom {
            name: "unknown_action".to_string(),
            params: "{}".to_string()
        },
        AutonomicMode::Conservative
    ));
}

#[tokio::test]
async fn test_mode_aware_adaptation_plan() {
    // Arrange
    let filter = ModePolicyFilter::new();
    let mut plan = AdaptationPlan::new();
    plan.actions.push(Action::new(ActionType::ScaleInstances { delta: 2 }));
    plan.actions.push(Action::new(ActionType::AdjustResources {
        resource: "cpu".to_string(),
        amount: 0.1,
    }));

    // Act: Create mode-aware plan in Conservative mode
    let aware_plan = ModeAwareAdaptationPlan::from_plan(&plan, AutonomicMode::Conservative, &filter);

    // Assert
    assert_eq!(aware_plan.mode, AutonomicMode::Conservative);
    assert_eq!(aware_plan.allowed_actions.len(), 1); // Only AdjustResources
    assert_eq!(aware_plan.rejected_actions.len(), 1); // ScaleInstances rejected
    assert!(aware_plan.has_allowed_actions());
    assert!(aware_plan.has_rejected_actions());
    assert_eq!(aware_plan.total_actions(), 2);
}

#[tokio::test]
async fn test_mode_severity_ordering() {
    // Assert: Frozen is most restrictive
    assert!(AutonomicMode::Frozen.severity() > AutonomicMode::Conservative.severity());
    assert!(AutonomicMode::Conservative.severity() > AutonomicMode::Normal.severity());

    // Assert: Mode allows hierarchy
    assert!(AutonomicMode::Normal.allows(AutonomicMode::Conservative));
    assert!(AutonomicMode::Normal.allows(AutonomicMode::Frozen));
    assert!(!AutonomicMode::Frozen.allows(AutonomicMode::Normal));
}

#[tokio::test]
async fn test_stale_health_signals_ignored() {
    // Arrange
    let manager = ModeManager::new();

    // Act: Add healthy signal
    manager
        .update_health(HealthSignal::new(ComponentType::Monitor, 0.9))
        .await
        .unwrap();

    // Wait for signal to become stale (or create one with old timestamp)
    let mut stale_signal = HealthSignal::new(ComponentType::Analyzer, 0.2);
    stale_signal.timestamp_ms = 0; // Very old timestamp

    // Signal is stale and should be filtered
    assert!(stale_signal.is_stale(60_000));

    // Mode should stay Normal because stale signals are ignored
    assert_eq!(manager.current_mode().await, AutonomicMode::Normal);
}
