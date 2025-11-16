//! Quantum Annealing Simulator
//!
//! Simulates quantum annealing for global optimization of workflow schedules.
//! Uses simulated annealing with quantum tunneling probability to escape local minima.
//!
//! # Algorithm
//!
//! 1. Start with random initial state at high temperature
//! 2. Calculate energy: E(state) = cost(state) + λ × penalties(state)
//! 3. Generate neighbor state by random modification
//! 4. Accept/reject based on Metropolis criterion with tunneling
//! 5. Cool down temperature according to schedule
//! 6. Repeat until temperature freezes or convergence

use super::constraints::ConstraintManager;
use super::error::{QuantumError, QuantumResult};
use super::types::{State, Temperature, WorkflowTask};
use fastrand::Rng;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

/// Quantum annealing configuration
#[derive(Debug, Clone)]
pub struct AnnealingConfig {
    /// Initial temperature
    pub initial_temp: f64,

    /// Final temperature (stopping condition)
    pub final_temp: f64,

    /// Cooling rate (0.0-1.0, typically 0.95-0.99)
    pub cooling_rate: f64,

    /// Maximum iterations
    pub max_iterations: usize,

    /// Constraint penalty weight (λ)
    pub penalty_weight: f64,

    /// Random seed for deterministic results
    pub seed: Option<u64>,

    /// Tunneling probability factor
    pub tunneling_factor: f64,
}

impl Default for AnnealingConfig {
    fn default() -> Self {
        Self {
            initial_temp: 1000.0,
            final_temp: 0.1,
            cooling_rate: 0.95,
            max_iterations: 10_000,
            penalty_weight: 10.0,
            seed: None,
            tunneling_factor: 0.1,
        }
    }
}

impl AnnealingConfig {
    /// Create new configuration with seed for determinism
    pub fn with_seed(seed: u64) -> Self {
        Self {
            seed: Some(seed),
            ..Default::default()
        }
    }

    /// Set initial temperature
    pub fn initial_temp(mut self, temp: f64) -> Self {
        self.initial_temp = temp;
        self
    }

    /// Set final temperature
    pub fn final_temp(mut self, temp: f64) -> Self {
        self.final_temp = temp;
        self
    }

    /// Set cooling rate
    pub fn cooling_rate(mut self, rate: f64) -> Self {
        self.cooling_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Set maximum iterations
    pub fn max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }
}

/// Quantum annealing optimizer
pub struct QuantumAnnealing {
    config: AnnealingConfig,
    rng: Rng,
    constraints: Arc<ConstraintManager>,
}

impl QuantumAnnealing {
    /// Create new quantum annealing optimizer
    pub fn new(config: AnnealingConfig, constraints: Arc<ConstraintManager>) -> Self {
        let rng = match config.seed {
            Some(seed) => Rng::with_seed(seed),
            None => Rng::new(),
        };

        Self {
            config,
            rng,
            constraints,
        }
    }

    /// Optimize workflow schedule using quantum annealing
    #[instrument(skip(self, tasks), fields(num_tasks = tasks.len()))]
    pub async fn optimize(&mut self, tasks: &[WorkflowTask]) -> QuantumResult<State> {
        if tasks.is_empty() {
            return Err(QuantumError::invalid_state("No tasks to optimize"));
        }

        info!(
            "Starting quantum annealing optimization for {} tasks",
            tasks.len()
        );

        // Initialize random state
        let mut current_state = self.generate_initial_state(tasks)?;
        let mut current_energy = self.calculate_energy(&current_state, tasks);

        let mut best_state = current_state.clone();
        let mut best_energy = current_energy;

        let mut temperature = Temperature::new(
            self.config.initial_temp,
            self.config.final_temp,
            self.config.cooling_rate,
        );

        let mut iterations = 0;
        let mut accepted = 0;
        let mut rejected = 0;

        while !temperature.is_frozen() && iterations < self.config.max_iterations {
            // Generate neighbor state
            let neighbor_state = self.generate_neighbor(&current_state, tasks)?;
            let neighbor_energy = self.calculate_energy(&neighbor_state, tasks);

            let delta_energy = neighbor_energy - current_energy;

            // Acceptance probability with quantum tunneling
            let accept_prob = self.acceptance_probability(delta_energy, temperature.value);

            // Accept or reject
            if self.should_accept(accept_prob) {
                current_state = neighbor_state;
                current_energy = neighbor_energy;
                accepted += 1;

                // Track best state
                if current_energy < best_energy {
                    best_state = current_state.clone();
                    best_energy = current_energy;

                    debug!(
                        iteration = iterations,
                        energy = best_energy,
                        temp = temperature.value,
                        "New best state found"
                    );
                }
            } else {
                rejected += 1;
            }

            // Cool down
            temperature.cool();
            iterations += 1;

            // Periodic progress logging
            if iterations % 1000 == 0 {
                debug!(
                    iteration = iterations,
                    current_energy,
                    best_energy,
                    temperature = temperature.value,
                    acceptance_rate = (accepted as f64) / (iterations as f64),
                    "Annealing progress"
                );
            }
        }

        // Update best state with final metrics
        best_state.energy = best_energy;
        best_state.constraint_satisfaction = self.calculate_constraint_satisfaction(&best_state, tasks);

        info!(
            iterations,
            final_energy = best_energy,
            constraint_satisfaction = best_state.constraint_satisfaction,
            acceptance_rate = (accepted as f64) / (iterations as f64),
            "Quantum annealing optimization complete"
        );

        Ok(best_state)
    }

    /// Generate initial random state
    fn generate_initial_state(&mut self, tasks: &[WorkflowTask]) -> QuantumResult<State> {
        let mut state = State::new();

        // Random execution order
        let mut task_ids: Vec<_> = tasks.iter().map(|t| t.id).collect();
        self.shuffle(&mut task_ids);

        state.execution_order = task_ids;

        // Simple round-robin assignment to resources
        for (idx, task_id) in state.execution_order.iter().enumerate() {
            let resource_id = format!("resource-{}", idx % 4); // 4 resources
            state.assignments.insert(*task_id, resource_id);
        }

        Ok(state)
    }

    /// Generate neighbor state by random perturbation
    fn generate_neighbor(&mut self, state: &State, tasks: &[WorkflowTask]) -> QuantumResult<State> {
        let mut neighbor = state.clone();

        // Choose perturbation type
        let perturbation_type = self.rng.usize(0..3);

        match perturbation_type {
            0 => {
                // Swap two random tasks in execution order
                if neighbor.execution_order.len() >= 2 {
                    let i = self.rng.usize(0..neighbor.execution_order.len());
                    let j = self.rng.usize(0..neighbor.execution_order.len());
                    neighbor.execution_order.swap(i, j);
                }
            }
            1 => {
                // Reassign random task to different resource
                if !neighbor.assignments.is_empty() {
                    let task_ids: Vec<_> = neighbor.assignments.keys().copied().collect();
                    let task_id = task_ids[self.rng.usize(0..task_ids.len())];
                    let new_resource = format!("resource-{}", self.rng.usize(0..4));
                    neighbor.assignments.insert(task_id, new_resource);
                }
            }
            _ => {
                // Reverse a random subsequence
                if neighbor.execution_order.len() >= 2 {
                    let start = self.rng.usize(0..neighbor.execution_order.len() - 1);
                    let end = self.rng.usize(start + 1..=neighbor.execution_order.len());
                    neighbor.execution_order[start..end].reverse();
                }
            }
        }

        Ok(neighbor)
    }

    /// Calculate energy function: E = cost + λ × penalties
    fn calculate_energy(&self, state: &State, tasks: &[WorkflowTask]) -> f64 {
        // Base cost: sum of task costs
        let base_cost: f64 = state
            .execution_order
            .iter()
            .filter_map(|task_id| tasks.iter().find(|t| &t.id == task_id))
            .map(|task| task.cost)
            .sum();

        // Constraint penalties
        let penalty = self.constraints.total_penalty(state, tasks);

        base_cost + self.config.penalty_weight * penalty
    }

    /// Calculate constraint satisfaction score
    fn calculate_constraint_satisfaction(&self, state: &State, tasks: &[WorkflowTask]) -> f64 {
        if self.constraints.all_satisfied(state, tasks) {
            1.0
        } else {
            let penalty = self.constraints.total_penalty(state, tasks);
            (1.0 / (1.0 + penalty)).max(0.0).min(1.0)
        }
    }

    /// Calculate acceptance probability with quantum tunneling
    fn acceptance_probability(&self, delta_energy: f64, temperature: f64) -> f64 {
        if delta_energy < 0.0 {
            // Always accept better states
            1.0
        } else {
            // Metropolis criterion with quantum tunneling boost
            let classical_prob = (-delta_energy / temperature).exp();
            let tunneling_boost = self.config.tunneling_factor * (-delta_energy.abs().sqrt() / temperature).exp();
            (classical_prob + tunneling_boost).min(1.0)
        }
    }

    /// Decide whether to accept based on probability
    fn should_accept(&mut self, probability: f64) -> bool {
        self.rng.f64() < probability
    }

    /// Shuffle a vector in-place
    fn shuffle<T>(&mut self, vec: &mut [T]) {
        for i in (1..vec.len()).rev() {
            let j = self.rng.usize(0..=i);
            vec.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quantum::constraints::{CostConstraint, LatencyConstraint, ResourceConstraint};

    fn create_test_tasks(count: usize) -> Vec<WorkflowTask> {
        (0..count)
            .map(|i| {
                WorkflowTask::new(format!("task-{}", i))
                    .with_duration(50 + (i as u64 * 10))
                    .with_cost(10.0 + (i as f64))
                    .with_cpu(50.0)
            })
            .collect()
    }

    #[tokio::test]
    async fn test_annealing_basic_optimization() {
        let tasks = create_test_tasks(10);

        let mut constraints = ConstraintManager::new();
        constraints
            .add_constraint(Box::new(LatencyConstraint::new(1000)))
            .unwrap();
        constraints
            .add_constraint(Box::new(CostConstraint::new(200.0)))
            .unwrap();

        let config = AnnealingConfig::with_seed(42)
            .max_iterations(1000);

        let mut annealer = QuantumAnnealing::new(config, Arc::new(constraints));
        let result = annealer.optimize(&tasks).await.unwrap();

        assert!(result.is_valid());
        assert!(result.constraint_satisfaction > 0.0);
        assert_eq!(result.execution_order.len(), tasks.len());
    }

    #[tokio::test]
    async fn test_annealing_deterministic_with_seed() {
        let tasks = create_test_tasks(5);

        let mut constraints = ConstraintManager::new();
        constraints
            .add_constraint(Box::new(CostConstraint::new(100.0)))
            .unwrap();

        // Run twice with same seed
        let config1 = AnnealingConfig::with_seed(42).max_iterations(500);
        let mut annealer1 = QuantumAnnealing::new(config1, Arc::new(constraints.clone()));
        let result1 = annealer1.optimize(&tasks).await.unwrap();

        let config2 = AnnealingConfig::with_seed(42).max_iterations(500);
        let mut annealer2 = QuantumAnnealing::new(config2, Arc::new(constraints));
        let result2 = annealer2.optimize(&tasks).await.unwrap();

        // Should produce same results
        assert_eq!(result1.execution_order, result2.execution_order);
        assert_eq!(result1.energy, result2.energy);
    }

    #[test]
    fn test_energy_calculation() {
        let tasks = create_test_tasks(3);
        let mut state = State::new();
        state.execution_order = tasks.iter().map(|t| t.id).collect();

        let constraints = ConstraintManager::new();
        let config = AnnealingConfig::default();
        let mut annealer = QuantumAnnealing::new(config, Arc::new(constraints));

        let energy = annealer.calculate_energy(&state, &tasks);
        assert!(energy > 0.0);
        assert!(energy.is_finite());
    }

    #[test]
    fn test_acceptance_probability() {
        let constraints = ConstraintManager::new();
        let config = AnnealingConfig::default();
        let mut annealer = QuantumAnnealing::new(config, Arc::new(constraints));

        // Better state (negative delta) always accepted
        let prob1 = annealer.acceptance_probability(-10.0, 100.0);
        assert_eq!(prob1, 1.0);

        // Worse state has non-zero probability
        let prob2 = annealer.acceptance_probability(10.0, 100.0);
        assert!(prob2 > 0.0 && prob2 < 1.0);

        // Higher temperature increases acceptance
        let prob3 = annealer.acceptance_probability(10.0, 1000.0);
        assert!(prob3 > prob2);
    }
}
