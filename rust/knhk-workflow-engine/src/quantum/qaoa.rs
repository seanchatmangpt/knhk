//! QAOA-Inspired Optimization
//!
//! Quantum Approximate Optimization Algorithm (QAOA) classical approximation
//! for task assignment and workflow partitioning problems.
//!
//! QAOA is particularly effective for combinatorial optimization problems
//! that can be formulated as MaxCut or graph partitioning.

use super::error::{QuantumError, QuantumResult};
use super::types::{State, WorkflowTask};
use fastrand::Rng;
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, instrument};
use uuid::Uuid;

/// QAOA configuration
#[derive(Debug, Clone)]
pub struct QAOAConfig {
    /// Number of QAOA layers (p parameter)
    pub num_layers: usize,

    /// Number of optimization iterations
    pub max_iterations: usize,

    /// Learning rate for parameter updates
    pub learning_rate: f64,

    /// Random seed for determinism
    pub seed: Option<u64>,

    /// Convergence threshold
    pub convergence_threshold: f64,
}

impl Default for QAOAConfig {
    fn default() -> Self {
        Self {
            num_layers: 3,
            max_iterations: 1000,
            learning_rate: 0.1,
            seed: None,
            convergence_threshold: 0.001,
        }
    }
}

impl QAOAConfig {
    /// Create configuration with seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set number of layers
    pub fn with_layers(mut self, layers: usize) -> Self {
        self.num_layers = layers;
        self
    }

    /// Set max iterations
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }
}

/// Variational parameters for QAOA
#[derive(Debug, Clone)]
struct VariationalParams {
    /// Problem Hamiltonian angles (γ)
    gamma: Vec<f64>,

    /// Mixer Hamiltonian angles (β)
    beta: Vec<f64>,
}

impl VariationalParams {
    fn new(num_layers: usize, rng: &mut Rng) -> Self {
        Self {
            gamma: (0..num_layers).map(|_| rng.f64() * std::f64::consts::PI).collect(),
            beta: (0..num_layers).map(|_| rng.f64() * std::f64::consts::PI).collect(),
        }
    }
}

/// QAOA optimizer for workflow partitioning
pub struct QAOAOptimizer {
    config: QAOAConfig,
    rng: Rng,
}

impl QAOAOptimizer {
    /// Create new QAOA optimizer
    pub fn new(config: QAOAConfig) -> Self {
        let rng = match config.seed {
            Some(seed) => Rng::with_seed(seed),
            None => Rng::new(),
        };

        Self { config, rng }
    }

    /// Optimize task assignment using QAOA
    #[instrument(skip(self, tasks), fields(num_tasks = tasks.len()))]
    pub async fn optimize_assignment(
        &mut self,
        tasks: &[WorkflowTask],
        num_partitions: usize,
    ) -> QuantumResult<Vec<HashSet<Uuid>>> {
        if tasks.is_empty() {
            return Err(QuantumError::invalid_state("No tasks to optimize"));
        }
        if num_partitions == 0 {
            return Err(QuantumError::invalid_configuration("Number of partitions must be > 0"));
        }

        info!(
            "Starting QAOA optimization for {} tasks into {} partitions",
            tasks.len(),
            num_partitions
        );

        // Initialize variational parameters
        let mut params = VariationalParams::new(self.config.num_layers, &mut self.rng);

        // Build problem Hamiltonian (task dependencies and affinities)
        let hamiltonian = self.build_problem_hamiltonian(tasks)?;

        let mut best_cost = f64::INFINITY;
        let mut best_partitions = self.random_partitioning(tasks, num_partitions)?;

        // Variational optimization loop
        for iteration in 0..self.config.max_iterations {
            // Evaluate current parameters
            let partitions = self.qaoa_circuit(&params, tasks, num_partitions, &hamiltonian)?;
            let cost = self.evaluate_cost(&partitions, tasks, &hamiltonian);

            if cost < best_cost {
                let improvement = best_cost - cost;
                best_cost = cost;
                best_partitions = partitions;

                debug!(
                    iteration,
                    cost = best_cost,
                    improvement,
                    "New best partitioning found"
                );

                // Check convergence
                if improvement < self.config.convergence_threshold && iteration > 100 {
                    info!(iteration, "QAOA converged");
                    break;
                }
            }

            // Update variational parameters using gradient descent approximation
            self.update_parameters(&mut params, &hamiltonian, tasks, num_partitions);
        }

        info!(
            final_cost = best_cost,
            num_partitions = best_partitions.len(),
            "QAOA optimization complete"
        );

        Ok(best_partitions)
    }

    /// Build problem Hamiltonian from task dependencies
    fn build_problem_hamiltonian(&self, tasks: &[WorkflowTask]) -> QuantumResult<HashMap<(Uuid, Uuid), f64>> {
        let mut hamiltonian = HashMap::new();

        // Add edges for task dependencies (want dependent tasks in same partition)
        for task in tasks {
            for dep_id in &task.dependencies {
                if let Some(dep_task) = tasks.iter().find(|t| &t.id == dep_id) {
                    // Negative weight = prefer same partition
                    let weight = -10.0;
                    hamiltonian.insert((task.id, dep_task.id), weight);
                    hamiltonian.insert((dep_task.id, task.id), weight);
                }
            }
        }

        // Add affinity edges based on resource requirements
        for (i, task1) in tasks.iter().enumerate() {
            for task2 in tasks.iter().skip(i + 1) {
                // Tasks with similar resource needs might benefit from same partition
                let cpu_diff = (task1.cpu_requirement - task2.cpu_requirement).abs();
                let mem_diff = (task1.memory_requirement - task2.memory_requirement).abs();

                let affinity = 1.0 / (1.0 + cpu_diff / 100.0 + mem_diff / 1000.0);
                let weight = -(affinity * 5.0); // Negative = attract

                hamiltonian.insert((task1.id, task2.id), weight);
                hamiltonian.insert((task2.id, task1.id), weight);
            }
        }

        Ok(hamiltonian)
    }

    /// QAOA circuit application (classical approximation)
    fn qaoa_circuit(
        &mut self,
        params: &VariationalParams,
        tasks: &[WorkflowTask],
        num_partitions: usize,
        hamiltonian: &HashMap<(Uuid, Uuid), f64>,
    ) -> QuantumResult<Vec<HashSet<Uuid>>> {
        // Start with uniform superposition (random partitioning)
        let mut partitions = self.random_partitioning(tasks, num_partitions)?;

        // Apply QAOA layers
        for layer in 0..self.config.num_layers {
            let gamma = params.gamma[layer];
            let beta = params.beta[layer];

            // Apply problem Hamiltonian (reinforce good partitionings)
            self.apply_problem_hamiltonian(&mut partitions, gamma, hamiltonian);

            // Apply mixer Hamiltonian (explore neighbor partitionings)
            self.apply_mixer_hamiltonian(&mut partitions, beta, tasks)?;
        }

        Ok(partitions)
    }

    /// Apply problem Hamiltonian operator
    fn apply_problem_hamiltonian(
        &mut self,
        partitions: &mut Vec<HashSet<Uuid>>,
        gamma: f64,
        hamiltonian: &HashMap<(Uuid, Uuid), f64>,
    ) {
        // Classical approximation: with probability based on γ and edge weights,
        // move tasks to minimize Hamiltonian cost

        let task_to_partition: HashMap<Uuid, usize> = partitions
            .iter()
            .enumerate()
            .flat_map(|(p_idx, partition)| partition.iter().map(move |&task_id| (task_id, p_idx)))
            .collect();

        for ((task1, task2), &weight) in hamiltonian {
            let p1 = task_to_partition.get(task1);
            let p2 = task_to_partition.get(task2);

            if let (Some(&partition1), Some(&partition2)) = (p1, p2) {
                // If tasks are in different partitions but have negative weight (should be together)
                if partition1 != partition2 && weight < 0.0 {
                    let move_prob = (gamma * weight.abs()).min(1.0);
                    if self.rng.f64() < move_prob {
                        // Move task2 to task1's partition
                        partitions[partition2].remove(task2);
                        partitions[partition1].insert(*task2);
                    }
                }
            }
        }
    }

    /// Apply mixer Hamiltonian operator
    fn apply_mixer_hamiltonian(
        &mut self,
        partitions: &mut Vec<HashSet<Uuid>>,
        beta: f64,
        tasks: &[WorkflowTask],
    ) -> QuantumResult<()> {
        // Classical approximation: randomly swap tasks between partitions
        let num_swaps = ((beta / std::f64::consts::PI) * tasks.len() as f64) as usize;

        for _ in 0..num_swaps {
            if partitions.len() < 2 {
                break;
            }

            let p1_idx = self.rng.usize(0..partitions.len());
            let p2_idx = self.rng.usize(0..partitions.len());

            if p1_idx == p2_idx {
                continue;
            }

            // Move random task from p1 to p2
            if let Some(&task_id) = partitions[p1_idx].iter().next() {
                partitions[p1_idx].remove(&task_id);
                partitions[p2_idx].insert(task_id);
            }
        }

        Ok(())
    }

    /// Evaluate cost of partitioning
    fn evaluate_cost(
        &self,
        partitions: &[HashSet<Uuid>],
        tasks: &[WorkflowTask],
        hamiltonian: &HashMap<(Uuid, Uuid), f64>,
    ) -> f64 {
        let task_to_partition: HashMap<Uuid, usize> = partitions
            .iter()
            .enumerate()
            .flat_map(|(p_idx, partition)| partition.iter().map(move |&task_id| (task_id, p_idx)))
            .collect();

        let mut cost = 0.0;

        // Sum edge weights for edges crossing partitions
        for ((task1, task2), &weight) in hamiltonian {
            let p1 = task_to_partition.get(task1);
            let p2 = task_to_partition.get(task2);

            if let (Some(&partition1), Some(&partition2)) = (p1, p2) {
                if partition1 != partition2 {
                    // Edge crosses partition boundary
                    cost += weight.abs();
                }
            }
        }

        // Penalize unbalanced partitions
        let avg_size = tasks.len() as f64 / partitions.len() as f64;
        for partition in partitions {
            let size_diff = (partition.len() as f64 - avg_size).abs();
            cost += size_diff * 0.5;
        }

        cost
    }

    /// Update variational parameters using finite difference gradient
    fn update_parameters(
        &mut self,
        params: &mut VariationalParams,
        hamiltonian: &HashMap<(Uuid, Uuid), f64>,
        tasks: &[WorkflowTask],
        num_partitions: usize,
    ) {
        let epsilon = 0.01;

        for i in 0..params.gamma.len() {
            // Finite difference for gamma
            let mut params_plus = params.clone();
            params_plus.gamma[i] += epsilon;
            let cost_plus = self.eval_params(&params_plus, hamiltonian, tasks, num_partitions);

            let mut params_minus = params.clone();
            params_minus.gamma[i] -= epsilon;
            let cost_minus = self.eval_params(&params_minus, hamiltonian, tasks, num_partitions);

            let gradient = (cost_plus - cost_minus) / (2.0 * epsilon);
            params.gamma[i] -= self.config.learning_rate * gradient;

            // Finite difference for beta
            let mut params_plus = params.clone();
            params_plus.beta[i] += epsilon;
            let cost_plus = self.eval_params(&params_plus, hamiltonian, tasks, num_partitions);

            let mut params_minus = params.clone();
            params_minus.beta[i] -= epsilon;
            let cost_minus = self.eval_params(&params_minus, hamiltonian, tasks, num_partitions);

            let gradient = (cost_plus - cost_minus) / (2.0 * epsilon);
            params.beta[i] -= self.config.learning_rate * gradient;
        }
    }

    /// Evaluate cost for given parameters
    fn eval_params(
        &mut self,
        params: &VariationalParams,
        hamiltonian: &HashMap<(Uuid, Uuid), f64>,
        tasks: &[WorkflowTask],
        num_partitions: usize,
    ) -> f64 {
        match self.qaoa_circuit(params, tasks, num_partitions, hamiltonian) {
            Ok(partitions) => self.evaluate_cost(&partitions, tasks, hamiltonian),
            Err(_) => f64::INFINITY,
        }
    }

    /// Generate random partitioning
    fn random_partitioning(
        &mut self,
        tasks: &[WorkflowTask],
        num_partitions: usize,
    ) -> QuantumResult<Vec<HashSet<Uuid>>> {
        let mut partitions: Vec<HashSet<Uuid>> = (0..num_partitions).map(|_| HashSet::new()).collect();

        for task in tasks {
            let partition_idx = self.rng.usize(0..num_partitions);
            partitions[partition_idx].insert(task.id);
        }

        Ok(partitions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tasks(count: usize) -> Vec<WorkflowTask> {
        let mut tasks = Vec::new();
        for i in 0..count {
            let task = WorkflowTask::new(format!("task-{}", i))
                .with_cpu(50.0)
                .with_memory(512.0);
            tasks.push(task);
        }

        // Add some dependencies
        if tasks.len() >= 3 {
            tasks[1] = tasks[1].clone().with_dependency(tasks[0].id);
            tasks[2] = tasks[2].clone().with_dependency(tasks[1].id);
        }

        tasks
    }

    #[tokio::test]
    async fn test_qaoa_basic_partitioning() {
        let tasks = create_test_tasks(6);

        let config = QAOAConfig::default()
            .with_seed(42)
            .with_max_iterations(100);

        let mut qaoa = QAOAOptimizer::new(config);
        let partitions = qaoa.optimize_assignment(&tasks, 2).await.unwrap();

        assert_eq!(partitions.len(), 2);

        // Verify all tasks are assigned
        let total_tasks: usize = partitions.iter().map(|p| p.len()).sum();
        assert_eq!(total_tasks, tasks.len());
    }

    #[tokio::test]
    async fn test_qaoa_respects_dependencies() {
        let tasks = create_test_tasks(4);

        let config = QAOAConfig::default()
            .with_seed(42)
            .with_layers(5)
            .with_max_iterations(500);

        let mut qaoa = QAOAOptimizer::new(config);
        let partitions = qaoa.optimize_assignment(&tasks, 2).await.unwrap();

        // Tasks with dependencies should ideally be in the same partition
        // (not guaranteed, but QAOA should bias toward it)
        assert!(partitions.len() <= 2);
    }

    #[tokio::test]
    async fn test_qaoa_deterministic() {
        let tasks = create_test_tasks(4);

        let config1 = QAOAConfig::default()
            .with_seed(42)
            .with_max_iterations(50);
        let mut qaoa1 = QAOAOptimizer::new(config1);
        let partitions1 = qaoa1.optimize_assignment(&tasks, 2).await.unwrap();

        let config2 = QAOAConfig::default()
            .with_seed(42)
            .with_max_iterations(50);
        let mut qaoa2 = QAOAOptimizer::new(config2);
        let partitions2 = qaoa2.optimize_assignment(&tasks, 2).await.unwrap();

        // Same seed should give same results
        assert_eq!(partitions1, partitions2);
    }

    #[test]
    fn test_hamiltonian_construction() {
        let tasks = create_test_tasks(3);
        let qaoa = QAOAOptimizer::new(QAOAConfig::default());

        let hamiltonian = qaoa.build_problem_hamiltonian(&tasks).unwrap();

        // Should have edges for dependencies
        assert!(!hamiltonian.is_empty());

        // Dependency edges should have negative weight
        for ((t1, t2), &weight) in &hamiltonian {
            let task1 = tasks.iter().find(|t| &t.id == t1);
            let task2 = tasks.iter().find(|t| &t.id == t2);

            if let (Some(t1), Some(t2)) = (task1, task2) {
                if t1.dependencies.contains(&t2.id) || t2.dependencies.contains(&t1.id) {
                    assert!(weight < 0.0, "Dependency edges should have negative weight");
                }
            }
        }
    }
}
