# Doctrine-Aware Failure Modes: Usage Examples

## Basic Setup

```rust
use knhk_workflow_engine::autonomic::*;
use std::sync::Arc;

// Create MAPE-K controller with failure mode support
let config = ControllerConfig::default();
let kb = Arc::new(KnowledgeBase::new());
let mut monitor = Monitor::new(kb.clone());
monitor.add_collector(Arc::new(WorkflowMetricsCollector::new()));

let controller = MapeKController::new(config, monitor);

// Access mode manager
let mode_manager = controller.mode_manager();
```

## Example 1: Automatic Degradation on Component Failure

```rust
// System starts in Normal mode
assert_eq!(controller.autonomic_mode().await, AutonomicMode::Normal);

// Monitor starts reporting degraded health
mode_manager.update_health(
    HealthSignal::new(ComponentType::Monitor, 0.55)
).await?;

// System automatically degrades to Conservative
assert_eq!(controller.autonomic_mode().await, AutonomicMode::Conservative);

// Monitor fails completely
mode_manager.update_health(
    HealthSignal::new(ComponentType::Monitor, 0.2)
).await?;

// System freezes for safety
assert_eq!(controller.autonomic_mode().await, AutonomicMode::Frozen);
```

## Example 2: Action Filtering by Mode

```rust
let filter = ModePolicyFilter::new();

// Plan with various action types
let actions = vec![
    Action::new(ActionType::ScaleInstances { delta: 2 }),
    Action::new(ActionType::AdjustResources {
        resource: "cpu".to_string(),
        amount: 0.1,
    }),
    Action::new(ActionType::OptimizePattern { pattern_id: 12 }),
];

// Normal mode: all actions allowed
let mode = AutonomicMode::Normal;
let filtered = filter.filter_actions(&actions, mode);
assert_eq!(filtered.len(), 3);

// Conservative mode: only safe actions
let mode = AutonomicMode::Conservative;
let filtered = filter.filter_actions(&actions, mode);
assert_eq!(filtered.len(), 2); // AdjustResources + OptimizePattern

// Frozen mode: no actions
let mode = AutonomicMode::Frozen;
let filtered = filter.filter_actions(&actions, mode);
assert_eq!(filtered.len(), 0);
```

## Example 3: Manual Override for Maintenance

```rust
// Force system into frozen mode for maintenance
mode_manager.set_manual_override(AutonomicMode::Frozen).await?;

println!("System frozen for maintenance");

// Perform maintenance tasks...
// - Update configurations
// - Apply patches
// - Test changes

// Even if health improves, mode stays frozen
mode_manager.update_health(
    HealthSignal::new(ComponentType::Monitor, 0.95)
).await?;
assert_eq!(mode_manager.current_mode().await, AutonomicMode::Frozen);

// Resume automatic mode management
mode_manager.clear_manual_override().await?;

// System recovers to Normal with good health
assert_eq!(mode_manager.current_mode().await, AutonomicMode::Normal);
```

## Example 4: Rejected Action Handling

```rust
let filter = ModePolicyFilter::new();
let actions = vec![
    Action::new(ActionType::ScaleInstances { delta: 2 }),
    Action::new(ActionType::MigrateRuntime {
        from: "W1".to_string(),
        to: "R1".to_string(),
    }),
];

// Filter in Conservative mode
let (allowed, rejected) = filter.filter_with_rejected(
    &actions,
    AutonomicMode::Conservative
);

// All structural changes rejected
assert_eq!(allowed.len(), 0);
assert_eq!(rejected.len(), 2);

// Examine rejection reasons
for rejection in rejected {
    println!(
        "Action {:?} rejected: {}",
        rejection.action.action_type,
        rejection.reason
    );
    println!(
        "Current mode: {:?}, Required: {:?}",
        rejection.current_mode,
        rejection.required_mode
    );
}
```

## Example 5: Mode Change History

```rust
// Make several mode changes
mode_manager.update_health(
    HealthSignal::new(ComponentType::Monitor, 0.5)
).await?; // → Conservative

mode_manager.update_health(
    HealthSignal::new(ComponentType::Monitor, 0.2)
).await?; // → Frozen

mode_manager.update_health(
    HealthSignal::new(ComponentType::Monitor, 0.9)
).await?; // → Normal

// Query history
let history = mode_manager.get_history().await;

for event in history {
    println!(
        "[{}] {} → {} ({})",
        event.timestamp_ms,
        event.from,
        event.to,
        event.reason
    );

    if event.manual_override {
        println!("  (Manual override)");
    }

    println!("  Health: {:.2}", event.metrics.overall_score);
}
```

## Example 6: Custom Action Annotations

```rust
let mut filter = ModePolicyFilter::new();

// Define custom safe action
filter.add_annotation(ActionAnnotation {
    action_pattern: ActionPattern::Custom {
        name: "emit_metrics".to_string(),
    },
    minimum_mode: MinimumMode::Frozen, // Even allowed when frozen
    rationale: "Metrics emission is read-only and always safe".to_string(),
});

// Define custom risky action
filter.add_annotation(ActionAnnotation {
    action_pattern: ActionPattern::Custom {
        name: "modify_schema".to_string(),
    },
    minimum_mode: MinimumMode::Normal, // Only in Normal mode
    rationale: "Schema modifications require full system confidence".to_string(),
});

// Test filtering
assert!(filter.is_allowed(
    &ActionType::Custom {
        name: "emit_metrics".to_string(),
        params: "{}".to_string(),
    },
    AutonomicMode::Frozen // ✓ Allowed even when frozen
));

assert!(!filter.is_allowed(
    &ActionType::Custom {
        name: "modify_schema".to_string(),
        params: "{}".to_string(),
    },
    AutonomicMode::Conservative // ✗ Requires Normal
));
```

## Example 7: Health Signal Reporting

```rust
// Report detailed health from Monitor
mode_manager.update_health(
    HealthSignal::new(ComponentType::Monitor, 0.85)
        .with_details(
            format!(
                "Collected 17/20 expected metrics (85%)"
            )
        )
).await?;

// Report confidence from Analyzer
let analysis = analyzer.analyze().await?;
let confidence = match analysis.health {
    HealthStatus::Healthy => 1.0,
    HealthStatus::Degraded => 0.7,
    HealthStatus::Unhealthy => 0.4,
    HealthStatus::Critical => 0.2,
};

mode_manager.update_health(
    HealthSignal::new(ComponentType::Analyzer, confidence)
        .with_details(
            format!(
                "Health: {:?}, Anomalies: {}",
                analysis.health,
                analysis.anomalies.len()
            )
        )
).await?;

// Report executor reliability
let success_rate = exec_results.iter()
    .filter(|r| r.success)
    .count() as f64 / exec_results.len() as f64;

mode_manager.update_health(
    HealthSignal::new(ComponentType::Executor, success_rate)
        .with_details(
            format!(
                "{}/{} actions succeeded",
                exec_results.iter().filter(|r| r.success).count(),
                exec_results.len()
            )
        )
).await?;
```

## Example 8: Querying Health Metrics

```rust
// Get current health metrics
let metrics = mode_manager.get_health_metrics().await;

println!("Overall health: {:.2}", metrics.overall_score);
println!("Monitor completeness: {:.2}", metrics.monitor_completeness);
println!("Analyzer confidence: {:.2}", metrics.analyzer_confidence);
println!("Planner viability: {:.2}", metrics.planner_viability);
println!("Executor reliability: {:.2}", metrics.executor_reliability);
println!("Knowledge staleness: {}ms", metrics.knowledge_staleness_ms);

// Determine what mode the health suggests
let suggested_mode = metrics.determine_mode();
println!("Suggested mode: {:?}", suggested_mode);
```

## Example 9: Mode-Aware Adaptation Plan

```rust
// Generate plan
let analysis = analyzer.analyze().await?;
let plan = planner.plan(&analysis).await?;

if let Some(plan) = plan {
    // Get current mode
    let mode = mode_manager.current_mode().await;

    // Create mode-aware plan
    let aware_plan = ModeAwareAdaptationPlan::from_plan(
        &plan,
        mode,
        &filter
    );

    println!("Mode: {:?}", aware_plan.mode);
    println!("Total actions: {}", aware_plan.total_actions());
    println!("Allowed: {}", aware_plan.allowed_actions.len());
    println!("Rejected: {}", aware_plan.rejected_actions.len());

    // Log rejected actions
    for rejection in &aware_plan.rejected_actions {
        eprintln!(
            "⚠️  Rejected: {:?} - {}",
            rejection.action.action_type,
            rejection.reason
        );
    }

    // Execute only allowed actions
    if aware_plan.has_allowed_actions() {
        let mut filtered_plan = plan.clone();
        filtered_plan.actions = aware_plan.allowed_actions;
        executor.execute(&filtered_plan).await?;
    }
}
```

## Example 10: Full MAPE-K Cycle with Modes

```rust
async fn execute_cycle_with_modes(
    controller: &MapeKController,
) -> WorkflowResult<()> {
    let mode_manager = controller.mode_manager();
    let monitor = controller.monitor();
    let policy_filter = ModePolicyFilter::new();

    // 1. Check current mode
    let mode = mode_manager.current_mode().await;
    println!("Cycle mode: {:?}", mode);

    // 2. In Frozen mode, only observe
    if mode == AutonomicMode::Frozen {
        println!("System frozen - observation only");
        return Ok(());
    }

    // 3. MONITOR: Emit health signal
    let monitor_health = if monitor.is_running().await {
        HealthSignal::new(ComponentType::Monitor, 0.9)
    } else {
        HealthSignal::new(ComponentType::Monitor, 0.0)
    };
    mode_manager.update_health(monitor_health).await?;

    // 4. ANALYZE: Get analysis and emit confidence
    let analyzer = Analyzer::new(controller.knowledge());
    let analysis = analyzer.analyze().await?;

    let confidence = match analysis.health {
        HealthStatus::Healthy => 1.0,
        HealthStatus::Degraded => 0.7,
        HealthStatus::Unhealthy => 0.4,
        HealthStatus::Critical => 0.2,
    };
    mode_manager.update_health(
        HealthSignal::new(ComponentType::Analyzer, confidence)
    ).await?;

    // 5. PLAN: Generate plan and emit viability
    let planner = Planner::new(controller.knowledge());
    if let Some(plan) = planner.plan(&analysis).await? {
        mode_manager.update_health(
            HealthSignal::new(ComponentType::Planner, 1.0)
        ).await?;

        // 6. Filter actions by current mode
        let execution_mode = mode_manager.current_mode().await;
        let (allowed, rejected) = policy_filter
            .filter_with_rejected(&plan.actions, execution_mode);

        println!(
            "Actions: {} allowed, {} rejected",
            allowed.len(),
            rejected.len()
        );

        // 7. EXECUTE: Only allowed actions
        if !allowed.is_empty() {
            let executor = Executor::new();
            let mut filtered_plan = plan.clone();
            filtered_plan.actions = allowed;

            let results = executor.execute(&filtered_plan).await?;
            let success_rate = results.iter()
                .filter(|r| r.success)
                .count() as f64 / results.len() as f64;

            mode_manager.update_health(
                HealthSignal::new(ComponentType::Executor, success_rate)
            ).await?;
        }
    } else {
        // Planner failed - degrade health
        mode_manager.update_health(
            HealthSignal::new(ComponentType::Planner, 0.5)
        ).await?;
    }

    Ok(())
}
```

## Example 11: Gradual Degradation Scenario

```rust
// Simulate gradual system degradation
let scenarios = vec![
    ("Normal operation", 0.95, AutonomicMode::Normal),
    ("Slight degradation", 0.85, AutonomicMode::Normal),
    ("Moderate degradation", 0.65, AutonomicMode::Normal),
    ("Entering conservative", 0.55, AutonomicMode::Conservative),
    ("Deep degradation", 0.4, AutonomicMode::Conservative),
    ("Critical failure", 0.25, AutonomicMode::Frozen),
];

for (desc, health, expected_mode) in scenarios {
    mode_manager.update_health(
        HealthSignal::new(ComponentType::Monitor, health)
    ).await?;

    let actual_mode = mode_manager.current_mode().await;
    println!(
        "{}: health={:.2} → mode={:?} (expected {:?})",
        desc, health, actual_mode, expected_mode
    );

    assert_eq!(actual_mode, expected_mode);
}
```

## Example 12: Recovery Scenario

```rust
// Simulate system recovery
let recovery_steps = vec![
    ("Critical failure", 0.2, AutonomicMode::Frozen),
    ("Partial recovery", 0.4, AutonomicMode::Conservative),
    ("Improving", 0.6, AutonomicMode::Conservative),
    ("Nearly recovered", 0.75, AutonomicMode::Normal),
    ("Fully recovered", 0.95, AutonomicMode::Normal),
];

for (desc, health, expected_mode) in recovery_steps {
    mode_manager.update_health(
        HealthSignal::new(ComponentType::Monitor, health)
    ).await?;
    mode_manager.update_health(
        HealthSignal::new(ComponentType::Analyzer, health)
    ).await?;

    let actual_mode = mode_manager.current_mode().await;
    println!(
        "{}: health={:.2} → mode={:?}",
        desc, health, actual_mode
    );
}

// Check history shows complete degradation-recovery cycle
let history = mode_manager.get_history().await;
assert!(history.iter().any(|e| e.to == AutonomicMode::Frozen));
assert!(history.iter().any(|e| e.from == AutonomicMode::Frozen));
```
