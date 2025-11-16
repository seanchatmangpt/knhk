//! # KNHK Autonomic - MAPE-K Self-Managing Workflows
//!
//! **Covenant 3**: Feedback Loops Run at Machine Speed
//!
//! This crate implements the complete MAPE-K (Monitor, Analyze, Plan, Execute, Knowledge)
//! autonomic feedback loop for self-managing workflows.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                                                     │
//! │  Monitor (Observe)  → Analyze (Understand)          │
//! │         ↑                      ↓                    │
//! │         └──────────────────────┘                    │
//! │                                                     │
//! │  Execute (Act)     ← Plan (Decide)                 │
//! │         ↑                      ↓                    │
//! │         └──────────────────────┘                    │
//! │                                                     │
//! │           Knowledge Base (Learn)                    │
//! │           - Patterns learned                        │
//! │           - Successes recorded                      │
//! │           - Predictions trained                     │
//! │           - Policies refined                        │
//! │                                                     │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Self-Management Properties
//!
//! - **Self-Healing**: Detects failures and recovers automatically
//! - **Self-Optimizing**: Monitors performance and improves continuously
//! - **Self-Configuring**: Adapts to changing conditions dynamically
//! - **Self-Protecting**: Detects threats and protects automatically
//! - **Self-Learning**: Learns from experience and improves decisions
//!
//! ## Doctrine Alignment
//!
//! **Principle**: MAPE-K (Monitor, Analyze, Plan, Execute, Knowledge)
//! **Covenant**: Covenant 3 - Feedback Loops Run at Machine Speed
//!
//! ### Critical Constraints
//!
//! - **Latency**: Hot path operations ≤ 8 ticks (Chatman Constant)
//! - **Autonomy**: No human approval in critical path
//! - **Mechanistic**: All policies are SPARQL queries (not implicit logic)
//! - **Observable**: All decisions emit telemetry validated by Weaver
//! - **Persistent**: Knowledge survives across workflow executions
//!
//! ## Example
//!
//! ```rust,no_run
//! use knhk_autonomic::{AutonomicController, Config};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create autonomic controller
//!     let config = Config::default()
//!         .with_loop_frequency(std::time::Duration::from_secs(5))
//!         .with_knowledge_path("./knowledge.db");
//!
//!     let controller = AutonomicController::new(config).await?;
//!
//!     // Start MAPE-K loop
//!     controller.start().await?;
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod monitor;
pub mod analyze;
pub mod planner;
pub mod execute;
pub mod knowledge;
pub mod hooks;
pub mod controller;
pub mod types;
pub mod error;

// Re-export main types
pub use controller::AutonomicController;
pub use types::{
    Config, Metric, MetricType, Observation, Analysis, Policy, Action,
    Plan, FeedbackCycle, LearnedPattern, SuccessMemory,
};
pub use error::{AutonomicError, Result};

/// Autonomic system version (aligned with workspace)
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Chatman Constant: Maximum latency for hot path operations (8 ticks)
pub const CHATMAN_CONSTANT_TICKS: u64 = 8;

/// MAPE-K loop default frequency
pub const DEFAULT_LOOP_FREQUENCY_MS: u64 = 5000;
