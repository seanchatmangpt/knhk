//! Quantum Walk-Based Scheduler
//!
//! Implements quantum walk on workflow dependency graphs for faster
//! dependency resolution and optimal execution ordering.
//!
//! Quantum walks achieve quadratically faster convergence compared to
//! classical random walks for graph traversal problems.

use super::error::{QuantumError, QuantumResult};
use super::types::WorkflowTask;
use fastrand::Rng;
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug, info, instrument};
use uuid::Uuid;

/// Quantum walk configuration
#[derive(Debug, Clone)]
pub struct QuantumWalkConfig {
    /// Number of walk iterations
    pub max_iterations: usize,

    /// Mixing parameter (quantum interference strength)
    pub mixing_param: f64,

    /// Random seed for determinism
    pub seed: Option<u64>,

    /// Convergence threshold
    pub convergence_threshold: f64,
}

impl Default for QuantumWalkConfig {
    fn default() -> Self {
        Self {
            max_iterations: 1000,
            mixing_param: 0.5,
            seed: None,
            convergence_threshold: 0.01,
        }
    }
}

impl QuantumWalkConfig {
    /// Create configuration with seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set mixing parameter
    pub fn with_mixing(mut self, mixing: f64) -> Self {
        self.mixing_param = mixing.clamp(0.0, 1.0);
        self
    }

    /// Set max iterations
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }
}

/// Quantum walk scheduler
pub struct QuantumWalk {
    config: QuantumWalkConfig,
    rng: Rng,
}

impl QuantumWalk {
    /// Create new quantum walk scheduler
    pub fn new(config: QuantumWalkConfig) -> Self {
        let rng = match config.seed {
            Some(seed) => Rng::with_seed(seed),
            None => Rng::new(),
        };

        Self { config, rng }
    }

    /// Find optimal execution order using quantum walk
    #[instrument(skip(self, tasks), fields(num_tasks = tasks.len()))]
    pub async fn find_execution_order(&mut self, tasks: &[WorkflowTask]) -> QuantumResult<Vec<Uuid>> {
        if tasks.is_empty() {
            return Err(QuantumError::invalid_state("No tasks to order"));
        }

        info!("Starting quantum walk for {} tasks", tasks.len());

        // Build dependency graph
        let graph = self.build_dependency_graph(tasks)?;

        // Initialize amplitude distribution (uniform superposition)
        let mut amplitudes = self.initialize_amplitudes(&graph);

        let mut best_order = Vec::new();
        let mut best_score = f64::NEG_INFINITY;

        // Quantum walk iterations
        for iteration in 0..self.config.max_iterations {
            // Apply mixing (quantum interference)
            self.apply_mixing(&mut amplitudes, &graph);

            // Measure (sample execution order based on amplitudes)
            let order = self.measure_execution_order(&amplitudes, &graph, tasks)?;

            // Evaluate order quality
            let score = self.evaluate_order(&order, tasks);

            if score > best_score {
                let improvement = score - best_score;
                best_score = score;
                best_order = order;

                debug!(
                    iteration,
                    score = best_score,
                    improvement,
                    "New best execution order found"
                );

                if improvement < self.config.convergence_threshold && iteration > 100 {
                    info!(iteration, "Quantum walk converged");
                    break;
                }
            }

            // Reinforce good paths (amplitude amplification)
            self.reinforce_order(&mut amplitudes, &best_order, &graph);
        }

        info!(
            final_score = best_score,
            order_length = best_order.len(),
            "Quantum walk complete"
        );

        Ok(best_order)
    }

    /// Build dependency graph from tasks
    fn build_dependency_graph(&self, tasks: &[WorkflowTask]) -> QuantumResult<HashMap<Uuid, Vec<Uuid>>> {
        let mut graph: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

        for task in tasks {
            // Add node
            graph.entry(task.id).or_insert_with(Vec::new);

            // Add edges to dependencies
            for dep_id in &task.dependencies {
                if tasks.iter().any(|t| &t.id == dep_id) {
                    graph.entry(task.id).or_insert_with(Vec::new).push(*dep_id);
                }
            }
        }

        Ok(graph)
    }

    /// Initialize uniform amplitude distribution
    fn initialize_amplitudes(&mut self, graph: &HashMap<Uuid, Vec<Uuid>>) -> HashMap<Uuid, f64> {
        let num_nodes = graph.len();
        let uniform_amplitude = 1.0 / (num_nodes as f64).sqrt();

        graph.keys().map(|&id| (id, uniform_amplitude)).collect()
    }

    /// Apply quantum mixing (interference) operator
    fn apply_mixing(&mut self, amplitudes: &mut HashMap<Uuid, f64>, graph: &HashMap<Uuid, Vec<Uuid>>) {
        let mut new_amplitudes = amplitudes.clone();

        for (node, neighbors) in graph {
            let current_amp = amplitudes.get(node).copied().unwrap_or(0.0);

            // Quantum interference with neighbors
            let neighbor_avg = if neighbors.is_empty() {
                0.0
            } else {
                let sum: f64 = neighbors
                    .iter()
                    .filter_map(|n| amplitudes.get(n).copied())
                    .sum();
                sum / neighbors.len() as f64
            };

            // Mix current amplitude with neighbor average
            let new_amp = (1.0 - self.config.mixing_param) * current_amp
                + self.config.mixing_param * neighbor_avg;

            new_amplitudes.insert(*node, new_amp);
        }

        // Normalize amplitudes
        let sum_squares: f64 = new_amplitudes.values().map(|a| a * a).sum();
        let norm = sum_squares.sqrt();

        if norm > 0.0 {
            for amp in new_amplitudes.values_mut() {
                *amp /= norm;
            }
        }

        *amplitudes = new_amplitudes;
    }

    /// Measure execution order from amplitude distribution
    fn measure_execution_order(
        &mut self,
        amplitudes: &HashMap<Uuid, f64>,
        graph: &HashMap<Uuid, Vec<Uuid>>,
        tasks: &[WorkflowTask],
    ) -> QuantumResult<Vec<Uuid>> {
        let mut order = Vec::new();
        let mut available: HashSet<Uuid> = graph.keys().copied().collect();
        let mut satisfied_deps: HashSet<Uuid> = HashSet::new();

        while !available.is_empty() {
            // Find tasks with satisfied dependencies
            let candidates: Vec<Uuid> = available
                .iter()
                .copied()
                .filter(|task_id| {
                    if let Some(task) = tasks.iter().find(|t| &t.id == task_id) {
                        task.dependencies.iter().all(|dep| satisfied_deps.contains(dep))
                    } else {
                        true
                    }
                })
                .collect();

            if candidates.is_empty() {
                // Cyclic dependency or disconnected graph - pick any available
                if let Some(&task_id) = available.iter().next() {
                    order.push(task_id);
                    available.remove(&task_id);
                    satisfied_deps.insert(task_id);
                }
                continue;
            }

            // Select task based on amplitude probability
            let selected = self.select_by_amplitude(&candidates, amplitudes)?;

            order.push(selected);
            available.remove(&selected);
            satisfied_deps.insert(selected);
        }

        Ok(order)
    }

    /// Select task based on amplitude probability
    fn select_by_amplitude(
        &mut self,
        candidates: &[Uuid],
        amplitudes: &HashMap<Uuid, f64>,
    ) -> QuantumResult<Uuid> {
        if candidates.is_empty() {
            return Err(QuantumError::invalid_state("No candidates to select"));
        }

        // Calculate probabilities (|amplitude|²)
        let mut probabilities: Vec<(Uuid, f64)> = candidates
            .iter()
            .map(|&id| {
                let amp = amplitudes.get(&id).copied().unwrap_or(0.0);
                (id, amp * amp)
            })
            .collect();

        // Normalize probabilities
        let total_prob: f64 = probabilities.iter().map(|(_, p)| p).sum();
        if total_prob > 0.0 {
            for (_, prob) in &mut probabilities {
                *prob /= total_prob;
            }
        } else {
            // Uniform if all amplitudes are zero
            let uniform = 1.0 / candidates.len() as f64;
            for (_, prob) in &mut probabilities {
                *prob = uniform;
            }
        }

        // Sample based on probabilities
        let mut cumulative = 0.0;
        let random = self.rng.f64();

        for (id, prob) in probabilities {
            cumulative += prob;
            if random <= cumulative {
                return Ok(id);
            }
        }

        // Fallback
        Ok(candidates[0])
    }

    /// Evaluate quality of execution order
    fn evaluate_order(&self, order: &[Uuid], tasks: &[WorkflowTask]) -> f64 {
        let mut score = 0.0;

        // Build position map
        let position: HashMap<Uuid, usize> = order.iter().enumerate().map(|(i, &id)| (id, i)).collect();

        // Check dependency satisfaction
        for task in tasks {
            if let Some(&task_pos) = position.get(&task.id) {
                for dep_id in &task.dependencies {
                    if let Some(&dep_pos) = position.get(dep_id) {
                        if dep_pos < task_pos {
                            // Dependency satisfied - good
                            score += 10.0;
                        } else {
                            // Dependency violated - very bad
                            score -= 100.0;
                        }
                    }
                }

                // Prefer high-priority tasks earlier
                let position_factor = 1.0 - (task_pos as f64 / order.len() as f64);
                score += task.priority as f64 * position_factor;
            }
        }

        score
    }

    /// Reinforce good execution order (update amplitudes)
    fn reinforce_order(
        &mut self,
        amplitudes: &mut HashMap<Uuid, f64>,
        order: &[Uuid],
        graph: &HashMap<Uuid, Vec<Uuid>>,
    ) {
        // Boost amplitudes for tasks in good order
        let boost_factor = 1.1;
        let decay_factor = 0.95;

        for (i, &task_id) in order.iter().enumerate() {
            // Earlier tasks get more boost
            let position_weight = 1.0 - (i as f64 / order.len() as f64);
            let boost = boost_factor + position_weight * 0.1;

            if let Some(amp) = amplitudes.get_mut(&task_id) {
                *amp *= boost;
            }

            // Slightly decay other tasks
            for &other_id in graph.keys() {
                if other_id != task_id {
                    if let Some(amp) = amplitudes.get_mut(&other_id) {
                        *amp *= decay_factor;
                    }
                }
            }
        }

        // Normalize
        let sum_squares: f64 = amplitudes.values().map(|a| a * a).sum();
        let norm = sum_squares.sqrt();

        if norm > 0.0 {
            for amp in amplitudes.values_mut() {
                *amp /= norm;
            }
        }
    }

    /// Topological sort (classical baseline for comparison)
    pub fn topological_sort(&self, tasks: &[WorkflowTask]) -> QuantumResult<Vec<Uuid>> {
        let graph = self.build_dependency_graph(tasks)?;

        // Kahn's algorithm
        let mut in_degree: HashMap<Uuid, usize> = graph.keys().map(|&id| (id, 0)).collect();

        for neighbors in graph.values() {
            for &neighbor in neighbors {
                *in_degree.entry(neighbor).or_insert(0) += 1;
            }
        }

        let mut queue: VecDeque<Uuid> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut result = Vec::new();

        while let Some(node) = queue.pop_front() {
            result.push(node);

            if let Some(neighbors) = graph.get(&node) {
                for &neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(&neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        if result.len() != graph.len() {
            Err(QuantumError::invalid_state("Cyclic dependency detected"))
        } else {
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tasks_with_deps() -> Vec<WorkflowTask> {
        let mut tasks = Vec::new();

        let task0 = WorkflowTask::new("task-0").with_priority(10);
        let task1 = WorkflowTask::new("task-1")
            .with_dependency(task0.id)
            .with_priority(5);
        let task2 = WorkflowTask::new("task-2")
            .with_dependency(task1.id)
            .with_priority(3);
        let task3 = WorkflowTask::new("task-3")
            .with_dependency(task0.id)
            .with_priority(7);

        tasks.push(task0);
        tasks.push(task1);
        tasks.push(task2);
        tasks.push(task3);

        tasks
    }

    #[tokio::test]
    async fn test_quantum_walk_basic() {
        let tasks = create_test_tasks_with_deps();

        let config = QuantumWalkConfig::default()
            .with_seed(42)
            .with_max_iterations(100);

        let mut qwalk = QuantumWalk::new(config);
        let order = qwalk.find_execution_order(&tasks).await.unwrap();

        assert_eq!(order.len(), tasks.len());

        // Verify all tasks are present
        let order_set: HashSet<_> = order.iter().collect();
        for task in &tasks {
            assert!(order_set.contains(&task.id));
        }
    }

    #[tokio::test]
    async fn test_quantum_walk_respects_dependencies() {
        let tasks = create_test_tasks_with_deps();

        let config = QuantumWalkConfig::default()
            .with_seed(42)
            .with_max_iterations(500);

        let mut qwalk = QuantumWalk::new(config);
        let order = qwalk.find_execution_order(&tasks).await.unwrap();

        // Build position map
        let position: HashMap<Uuid, usize> = order.iter().enumerate().map(|(i, &id)| (id, i)).collect();

        // Check dependencies
        for task in &tasks {
            let task_pos = position[&task.id];
            for dep_id in &task.dependencies {
                let dep_pos = position[dep_id];
                assert!(
                    dep_pos < task_pos,
                    "Task {} at position {} depends on {} at position {}",
                    task.name,
                    task_pos,
                    tasks.iter().find(|t| t.id == *dep_id).unwrap().name,
                    dep_pos
                );
            }
        }
    }

    #[tokio::test]
    async fn test_topological_sort() {
        let tasks = create_test_tasks_with_deps();
        let qwalk = QuantumWalk::new(QuantumWalkConfig::default());

        let order = qwalk.topological_sort(&tasks).unwrap();

        // Build position map
        let position: HashMap<Uuid, usize> = order.iter().enumerate().map(|(i, &id)| (id, i)).collect();

        // Verify dependencies
        for task in &tasks {
            let task_pos = position[&task.id];
            for dep_id in &task.dependencies {
                let dep_pos = position[dep_id];
                assert!(dep_pos < task_pos);
            }
        }
    }

    #[test]
    fn test_dependency_graph_construction() {
        let tasks = create_test_tasks_with_deps();
        let qwalk = QuantumWalk::new(QuantumWalkConfig::default());

        let graph = qwalk.build_dependency_graph(&tasks).unwrap();

        assert_eq!(graph.len(), tasks.len());

        // task1 depends on task0
        assert!(graph[&tasks[1].id].contains(&tasks[0].id));
        // task2 depends on task1
        assert!(graph[&tasks[2].id].contains(&tasks[1].id));
        // task3 depends on task0
        assert!(graph[&tasks[3].id].contains(&tasks[0].id));
    }

    #[test]
    fn test_evaluate_order_quality() {
        let tasks = create_test_tasks_with_deps();
        let qwalk = QuantumWalk::new(QuantumWalkConfig::default());

        // Good order: task0, task1, task2, task3
        let good_order = vec![tasks[0].id, tasks[1].id, tasks[2].id, tasks[3].id];
        let good_score = qwalk.evaluate_order(&good_order, &tasks);

        // Bad order: task2, task1, task0, task3 (violates dependencies)
        let bad_order = vec![tasks[2].id, tasks[1].id, tasks[0].id, tasks[3].id];
        let bad_score = qwalk.evaluate_order(&bad_order, &tasks);

        assert!(good_score > bad_score);
    }
}
