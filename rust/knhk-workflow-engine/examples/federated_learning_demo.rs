//! Federated Learning Demo
//!
//! Demonstrates Byzantine-robust federated learning across AI agent swarms.
//!
//! # Features
//!
//! - 100 agents with distributed training
//! - Byzantine tolerance (f < n/3)
//! - Convergence monitoring (KL divergence < 0.01)
//! - Full OpenTelemetry instrumentation
//!
//! # Usage
//!
//! ```bash
//! cargo run --example federated_learning_demo
//! ```

use knhk_workflow_engine::federated::*;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(false)
        .init();

    info!("=== Federated Learning Demo ===");
    info!("Initializing 100-agent swarm...");

    // Create federated coordinator with 100 agents
    let num_agents = 100;
    let coordinator = FederatedLearningCoordinator::new_with_defaults(num_agents);

    // Populate experience buffers for all agents
    info!("Populating experience buffers...");
    {
        let coord = coordinator.read().await;
        for i in 0..coord.num_agents() {
            if let Some(agent) = coord.agent(i) {
                let mut agent = agent.write().await;

                // Add diverse experiences (simulating different workflows)
                for j in 0..1000 {
                    agent.buffer_mut().push(Experience {
                        state: generate_random_state(i, j),
                        action: (i + j) % 10,
                        reward: calculate_reward(i, j),
                        next_state: generate_random_state(i, j + 1),
                        done: j % 100 == 99,
                    });
                }
            }
        }
    }

    info!("Starting federated learning...");
    info!("Target: Converge within 1000 rounds (KL < 0.01)");

    // Run federated learning rounds
    let max_rounds = 1000;
    for round in 1..=max_rounds {
        let metrics = {
            let mut coord = coordinator.write().await;
            coord.run_federated_round().await?
        };

        // Log progress every 10 rounds
        if round % 10 == 0 || metrics.converged {
            info!(
                "Round {:4}: loss={:.4}, kl={:.6}, byzantine={:2}, time={}ms, converged={}",
                round,
                metrics.avg_local_loss,
                metrics.kl_divergence,
                metrics.byzantine_count,
                metrics.total_duration_ms,
                metrics.converged
            );
        }

        // Check convergence
        if metrics.converged {
            info!("🎉 Federated learning CONVERGED in {} rounds!", round);
            info!("  - Final KL divergence: {:.6}", metrics.kl_divergence);
            info!("  - Final loss: {:.4}", metrics.avg_local_loss);
            info!("  - Byzantine agents detected: {}", metrics.byzantine_count);
            break;
        }

        if round == max_rounds {
            info!("⚠️  Did not converge within {} rounds", max_rounds);
            info!("  - Current KL divergence: {:.6}", metrics.kl_divergence);
        }
    }

    info!("=== Demo Complete ===");
    Ok(())
}

/// Generate random state for agent (simulates workflow features)
fn generate_random_state(agent_id: usize, step: usize) -> Vec<f32> {
    let seed = (agent_id * 1000 + step) as u64;
    fastrand::seed(seed);

    (0..100).map(|_| fastrand::f32()).collect()
}

/// Calculate reward for agent (simulates workflow performance)
fn calculate_reward(agent_id: usize, step: usize) -> f64 {
    let base = (agent_id as f64 / 100.0) + (step as f64 / 1000.0);
    (base % 1.0) * 0.5 + 0.25 // Range: [0.25, 0.75]
}
