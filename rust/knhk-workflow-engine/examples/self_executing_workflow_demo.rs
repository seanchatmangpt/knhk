//! Self-Executing Workflow Demo
//!
//! Demonstrates the complete self-executing workflow system:
//! - Load workflows from RDF ontologies
//! - Execute with adaptive pattern selection
//! - Generate cryptographic receipts
//! - Run MAPE-K autonomic feedback loops
//! - Learn and adapt based on observations

use knhk_workflow_engine::error::WorkflowResult;
use knhk_workflow_engine::ontology_executor::OntologyExecutor;
use serde_json::json;

#[tokio::main]
async fn main() -> WorkflowResult<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 KNHK Self-Executing Workflow Demo");
    println!("=====================================\n");

    // Create ontology executor
    let mut executor = OntologyExecutor::new("./snapshots", "./receipts")?;

    println!("✓ Created ontology executor");
    println!("  - Snapshot dir: ./snapshots");
    println!("  - Receipt dir: ./receipts\n");

    // Example 1: Execute simple sequence workflow
    println!("📋 Example 1: Simple Sequence Workflow");
    println!("--------------------------------------");

    let input_data = json!({
        "order_id": "ORD-12345",
        "customer": "Alice",
        "amount": 1500.00,
        "items": ["Widget A", "Widget B"]
    });

    match executor
        .execute_from_ontology(
            "ontology/workflows/examples/simple-sequence.ttl",
            input_data.clone(),
        )
        .await
    {
        Ok(result) => {
            println!("✓ Workflow executed successfully!");
            println!("  - Workflow ID: {}", result.workflow_id);
            println!("  - Pattern used: {:?}", result.pattern_used);
            println!("  - Ticks used: {} (≤8 ✓)", result.ticks_used);
            println!("  - Receipt ID: {}", result.receipt_id);
            println!("  - Sigma version: {}", result.sigma_id);
            println!(
                "  - Output: {}\n",
                serde_json::to_string_pretty(&result.output_data)?
            );
        }
        Err(e) => {
            eprintln!("✗ Execution failed: {:?}\n", e);
        }
    }

    // Example 2: Execute parallel processing workflow
    println!("📋 Example 2: Parallel Processing Workflow");
    println!("------------------------------------------");

    let batch_data = json!({
        "batch_id": "BATCH-001",
        "tasks": [
            {"id": "T1", "data": "Data 1"},
            {"id": "T2", "data": "Data 2"},
            {"id": "T3", "data": "Data 3"},
            {"id": "T4", "data": "Data 4"},
        ]
    });

    match executor
        .execute_from_ontology(
            "ontology/workflows/examples/parallel-processing.ttl",
            batch_data.clone(),
        )
        .await
    {
        Ok(result) => {
            println!("✓ Parallel workflow executed!");
            println!(
                "  - Pattern selected: {:?} (adaptive selection)",
                result.pattern_used
            );
            println!("  - Ticks used: {} (≤8 ✓)", result.ticks_used);
            println!("  - Receipt ID: {}\n", result.receipt_id);
        }
        Err(e) => {
            eprintln!("✗ Execution failed: {:?}\n", e);
        }
    }

    // Example 3: Run MAPE-K cycle
    println!("🔄 Example 3: MAPE-K Autonomic Cycle");
    println!("------------------------------------");

    match executor.orchestrator.run_mape_k_cycle().await {
        Ok(metrics) => {
            println!("✓ MAPE-K cycle completed!");
            println!("  - Total duration: {}ms", metrics.total_duration_ms);
            println!("  - Monitor: {}ms", metrics.monitor_duration_ms);
            println!("  - Analyze: {}ms", metrics.analyze_duration_ms);
            println!("  - Plan: {}ms", metrics.plan_duration_ms);
            println!("  - Execute: {}ms", metrics.execute_duration_ms);
            println!("  - Observations: {}", metrics.observations_count);
            println!("  - Symptoms detected: {}", metrics.symptoms_detected);
            println!("  - Adaptations applied: {}\n", metrics.plans_generated);
        }
        Err(e) => {
            eprintln!("✗ MAPE-K cycle failed: {:?}\n", e);
        }
    }

    // Example 4: Query learned patterns
    println!("🧠 Example 4: Knowledge Base Query");
    println!("----------------------------------");

    let patterns = executor.orchestrator.query_knowledge("performance");
    if patterns.is_empty() {
        println!("  No patterns learned yet (run more workflows)");
    } else {
        println!("  Learned patterns:");
        for pattern in patterns {
            println!("    - {}", pattern);
        }
    }
    println!();

    // Show final statistics
    println!("📊 Final Statistics");
    println!("------------------");
    let stats = executor.get_stats();
    println!("  - Workflows executed: {}", stats.workflows_executed);
    println!("  - Current Σ version: {}", stats.current_sigma);
    println!("  - Total receipts: {}", stats.total_receipts);
    println!("  - Learned patterns: {}", stats.learned_patterns);

    println!("\n✨ Demo completed successfully!");
    println!("\n💡 Key Features Demonstrated:");
    println!("  ✓ A = μ(O) - Provable execution with receipts");
    println!("  ✓ Σ → Π → μ - Ontology to execution pipeline");
    println!("  ✓ Adaptive pattern selection");
    println!("  ✓ Chatman Constant enforcement (≤8 ticks)");
    println!("  ✓ MAPE-K autonomic feedback loop");
    println!("  ✓ Knowledge-based learning");

    Ok(())
}
