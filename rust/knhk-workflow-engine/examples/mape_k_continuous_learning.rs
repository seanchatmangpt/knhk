//! MAPE-K Continuous Learning Demo
//!
//! Demonstrates continuous autonomic adaptation using the MAPE-K loop.
//! Shows how workflows self-optimize based on runtime observations.

use knhk_workflow_engine::error::WorkflowResult;
use knhk_workflow_engine::orchestrator::SelfExecutingOrchestrator;
use serde_json::json;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> WorkflowResult<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔄 MAPE-K Continuous Learning Demo");
    println!("===================================\n");

    // Create orchestrator
    let mut orchestrator = SelfExecutingOrchestrator::new("./snapshots", "./receipts")?;

    // Load a workflow
    let workflow_id = orchestrator
        .load_workflow_from_ontology("ontology/workflows/examples/simple-sequence.ttl")
        .await?;

    println!("✓ Loaded workflow: {}\n", workflow_id);

    println!("🎯 Running 100 workflow executions to generate observations...");
    println!("--------------------------------------------------------------\n");

    // Execute workflow multiple times to generate observations
    for i in 0..100 {
        let input = json!({
            "iteration": i,
            "data": format!("Test data {}", i),
            "size": (i % 10) + 1, // Varying data sizes
        });

        match orchestrator.execute_workflow(&workflow_id, input).await {
            Ok(result) => {
                if i % 10 == 0 {
                    println!(
                        "  Execution {}: {} ticks, receipt: {}",
                        i, result.ticks_used, result.receipt_id
                    );
                }
            }
            Err(e) => {
                eprintln!("  Execution {} failed: {:?}", i, e);
            }
        }

        // Small delay between executions
        sleep(Duration::from_millis(10)).await;
    }

    println!("\n✓ Completed 100 executions\n");

    println!("🔍 Running MAPE-K Cycles for Continuous Adaptation");
    println!("--------------------------------------------------\n");

    // Run several MAPE-K cycles
    for cycle in 1..=5 {
        println!("Cycle {}/5:", cycle);

        match orchestrator.run_mape_k_cycle().await {
            Ok(metrics) => {
                println!(
                    "  Monitor:  Collected {} observations in {}ms",
                    metrics.observations_count, metrics.monitor_duration_ms
                );
                println!(
                    "  Analyze:  Detected {} symptoms in {}ms",
                    metrics.symptoms_detected, metrics.analyze_duration_ms
                );
                println!(
                    "  Plan:     Generated {} plans in {}ms",
                    metrics.plans_generated, metrics.plan_duration_ms
                );
                println!(
                    "  Execute:  Applied adaptations in {}ms",
                    metrics.execute_duration_ms
                );
                println!("  Total:    {}ms\n", metrics.total_duration_ms);

                if metrics.symptoms_detected > 0 {
                    println!("  ⚠️  Symptoms detected! System is adapting...\n");
                } else {
                    println!("  ✓ System operating normally\n");
                }
            }
            Err(e) => {
                eprintln!("  ✗ MAPE-K cycle failed: {:?}\n", e);
            }
        }

        sleep(Duration::from_secs(2)).await;
    }

    println!("🧠 Knowledge Base Insights");
    println!("-------------------------");

    let performance_patterns = orchestrator.query_knowledge("performance");
    let guard_patterns = orchestrator.query_knowledge("guard");

    println!("Performance patterns learned:");
    if performance_patterns.is_empty() {
        println!("  (none yet - more data needed)");
    } else {
        for pattern in performance_patterns {
            println!("  - {}", pattern);
        }
    }

    println!("\nGuard patterns learned:");
    if guard_patterns.is_empty() {
        println!("  (none yet - more data needed)");
    } else {
        for pattern in guard_patterns {
            println!("  - {}", pattern);
        }
    }

    println!("\n📊 Final System State");
    println!("--------------------");
    let stats = orchestrator.get_statistics();
    println!("Workflows: {}", stats.workflows_loaded);
    println!("Current Σ: {}", stats.current_sigma_id);
    println!("Receipts:  {}", stats.total_receipts);

    println!("\n✨ Demonstration Complete!");
    println!("\n💡 What Happened:");
    println!("  1. ✓ Executed 100 workflows with varying characteristics");
    println!("  2. ✓ Generated 100 cryptographic receipts (observation plane)");
    println!("  3. ✓ Ran 5 MAPE-K cycles:");
    println!("     - Monitored execution patterns");
    println!("     - Analyzed for symptoms (performance, guards)");
    println!("     - Planned adaptations (config, patterns)");
    println!("     - Executed adaptations via shadow deployment");
    println!("     - Learned patterns for future optimization");
    println!("  4. ✓ System continuously self-optimizes");

    println!("\n🎓 Key Concepts:");
    println!("  • A = μ(O) - Every action provably derived from observations");
    println!("  • μ ∘ μ = μ - Idempotent execution via snapshots");
    println!("  • τ ≤ 8 - Chatman Constant enforced");
    println!("  • MAPE-K - Autonomic computing feedback loop");
    println!("  • Shadow Deployment - Safe adaptation testing");

    Ok(())
}
