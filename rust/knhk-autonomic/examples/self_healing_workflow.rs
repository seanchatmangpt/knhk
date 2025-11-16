//! # Self-Healing Workflow Example
//!
//! **Covenant 3**: Feedback Loops Run at Machine Speed
//!
//! This example demonstrates the MAPE-K autonomic feedback loop in action.
//! It creates a payment processing workflow that:
//! - Detects failures automatically
//! - Analyzes root causes
//! - Selects recovery actions
//! - Executes corrective actions
//! - Learns from results
//!
//! ## How It Works
//!
//! 1. Setup metrics, rules, policies, and actions
//! 2. Inject a failure (high error rate)
//! 3. Watch MAPE-K detect, analyze, plan, and execute recovery
//! 4. Observe learning and improvement over time

use knhk_autonomic::{
    AutonomicController, Config,
    types::{Action, ActionType, RiskLevel, MetricType, RuleType},
};
use std::time::Duration;
use tracing_subscriber;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== MAPE-K Autonomic Self-Healing Workflow ===\n");

    // Create autonomic controller
    let config = Config::default()
        .with_loop_frequency(Duration::from_secs(2))
        .with_knowledge_path("./examples/knowledge.db");

    let mut controller = AutonomicController::new(config).await?;

    println!("1. Setting up autonomic system...\n");

    // === SETUP MONITORING ===
    {
        let mut monitor = controller.monitor().write().await;

        // Register payment success rate metric
        monitor.register_metric(
            "Payment Success Rate",
            MetricType::Reliability,
            0.99,      // expected: 99%
            0.95,      // anomaly threshold: 95%
            "percent",
        ).await?;

        // Register payment latency metric
        monitor.register_metric(
            "Payment Latency",
            MetricType::Performance,
            1500.0,    // expected: 1500ms
            2500.0,    // anomaly threshold: 2500ms
            "milliseconds",
        ).await?;

        // Register error count metric
        monitor.register_metric(
            "Error Count",
            MetricType::Reliability,
            0.0,       // expected: 0
            5.0,       // anomaly threshold: 5
            "count",
        ).await?;

        println!("✓ Registered 3 metrics");
    }

    // === SETUP ANALYSIS ===
    {
        let mut analyzer = controller.analyzer().write().await;

        // Register high error rate rule
        analyzer.register_rule(
            "High Error Rate Detection",
            RuleType::HighErrorRate,
            "?metric mape:metricName 'Error Count' ; mape:currentValue ?val . FILTER(?val > 5)"
        ).await?;

        // Register performance degradation rule
        analyzer.register_rule(
            "Performance Degradation Detection",
            RuleType::PerformanceDegradation,
            "?metric mape:metricType mape:PerformanceMetric ; mape:isAnomalous true"
        ).await?;

        println!("✓ Registered 2 analysis rules");
    }

    // === SETUP PLANNING ===
    let retry_action_id: Uuid;
    let fallback_action_id: Uuid;
    let optimize_action_id: Uuid;

    {
        let mut planner = controller.planner().write().await;

        // Create retry action
        let retry_action = Action {
            id: Uuid::new_v4(),
            action_type: ActionType::Heal,
            description: "Retry payment with exponential backoff".to_string(),
            target: "payment_processor".to_string(),
            implementation: "retry_handler".to_string(),
            estimated_impact: "Recovers 70% of transient failures".to_string(),
            risk_level: RiskLevel::LowRisk,
        };
        retry_action_id = retry_action.id;
        planner.register_action(retry_action).await?;

        // Create fallback action
        let fallback_action = Action {
            id: Uuid::new_v4(),
            action_type: ActionType::Heal,
            description: "Switch to fallback payment processor".to_string(),
            target: "payment_processor".to_string(),
            implementation: "fallback_handler".to_string(),
            estimated_impact: "Recovers 90% of persistent failures".to_string(),
            risk_level: RiskLevel::MediumRisk,
        };
        fallback_action_id = fallback_action.id;
        planner.register_action(fallback_action).await?;

        // Create optimize action
        let optimize_action = Action {
            id: Uuid::new_v4(),
            action_type: ActionType::Optimize,
            description: "Optimize payment processing performance".to_string(),
            target: "payment_processor".to_string(),
            implementation: "optimize_handler".to_string(),
            estimated_impact: "Reduces latency by 30-50%".to_string(),
            risk_level: RiskLevel::LowRisk,
        };
        optimize_action_id = optimize_action.id;
        planner.register_action(optimize_action).await?;

        // Register retry policy
        planner.register_policy(
            "Retry on Failure",
            "HighErrorRate",
            vec![retry_action_id, fallback_action_id],
            100, // high priority
        ).await?;

        // Register optimize policy
        planner.register_policy(
            "Optimize on Slowdown",
            "PerformanceDegradation",
            vec![optimize_action_id],
            80, // medium priority
        ).await?;

        println!("✓ Registered 3 actions and 2 policies");
    }

    println!("\n2. Starting MAPE-K feedback loop...\n");

    // Start controller in background
    let controller_handle = tokio::spawn({
        let mut controller_clone = controller.clone();
        async move {
            controller_clone.start().await
        }
    });

    // Wait for loop to stabilize
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("3. Injecting failure scenario...\n");

    // === INJECT FAILURE ===
    {
        let mut monitor = controller.monitor().write().await;

        // Simulate high error rate
        monitor.update_metric("Error Count", 10.0).await?;
        monitor.update_metric("Payment Success Rate", 0.85).await?;

        println!("✗ Injected failure: Error Count = 10, Success Rate = 85%");
    }

    println!("\n4. Watching MAPE-K detect and recover...\n");

    // Wait for MAPE-K cycles to detect and respond
    for cycle in 1..=3 {
        tokio::time::sleep(Duration::from_secs(2)).await;
        println!("   Cycle {} complete", cycle);
    }

    println!("\n5. Checking recovery...\n");

    // Check execution history
    {
        let kb = controller.knowledge().read().await;

        let cycles = kb.get_cycles().await?;
        println!("✓ Completed {} MAPE-K cycles", cycles.len());

        if let Some(last_cycle) = cycles.last() {
            println!("   Last cycle outcome: {}", last_cycle.outcome);
            println!("   Effectiveness: {:.1}%", last_cycle.effectiveness * 100.0);
            println!("   Actions executed: {}", last_cycle.executions.len());
        }

        let patterns = kb.get_patterns().await?;
        println!("✓ Learned {} patterns", patterns.len());

        let memories = kb.get_memories().await?;
        println!("✓ Recorded {} success memories", memories.len());

        // Show success rates
        let retry_rate = kb.get_success_rate(&retry_action_id).await?;
        let fallback_rate = kb.get_success_rate(&fallback_action_id).await?;
        let optimize_rate = kb.get_success_rate(&optimize_action_id).await?;

        println!("\n   Action Success Rates:");
        println!("   - Retry:    {:.1}%", retry_rate * 100.0);
        println!("   - Fallback: {:.1}%", fallback_rate * 100.0);
        println!("   - Optimize: {:.1}%", optimize_rate * 100.0);
    }

    println!("\n6. Simulating second failure (should recover faster)...\n");

    // === INJECT SECOND FAILURE ===
    {
        let mut monitor = controller.monitor().write().await;
        monitor.update_metric("Payment Latency", 3500.0).await?;
        println!("✗ Injected performance degradation: Latency = 3500ms");
    }

    // Wait for recovery
    for cycle in 1..=2 {
        tokio::time::sleep(Duration::from_secs(2)).await;
        println!("   Cycle {} complete", cycle);
    }

    println!("\n7. Final knowledge state...\n");

    {
        let kb = controller.knowledge().read().await;
        let cycles = kb.get_cycles().await?;
        let total_effectiveness: f64 = cycles.iter().map(|c| c.effectiveness).sum();
        let avg_effectiveness = total_effectiveness / cycles.len() as f64;

        println!("✓ Total cycles: {}", cycles.len());
        println!("✓ Average effectiveness: {:.1}%", avg_effectiveness * 100.0);
    }

    println!("\n=== MAPE-K Demonstration Complete ===\n");
    println!("Key Takeaways:");
    println!("1. System detected failures automatically (Monitor)");
    println!("2. Root causes were analyzed (Analyze)");
    println!("3. Recovery actions were selected (Plan)");
    println!("4. Actions were executed successfully (Execute)");
    println!("5. System learned and improved (Knowledge)");
    println!("\nResult: Truly autonomous, self-healing workflow!");

    // Stop controller
    controller.stop().await;
    controller_handle.abort();

    Ok(())
}
