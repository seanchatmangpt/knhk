//! Quantum-Inspired Optimization for Workflow Scheduling
//!
//! This module provides quantum-inspired classical algorithms for optimal
//! workflow scheduling and resource allocation:
//!
//! - **Quantum Annealing**: Global optimization via simulated quantum tunneling
//! - **Grover Search**: Amplitude amplification for resource discovery (O(√N) speedup)
//! - **QAOA**: Quantum Approximate Optimization Algorithm for task assignment
//! - **Quantum Walk**: Graph-based dependency resolution with faster convergence
//!
//! # Performance Goals
//!
//! - Schedule 1M workflows in < 100ms
//! - Solution quality ≥95% of global optimum
//! - Memory usage O(N) where N = number of workflows
//! - Deterministic results with seeded RNG
//!
//! # Example
//!
//! ```no_run
//! use knhk_workflow_engine::quantum::*;
//!
//! # async fn example() -> Result<(), QuantumError> {
//! // Create quantum-inspired scheduler
//! let mut scheduler = QuantumScheduler::builder()
//!     .with_seed(42)
//!     .build()?;
//!
//! // Define constraints
//! scheduler.add_constraint(LatencyConstraint::new(100))?;
//! scheduler.add_constraint(CostConstraint::new(1000.0))?;
//! scheduler.add_constraint(ResourceConstraint::new(80.0))?;
//!
//! // Optimize using quantum annealing
//! let schedule = scheduler.optimize_quantum_annealing().await?;
//!
//! assert!(schedule.satisfies_constraints());
//! assert!(schedule.quality_score() >= 0.95);
//! # Ok(())
//! # }
//! ```

pub mod annealing;
pub mod constraints;
pub mod error;
pub mod grover;
pub mod qaoa;
pub mod quantum_walk;
pub mod scheduler;
pub mod types;

pub use annealing::QuantumAnnealing;
pub use constraints::{Constraint, CostConstraint, LatencyConstraint, ResourceConstraint};
pub use error::{QuantumError, QuantumResult};
pub use grover::GroverSearch;
pub use qaoa::QAOAOptimizer;
pub use quantum_walk::QuantumWalk;
pub use scheduler::{OptimizationMethod, QuantumScheduler, Schedule, SchedulerBuilder};
pub use types::{EnergyFunction, State, WorkflowTask};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quantum_module_integration() {
        let scheduler = QuantumScheduler::builder()
            .with_seed(42)
            .build()
            .expect("Failed to build scheduler");

        assert!(scheduler.is_ready());
    }
}
