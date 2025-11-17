// Main executable for Fortune 500 RevOps scenario execution
// Runs TechCorp scenario and generates comprehensive results

use knhk::avatars;
use knhk::knhk_client;
use std::path::Path;

// Import from src modules (relative to src/ directory)
mod avatars_module {
    pub use knhk::avatars::*;
}

mod knhk_client_module {
    pub use knhk::knhk_client::*;
}

// Import scenario and results modules
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Include the scenario and results modules inline since they're not in lib.rs yet
#[path = "../scenarios.rs"]
mod scenarios;

#[path = "../results.rs"]
mod results;

use scenarios::DealScenario;
use results::ComprehensiveResults;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    tracing::info!("=== Fortune 500 RevOps Pipeline Execution ===");
    tracing::info!("Scenario: TechCorp Enterprise Deal");

    // Create TechCorp scenario
    let scenario = DealScenario::techcorp();

    tracing::info!("Deal Details:");
    tracing::info!("  Company: {}", scenario.company);
    tracing::info!("  ACV: ${}", scenario.acv);
    tracing::info!("  Discount: {}%", scenario.discount);
    tracing::info!("  Industry: {}", scenario.industry);
    tracing::info!("  Company Size: {} employees", scenario.company_size);

    // Execute scenario
    tracing::info!("\nExecuting complete RevOps workflow...");
    let scenario_result = scenario.execute().await?;

    // Generate comprehensive results
    let results = ComprehensiveResults::from_scenario(scenario_result);

    // Print summary to console
    println!("\n{}", results.generate_summary());

    // Save to JSON file
    let output_path = Path::new("/home/user/knhk/results/techcorp_execution.json");
    results.save_to_file(output_path).await?;

    tracing::info!("\n✓ Results saved to: {}", output_path.display());

    // Print key metrics
    println!("\n=== Key Metrics ===");
    println!("Total Cycle Time: {:.2} hours", results.total_cycle_time_hours);
    println!("Automation Rate: {:.1}%", results.automation_percentage);
    println!("Total Decisions: {}", results.execution_summary.total_decisions);
    println!("Escalations: {}", results.execution_summary.escalations);

    let slo_compliance_rate = if !results.slo_compliance.is_empty() {
        let compliant = results.slo_compliance.iter().filter(|s| s.compliant).count();
        (compliant as f64 / results.slo_compliance.len() as f64) * 100.0
    } else {
        0.0
    };
    println!("SLO Compliance: {:.1}%", slo_compliance_rate);

    println!("\n=== Decision Timeline ===");
    for decision in &results.scenario.decisions {
        if let (Some(stage), Some(avatar), Some(decision_data)) = (
            decision.get("stage").and_then(|s| s.as_str()),
            decision.get("avatar").and_then(|a| a.as_str()),
            decision.get("decision")
        ) {
            if let (Some(outcome), Some(time_ms)) = (
                decision_data.get("outcome").and_then(|o| o.as_str()),
                decision_data.get("decision_time_ms").and_then(|t| t.as_u64())
            ) {
                let hours = time_ms as f64 / 3600000.0;
                println!("  {} ({}): {} [{:.2}h]", stage, avatar, outcome, hours);
            }
        }
    }

    println!("\n=== Avatar Contributions ===");
    for (avatar, count) in &results.execution_summary.avatar_participation {
        println!("  {}: {} decision(s)", avatar, count);
    }

    if results.scenario.success {
        println!("\n✓ Scenario completed successfully!");
        println!("  Output available for FMEA/TRIZ analysis");
    } else {
        println!("\n✗ Scenario failed - review decision log");
    }

    Ok(())
}
