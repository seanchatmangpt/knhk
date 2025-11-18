//! Federated Learning for KNHK AI Agent Swarms
//!
//! This module implements Byzantine-robust federated learning with:
//! - Byzantine tolerance: f < n/3 malicious agents
//! - Convergence guarantees: KL divergence < 0.01 in <1000 rounds
//! - Sub-8-tick performance: <150ms per round (off hot-path)
//! - Full observability: OpenTelemetry Weaver validation
//!
//! # DOCTRINE ALIGNMENT
//!
//! - **Principle**: MAPE-K (Analyze learns from distributed execution) + O (All learning observable)
//! - **Covenants**: 3 (Feedback loops at swarm speed), 6 (Observations drive learning)
//!
//! # Architecture
//!
//! ```text
//! Agents → Local Training → Byzantine-Robust Aggregation → Global Model → MAPE-K
//!                                        ↓
//!                            Weaver OTEL Validation
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use knhk_workflow_engine::federated::*;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create federated coordinator
//! let mut coordinator = FederatedLearningCoordinator::new(
//!     100, // num_agents
//!     MedianAggregator::new(),
//!     KLConvergenceValidator::new(),
//! );
//!
//! // Run federated learning
//! for round in 1..=1000 {
//!     let metrics = coordinator.run_federated_round().await?;
//!     if metrics.converged {
//!         println!("Converged in {} rounds!", round);
//!         break;
//!     }
//! }
//! # Ok(())
//! # }
//! ```

pub mod types;
pub mod traits;
pub mod aggregation;
pub mod convergence;
pub mod local_training;
pub mod coordinator;
pub mod mape_integration;

pub use types::{
    Experience, Gradients, AggregatedGradients, ConvergenceStatus,
    LocalTrainingMetrics, FederatedRoundMetrics, FederatedError,
};

pub use traits::{
    LocalModel, ExperienceBuffer, Optimizer,
    ByzantineRobustAggregator, ConvergenceValidator,
    LocalTrainer, FederatedCoordinator,
};

pub use aggregation::MedianAggregator;
pub use convergence::KLConvergenceValidator;
pub use local_training::LocalTrainingCoordinator;
pub use coordinator::FederatedLearningCoordinator;
