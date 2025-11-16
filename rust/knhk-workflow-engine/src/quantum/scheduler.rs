//! Quantum-Inspired Workflow Scheduler
//!
//! Main integration layer combining all quantum-inspired algorithms
//! for optimal workflow scheduling.

use super::annealing::{AnnealingConfig, QuantumAnnealing};
use super::constraints::{Constraint, ConstraintManager};
use super::error::{QuantumError, QuantumResult};
use super::grover::{default_oracle, GroverConfig, GroverSearch};
use super::qaoa::{QAOAConfig, QAOAOptimizer};
use super::quantum_walk::{QuantumWalk, QuantumWalkConfig};
use super::types::{Resource, State, WorkflowTask};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, instrument, warn};
use uuid::Uuid;

/// Optimization method selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationMethod {
    /// Quantum annealing (global optimization)
    QuantumAnnealing,

    /// Grover search (resource discovery)
    GroverSearch,

    /// QAOA (task partitioning)
    QAOA,

    /// Quantum walk (dependency resolution)
    QuantumWalk,

    /// Hybrid (combines multiple methods)
    Hybrid,

    /// Auto-select based on problem characteristics
    Auto,
}

/// Complete workflow schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    /// Execution state
    pub state: State,

    /// Resource allocations
    pub allocations: HashMap<Uuid, String>,

    /// Optimization method used
    pub method: OptimizationMethod,

    /// Optimization time in milliseconds
    pub optimization_time_ms: u64,

    /// Quality score (0.0-1.0, higher is better)
    pub quality_score: f64,

    /// Whether all constraints are satisfied
    pub constraints_satisfied: bool,
}

impl Schedule {
    /// Create new schedule
    pub fn new(state: State, method: OptimizationMethod) -> Self {
        Self {
            state,
            allocations: HashMap::new(),
            method,
            optimization_time_ms: 0,
            quality_score: 0.0,
            constraints_satisfied: false,
        }
    }

    /// Check if constraints are satisfied
    pub fn satisfies_constraints(&self) -> bool {
        self.constraints_satisfied
    }

    /// Check if schedule is optimal within threshold
    pub fn is_optimal_within(&self, threshold: f64) -> bool {
        self.quality_score >= (1.0 - threshold)
    }
}

/// Builder for quantum scheduler
pub struct SchedulerBuilder {
    constraints: ConstraintManager,
    resources: Vec<Resource>,
    seed: Option<u64>,
    method: OptimizationMethod,
}

impl SchedulerBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            constraints: ConstraintManager::new(),
            resources: Vec::new(),
            seed: None,
            method: OptimizationMethod::Auto,
        }
    }

    /// Set random seed for determinism
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set optimization method
    pub fn with_method(mut self, method: OptimizationMethod) -> Self {
        self.method = method;
        self
    }

    /// Add constraint
    pub fn with_constraint(mut self, constraint: Box<dyn Constraint>) -> Self {
        let _ = self.constraints.add_constraint(constraint);
        self
    }

    /// Add resource
    pub fn with_resource(mut self, resource: Resource) -> Self {
        self.resources.push(resource);
        self
    }

    /// Build scheduler
    pub fn build(self) -> QuantumResult<QuantumScheduler> {
        if self.resources.is_empty() {
            // Add default resources
            let default_resources = vec![
                Resource::new("resource-0"),
                Resource::new("resource-1"),
                Resource::new("resource-2"),
                Resource::new("resource-3"),
            ];

            Ok(QuantumScheduler {
                constraints: Arc::new(self.constraints),
                resources: default_resources,
                seed: self.seed,
                method: self.method,
            })
        } else {
            Ok(QuantumScheduler {
                constraints: Arc::new(self.constraints),
                resources: self.resources,
                seed: self.seed,
                method: self.method,
            })
        }
    }
}

impl Default for SchedulerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Main quantum-inspired scheduler
pub struct QuantumScheduler {
    constraints: Arc<ConstraintManager>,
    resources: Vec<Resource>,
    seed: Option<u64>,
    method: OptimizationMethod,
}

impl QuantumScheduler {
    /// Create new builder
    pub fn builder() -> SchedulerBuilder {
        SchedulerBuilder::new()
    }

    /// Add constraint
    pub fn add_constraint(&mut self, constraint: Box<dyn Constraint>) -> QuantumResult<()> {
        Arc::get_mut(&mut self.constraints)
            .ok_or_else(|| QuantumError::internal("Cannot modify constraints"))?
            .add_constraint(constraint)
    }

    /// Check if scheduler is ready
    pub fn is_ready(&self) -> bool {
        !self.resources.is_empty()
    }

    /// Optimize using quantum annealing
    #[instrument(skip(self, tasks), fields(num_tasks = tasks.len()))]
    pub async fn optimize_quantum_annealing(&self, tasks: &[WorkflowTask]) -> QuantumResult<Schedule> {
        let start = Instant::now();

        info!("Starting quantum annealing optimization");

        let config = match self.seed {
            Some(seed) => AnnealingConfig::with_seed(seed),
            None => AnnealingConfig::default(),
        };

        let mut annealer = QuantumAnnealing::new(config, Arc::clone(&self.constraints));
        let state = annealer.optimize(tasks).await?;

        let elapsed = start.elapsed().as_millis() as u64;

        let mut schedule = Schedule::new(state.clone(), OptimizationMethod::QuantumAnnealing);
        schedule.optimization_time_ms = elapsed;
        schedule.quality_score = state.quality_score();
        schedule.constraints_satisfied = self.constraints.all_satisfied(&state, tasks);
        schedule.allocations = state.assignments.clone();

        info!(
            elapsed_ms = elapsed,
            quality = schedule.quality_score,
            "Quantum annealing complete"
        );

        Ok(schedule)
    }

    /// Optimize using Grover search
    #[instrument(skip(self, tasks), fields(num_tasks = tasks.len()))]
    pub async fn optimize_grover_search(&self, tasks: &[WorkflowTask]) -> QuantumResult<Schedule> {
        let start = Instant::now();

        info!("Starting Grover search optimization");

        let search_space = tasks.len() * self.resources.len();
        let mut config = GroverConfig::for_search_space(search_space);
        if let Some(seed) = self.seed {
            config = config.with_seed(seed);
        }

        let mut grover = GroverSearch::new(config);
        let allocations = grover
            .find_optimal_allocation(tasks, &self.resources, Box::new(default_oracle))
            .await?;

        // Build state from allocations
        let mut state = State::new();
        state.assignments = allocations.clone();
        state.execution_order = tasks.iter().map(|t| t.id).collect();

        // Calculate energy and satisfaction
        let mut total_cost = 0.0;
        for task in tasks {
            total_cost += task.cost;
        }
        state.energy = total_cost + self.constraints.total_penalty(&state, tasks);
        state.constraint_satisfaction = if self.constraints.all_satisfied(&state, tasks) {
            1.0
        } else {
            let penalty = self.constraints.total_penalty(&state, tasks);
            (1.0 / (1.0 + penalty)).max(0.0)
        };

        let elapsed = start.elapsed().as_millis() as u64;

        let mut schedule = Schedule::new(state.clone(), OptimizationMethod::GroverSearch);
        schedule.optimization_time_ms = elapsed;
        schedule.quality_score = state.quality_score();
        schedule.constraints_satisfied = self.constraints.all_satisfied(&state, tasks);
        schedule.allocations = allocations;

        info!(
            elapsed_ms = elapsed,
            quality = schedule.quality_score,
            "Grover search complete"
        );

        Ok(schedule)
    }

    /// Optimize using QAOA
    #[instrument(skip(self, tasks), fields(num_tasks = tasks.len()))]
    pub async fn optimize_qaoa(&self, tasks: &[WorkflowTask], num_partitions: usize) -> QuantumResult<Schedule> {
        let start = Instant::now();

        info!("Starting QAOA optimization");

        let mut config = QAOAConfig::default();
        if let Some(seed) = self.seed {
            config = config.with_seed(seed);
        }

        let mut qaoa = QAOAOptimizer::new(config);
        let partitions = qaoa.optimize_assignment(tasks, num_partitions).await?;

        // Build state from partitions
        let mut state = State::new();
        let mut allocations = HashMap::new();

        for (partition_idx, partition) in partitions.iter().enumerate() {
            let resource_id = if partition_idx < self.resources.len() {
                self.resources[partition_idx].id.clone()
            } else {
                format!("resource-{}", partition_idx)
            };

            for &task_id in partition {
                allocations.insert(task_id, resource_id.clone());
                state.execution_order.push(task_id);
            }
        }

        state.assignments = allocations.clone();

        // Calculate metrics
        let mut total_cost = 0.0;
        for task in tasks {
            total_cost += task.cost;
        }
        state.energy = total_cost + self.constraints.total_penalty(&state, tasks);
        state.constraint_satisfaction = if self.constraints.all_satisfied(&state, tasks) {
            1.0
        } else {
            let penalty = self.constraints.total_penalty(&state, tasks);
            (1.0 / (1.0 + penalty)).max(0.0)
        };

        let elapsed = start.elapsed().as_millis() as u64;

        let mut schedule = Schedule::new(state.clone(), OptimizationMethod::QAOA);
        schedule.optimization_time_ms = elapsed;
        schedule.quality_score = state.quality_score();
        schedule.constraints_satisfied = self.constraints.all_satisfied(&state, tasks);
        schedule.allocations = allocations;

        info!(
            elapsed_ms = elapsed,
            quality = schedule.quality_score,
            "QAOA complete"
        );

        Ok(schedule)
    }

    /// Optimize using quantum walk
    #[instrument(skip(self, tasks), fields(num_tasks = tasks.len()))]
    pub async fn optimize_quantum_walk(&self, tasks: &[WorkflowTask]) -> QuantumResult<Schedule> {
        let start = Instant::now();

        info!("Starting quantum walk optimization");

        let mut config = QuantumWalkConfig::default();
        if let Some(seed) = self.seed {
            config = config.with_seed(seed);
        }

        let mut qwalk = QuantumWalk::new(config);
        let execution_order = qwalk.find_execution_order(tasks).await?;

        // Build state
        let mut state = State::new();
        state.execution_order = execution_order;

        // Simple round-robin allocation
        let mut allocations = HashMap::new();
        for (idx, &task_id) in state.execution_order.iter().enumerate() {
            let resource_idx = idx % self.resources.len();
            allocations.insert(task_id, self.resources[resource_idx].id.clone());
        }
        state.assignments = allocations.clone();

        // Calculate metrics
        let mut total_cost = 0.0;
        for task in tasks {
            total_cost += task.cost;
        }
        state.energy = total_cost + self.constraints.total_penalty(&state, tasks);
        state.constraint_satisfaction = if self.constraints.all_satisfied(&state, tasks) {
            1.0
        } else {
            let penalty = self.constraints.total_penalty(&state, tasks);
            (1.0 / (1.0 + penalty)).max(0.0)
        };

        let elapsed = start.elapsed().as_millis() as u64;

        let mut schedule = Schedule::new(state.clone(), OptimizationMethod::QuantumWalk);
        schedule.optimization_time_ms = elapsed;
        schedule.quality_score = state.quality_score();
        schedule.constraints_satisfied = self.constraints.all_satisfied(&state, tasks);
        schedule.allocations = allocations;

        info!(
            elapsed_ms = elapsed,
            quality = schedule.quality_score,
            "Quantum walk complete"
        );

        Ok(schedule)
    }

    /// Hybrid optimization (combines multiple methods)
    #[instrument(skip(self, tasks), fields(num_tasks = tasks.len()))]
    pub async fn optimize_hybrid(&self, tasks: &[WorkflowTask]) -> QuantumResult<Schedule> {
        info!("Starting hybrid optimization");

        // Run multiple methods in parallel
        let (annealing_result, grover_result, qwalk_result) = tokio::join!(
            self.optimize_quantum_annealing(tasks),
            self.optimize_grover_search(tasks),
            self.optimize_quantum_walk(tasks),
        );

        // Select best result
        let mut best_schedule = annealing_result?;

        if let Ok(grover_schedule) = grover_result {
            if grover_schedule.quality_score > best_schedule.quality_score {
                best_schedule = grover_schedule;
            }
        }

        if let Ok(qwalk_schedule) = qwalk_result {
            if qwalk_schedule.quality_score > best_schedule.quality_score {
                best_schedule = qwalk_schedule;
            }
        }

        best_schedule.method = OptimizationMethod::Hybrid;

        info!(
            quality = best_schedule.quality_score,
            method = ?best_schedule.method,
            "Hybrid optimization complete"
        );

        Ok(best_schedule)
    }

    /// Auto-select optimization method based on problem characteristics
    pub async fn optimize(&self, tasks: &[WorkflowTask]) -> QuantumResult<Schedule> {
        let method = match self.method {
            OptimizationMethod::Auto => self.select_method(tasks),
            method => method,
        };

        match method {
            OptimizationMethod::QuantumAnnealing => self.optimize_quantum_annealing(tasks).await,
            OptimizationMethod::GroverSearch => self.optimize_grover_search(tasks).await,
            OptimizationMethod::QAOA => self.optimize_qaoa(tasks, 4).await,
            OptimizationMethod::QuantumWalk => self.optimize_quantum_walk(tasks).await,
            OptimizationMethod::Hybrid => self.optimize_hybrid(tasks).await,
            OptimizationMethod::Auto => {
                warn!("Auto method should have been resolved");
                self.optimize_quantum_annealing(tasks).await
            }
        }
    }

    /// Select best method based on problem characteristics
    fn select_method(&self, tasks: &[WorkflowTask]) -> OptimizationMethod {
        let num_tasks = tasks.len();
        let has_dependencies = tasks.iter().any(|t| !t.dependencies.is_empty());
        let has_complex_constraints = self.constraints.len() > 2;

        if has_dependencies && num_tasks > 50 {
            OptimizationMethod::QuantumWalk
        } else if has_complex_constraints {
            OptimizationMethod::QuantumAnnealing
        } else if num_tasks > 100 {
            OptimizationMethod::Hybrid
        } else {
            OptimizationMethod::GroverSearch
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quantum::constraints::{CostConstraint, LatencyConstraint};

    fn create_test_tasks(count: usize) -> Vec<WorkflowTask> {
        (0..count)
            .map(|i| {
                WorkflowTask::new(format!("task-{}", i))
                    .with_duration(50)
                    .with_cost(10.0)
                    .with_cpu(50.0)
            })
            .collect()
    }

    #[tokio::test]
    async fn test_scheduler_builder() {
        let scheduler = QuantumScheduler::builder()
            .with_seed(42)
            .with_constraint(Box::new(LatencyConstraint::new(1000)))
            .with_constraint(Box::new(CostConstraint::new(200.0)))
            .build()
            .unwrap();

        assert!(scheduler.is_ready());
    }

    #[tokio::test]
    async fn test_quantum_annealing_optimization() {
        let tasks = create_test_tasks(10);

        let scheduler = QuantumScheduler::builder()
            .with_seed(42)
            .with_method(OptimizationMethod::QuantumAnnealing)
            .build()
            .unwrap();

        let schedule = scheduler.optimize(&tasks).await.unwrap();

        assert_eq!(schedule.method, OptimizationMethod::QuantumAnnealing);
        assert!(schedule.quality_score >= 0.0);
        assert!(schedule.optimization_time_ms > 0);
    }

    #[tokio::test]
    async fn test_grover_search_optimization() {
        let tasks = create_test_tasks(5);

        let scheduler = QuantumScheduler::builder()
            .with_seed(42)
            .with_method(OptimizationMethod::GroverSearch)
            .build()
            .unwrap();

        let schedule = scheduler.optimize(&tasks).await.unwrap();

        assert_eq!(schedule.method, OptimizationMethod::GroverSearch);
        assert_eq!(schedule.allocations.len(), tasks.len());
    }

    #[tokio::test]
    async fn test_hybrid_optimization() {
        let tasks = create_test_tasks(8);

        let scheduler = QuantumScheduler::builder()
            .with_seed(42)
            .with_method(OptimizationMethod::Hybrid)
            .with_constraint(Box::new(CostConstraint::new(150.0)))
            .build()
            .unwrap();

        let schedule = scheduler.optimize(&tasks).await.unwrap();

        assert_eq!(schedule.method, OptimizationMethod::Hybrid);
        assert!(schedule.quality_score >= 0.0);
    }

    #[tokio::test]
    async fn test_auto_method_selection() {
        let tasks = create_test_tasks(20);

        let scheduler = QuantumScheduler::builder()
            .with_seed(42)
            .with_method(OptimizationMethod::Auto)
            .build()
            .unwrap();

        let schedule = scheduler.optimize(&tasks).await.unwrap();

        // Should automatically select a method
        assert_ne!(schedule.method, OptimizationMethod::Auto);
    }
}
