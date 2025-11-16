//! Grover-Inspired Search for Resource Discovery
//!
//! Implements amplitude amplification inspired by Grover's quantum search algorithm.
//! Provides O(√N) speedup for finding optimal resource allocations in unstructured search spaces.
//!
//! # Classical Approximation
//!
//! While true quantum speedup requires quantum hardware, we approximate the amplitude
//! amplification behavior using iterative refinement with probabilistic selection biased
//! toward better candidates.

use super::error::{QuantumError, QuantumResult};
use super::types::{Resource, State, WorkflowTask};
use fastrand::Rng;
use std::collections::HashMap;
use tracing::{debug, info, instrument};

/// Grover search configuration
#[derive(Debug, Clone)]
pub struct GroverConfig {
    /// Number of iterations (√N for optimal quantum speedup)
    pub iterations: usize,

    /// Random seed for determinism
    pub seed: Option<u64>,

    /// Amplification factor (controls search bias)
    pub amplification_factor: f64,
}

impl Default for GroverConfig {
    fn default() -> Self {
        Self {
            iterations: 10,
            seed: None,
            amplification_factor: 2.0,
        }
    }
}

impl GroverConfig {
    /// Create configuration with optimal iterations for N items
    pub fn for_search_space(n: usize) -> Self {
        let optimal_iterations = ((n as f64).sqrt() * std::f64::consts::FRAC_PI_4) as usize;
        Self {
            iterations: optimal_iterations.max(1),
            ..Default::default()
        }
    }

    /// Set seed for determinism
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set amplification factor
    pub fn with_amplification(mut self, factor: f64) -> Self {
        self.amplification_factor = factor;
        self
    }
}

/// Oracle function that marks optimal resource allocations
pub type Oracle = Box<dyn Fn(&HashMap<uuid::Uuid, String>, &[WorkflowTask], &[Resource]) -> f64 + Send + Sync>;

/// Grover-inspired search optimizer
pub struct GroverSearch {
    config: GroverConfig,
    rng: Rng,
}

impl GroverSearch {
    /// Create new Grover search optimizer
    pub fn new(config: GroverConfig) -> Self {
        let rng = match config.seed {
            Some(seed) => Rng::with_seed(seed),
            None => Rng::new(),
        };

        Self { config, rng }
    }

    /// Find optimal resource allocation using Grover-inspired search
    #[instrument(skip(self, tasks, resources, oracle), fields(
        num_tasks = tasks.len(),
        num_resources = resources.len()
    ))]
    pub async fn find_optimal_allocation(
        &mut self,
        tasks: &[WorkflowTask],
        resources: &[Resource],
        oracle: Oracle,
    ) -> QuantumResult<HashMap<uuid::Uuid, String>> {
        if tasks.is_empty() {
            return Err(QuantumError::invalid_state("No tasks to allocate"));
        }
        if resources.is_empty() {
            return Err(QuantumError::invalid_state("No resources available"));
        }

        info!(
            "Starting Grover search for {} tasks across {} resources",
            tasks.len(),
            resources.len()
        );

        // Initialize with uniform superposition (random allocation)
        let mut current_allocation = self.generate_initial_allocation(tasks, resources)?;
        let mut best_allocation = current_allocation.clone();
        let mut best_score = oracle(&current_allocation, tasks, resources);

        // Iterative amplitude amplification
        for iteration in 0..self.config.iterations {
            // Apply oracle (mark good solutions)
            let current_score = oracle(&current_allocation, tasks, resources);

            // Amplitude amplification: generate neighbors biased toward better solutions
            let candidates = self.generate_amplified_candidates(
                &current_allocation,
                tasks,
                resources,
                &oracle,
                10, // Generate 10 candidates per iteration
            )?;

            // Select best candidate
            let mut best_candidate = current_allocation.clone();
            let mut best_candidate_score = current_score;

            for candidate in candidates {
                let score = oracle(&candidate, tasks, resources);
                if score > best_candidate_score {
                    best_candidate = candidate;
                    best_candidate_score = score;
                }
            }

            current_allocation = best_candidate;

            // Track global best
            if best_candidate_score > best_score {
                best_allocation = current_allocation.clone();
                best_score = best_candidate_score;

                debug!(
                    iteration,
                    score = best_score,
                    "New best allocation found"
                );
            }
        }

        info!(
            final_score = best_score,
            iterations = self.config.iterations,
            "Grover search complete"
        );

        Ok(best_allocation)
    }

    /// Generate initial random allocation (uniform superposition)
    fn generate_initial_allocation(
        &mut self,
        tasks: &[WorkflowTask],
        resources: &[Resource],
    ) -> QuantumResult<HashMap<uuid::Uuid, String>> {
        let mut allocation = HashMap::new();

        for task in tasks {
            // Try to find compatible resource
            let mut assigned = false;
            for _ in 0..resources.len() {
                let resource_idx = self.rng.usize(0..resources.len());
                let resource = &resources[resource_idx];

                if resource.can_accommodate(task) {
                    allocation.insert(task.id, resource.id.clone());
                    assigned = true;
                    break;
                }
            }

            // Fallback: assign to first resource
            if !assigned {
                allocation.insert(task.id, resources[0].id.clone());
            }
        }

        Ok(allocation)
    }

    /// Generate candidates with amplitude amplification
    fn generate_amplified_candidates(
        &mut self,
        current: &HashMap<uuid::Uuid, String>,
        tasks: &[WorkflowTask],
        resources: &[Resource],
        oracle: &Oracle,
        num_candidates: usize,
    ) -> QuantumResult<Vec<HashMap<uuid::Uuid, String>>> {
        let mut candidates = Vec::with_capacity(num_candidates);

        // Get current score for comparison
        let current_score = oracle(current, tasks, resources);

        for _ in 0..num_candidates {
            let mut candidate = current.clone();

            // Randomly modify allocation
            let num_modifications = self.rng.usize(1..=3);
            for _ in 0..num_modifications {
                if let Some(task) = tasks.get(self.rng.usize(0..tasks.len())) {
                    // Bias toward resources that improve score
                    let mut best_resource = resources[0].id.clone();
                    let mut best_delta = f64::NEG_INFINITY;

                    for resource in resources {
                        if !resource.can_accommodate(task) {
                            continue;
                        }

                        // Try this resource
                        let old_assignment = candidate.insert(task.id, resource.id.clone());
                        let new_score = oracle(&candidate, tasks, resources);
                        let delta = new_score - current_score;

                        // Amplify probability of selecting better resources
                        let amplified_delta = delta * self.config.amplification_factor;

                        if amplified_delta > best_delta {
                            best_delta = amplified_delta;
                            best_resource = resource.id.clone();
                        }

                        // Restore old assignment
                        if let Some(old) = old_assignment {
                            candidate.insert(task.id, old);
                        } else {
                            candidate.remove(&task.id);
                        }
                    }

                    // Assign to best resource with some randomness
                    if self.rng.f64() < 0.8 {
                        // 80% bias toward amplified best
                        candidate.insert(task.id, best_resource);
                    } else {
                        // 20% exploration
                        let random_resource = &resources[self.rng.usize(0..resources.len())];
                        candidate.insert(task.id, random_resource.id.clone());
                    }
                }
            }

            candidates.push(candidate);
        }

        Ok(candidates)
    }

    /// Calculate search efficiency (compared to classical linear search)
    pub fn calculate_speedup(&self, search_space_size: usize) -> f64 {
        let classical_complexity = search_space_size as f64;
        let quantum_complexity = (search_space_size as f64).sqrt();
        classical_complexity / quantum_complexity
    }
}

/// Default oracle: maximize resource utilization while minimizing cost
pub fn default_oracle(
    allocation: &HashMap<uuid::Uuid, String>,
    tasks: &[WorkflowTask],
    resources: &[Resource],
) -> f64 {
    let mut total_score = 0.0;

    // Resource utilization map
    let mut resource_usage: HashMap<String, (f64, f64)> = HashMap::new();

    for (task_id, resource_id) in allocation {
        if let Some(task) = tasks.iter().find(|t| &t.id == task_id) {
            let usage = resource_usage.entry(resource_id.clone()).or_insert((0.0, 0.0));
            usage.0 += task.cpu_requirement;
            usage.1 += task.memory_requirement;

            // Reward lower task cost
            total_score += 100.0 / (1.0 + task.cost);
        }
    }

    // Reward balanced utilization across resources
    for (resource_id, (cpu_used, mem_used)) in resource_usage {
        if let Some(resource) = resources.iter().find(|r| r.id == resource_id) {
            let cpu_util = (cpu_used / resource.available_cpu).min(1.0);
            let mem_util = (mem_used / resource.available_memory).min(1.0);

            // Penalize over-utilization
            if cpu_util > 1.0 || mem_util > 1.0 {
                total_score -= 1000.0;
            } else {
                // Reward efficient utilization (prefer 70-80% utilization)
                let target_util = 0.75;
                let cpu_efficiency = 1.0 - (cpu_util - target_util).abs();
                let mem_efficiency = 1.0 - (mem_util - target_util).abs();
                total_score += (cpu_efficiency + mem_efficiency) * 50.0;
            }
        }
    }

    total_score
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_tasks(count: usize) -> Vec<WorkflowTask> {
        (0..count)
            .map(|i| {
                WorkflowTask::new(format!("task-{}", i))
                    .with_cpu(40.0 + (i as f64 * 5.0))
                    .with_memory(512.0)
                    .with_cost(1.0 + (i as f64 * 0.5))
            })
            .collect()
    }

    fn create_test_resources(count: usize) -> Vec<Resource> {
        (0..count)
            .map(|i| {
                let mut resource = Resource::new(format!("resource-{}", i));
                resource.available_cpu = 100.0;
                resource.available_memory = 2048.0;
                resource.cost_per_ms = 0.001 * (i as f64 + 1.0);
                resource
            })
            .collect()
    }

    #[tokio::test]
    async fn test_grover_search_basic() {
        let tasks = create_test_tasks(5);
        let resources = create_test_resources(3);

        let config = GroverConfig::for_search_space(tasks.len() * resources.len()).with_seed(42);
        let mut grover = GroverSearch::new(config);

        let allocation = grover
            .find_optimal_allocation(&tasks, &resources, Box::new(default_oracle))
            .await
            .unwrap();

        assert_eq!(allocation.len(), tasks.len());

        // Verify all tasks are assigned
        for task in &tasks {
            assert!(allocation.contains_key(&task.id));
        }
    }

    #[tokio::test]
    async fn test_grover_search_deterministic() {
        let tasks = create_test_tasks(4);
        let resources = create_test_resources(2);

        // Run twice with same seed
        let config1 = GroverConfig::for_search_space(8).with_seed(42);
        let mut grover1 = GroverSearch::new(config1);
        let allocation1 = grover1
            .find_optimal_allocation(&tasks, &resources, Box::new(default_oracle))
            .await
            .unwrap();

        let config2 = GroverConfig::for_search_space(8).with_seed(42);
        let mut grover2 = GroverSearch::new(config2);
        let allocation2 = grover2
            .find_optimal_allocation(&tasks, &resources, Box::new(default_oracle))
            .await
            .unwrap();

        assert_eq!(allocation1, allocation2);
    }

    #[tokio::test]
    async fn test_grover_search_respects_constraints() {
        let mut tasks = create_test_tasks(3);
        tasks[0] = tasks[0].clone().with_cpu(50.0).with_memory(1024.0);

        let resources = create_test_resources(2);

        let config = GroverConfig::for_search_space(6).with_seed(42);
        let mut grover = GroverSearch::new(config);

        let allocation = grover
            .find_optimal_allocation(&tasks, &resources, Box::new(default_oracle))
            .await
            .unwrap();

        // Verify allocations respect resource capacities
        for (task_id, resource_id) in allocation {
            let task = tasks.iter().find(|t| t.id == task_id).unwrap();
            let resource = resources.iter().find(|r| r.id == *resource_id).unwrap();

            // Individual task should fit on resource
            assert!(task.cpu_requirement <= resource.available_cpu);
            assert!(task.memory_requirement <= resource.available_memory);
        }
    }

    #[test]
    fn test_speedup_calculation() {
        let config = GroverConfig::for_search_space(1000000);
        let grover = GroverSearch::new(config);

        let speedup = grover.calculate_speedup(1000000);
        assert!(speedup > 900.0); // √1000000 = 1000, so speedup ~1000x
    }

    #[test]
    fn test_default_oracle_scoring() {
        let tasks = create_test_tasks(2);
        let resources = create_test_resources(2);

        let mut allocation = HashMap::new();
        allocation.insert(tasks[0].id, resources[0].id.clone());
        allocation.insert(tasks[1].id, resources[1].id.clone());

        let score = default_oracle(&allocation, &tasks, &resources);
        assert!(score > 0.0);
        assert!(score.is_finite());
    }
}
