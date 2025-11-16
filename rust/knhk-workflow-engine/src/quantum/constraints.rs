//! Constraint system for workflow scheduling optimization

use super::error::{QuantumError, QuantumResult};
use super::types::{State, WorkflowTask};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Trait for optimization constraints
pub trait Constraint: Send + Sync + fmt::Debug {
    /// Check if state satisfies this constraint
    fn is_satisfied(&self, state: &State, tasks: &[WorkflowTask]) -> bool;

    /// Calculate penalty for violating this constraint (0.0 = satisfied, higher = worse)
    fn penalty(&self, state: &State, tasks: &[WorkflowTask]) -> f64;

    /// Get constraint name
    fn name(&self) -> &str;

    /// Get constraint weight (importance)
    fn weight(&self) -> f64 {
        1.0
    }
}

/// Latency constraint (maximum execution time)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyConstraint {
    /// Maximum allowed latency in milliseconds
    pub max_latency_ms: u64,

    /// Constraint weight
    pub weight: f64,
}

impl LatencyConstraint {
    /// Create new latency constraint
    pub fn new(max_latency_ms: u64) -> Self {
        Self {
            max_latency_ms,
            weight: 1.0,
        }
    }

    /// Set constraint weight
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    /// Calculate total latency for a state
    fn calculate_latency(&self, state: &State, tasks: &[WorkflowTask]) -> u64 {
        let mut total_latency = 0u64;

        for task_id in &state.execution_order {
            if let Some(task) = tasks.iter().find(|t| &t.id == task_id) {
                total_latency += task.estimated_duration_ms;
            }
        }

        total_latency
    }
}

impl Constraint for LatencyConstraint {
    fn is_satisfied(&self, state: &State, tasks: &[WorkflowTask]) -> bool {
        self.calculate_latency(state, tasks) <= self.max_latency_ms
    }

    fn penalty(&self, state: &State, tasks: &[WorkflowTask]) -> f64 {
        let actual_latency = self.calculate_latency(state, tasks);
        if actual_latency <= self.max_latency_ms {
            0.0
        } else {
            let violation = actual_latency - self.max_latency_ms;
            (violation as f64 / self.max_latency_ms as f64) * self.weight * 100.0
        }
    }

    fn name(&self) -> &str {
        "LatencyConstraint"
    }

    fn weight(&self) -> f64 {
        self.weight
    }
}

/// Cost constraint (maximum total cost)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostConstraint {
    /// Maximum allowed cost
    pub max_cost: f64,

    /// Constraint weight
    pub weight: f64,
}

impl CostConstraint {
    /// Create new cost constraint
    pub fn new(max_cost: f64) -> Self {
        Self {
            max_cost,
            weight: 1.0,
        }
    }

    /// Set constraint weight
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    /// Calculate total cost for a state
    fn calculate_cost(&self, state: &State, tasks: &[WorkflowTask]) -> f64 {
        let mut total_cost = 0.0;

        for task_id in &state.execution_order {
            if let Some(task) = tasks.iter().find(|t| &t.id == task_id) {
                total_cost += task.cost;
            }
        }

        total_cost
    }
}

impl Constraint for CostConstraint {
    fn is_satisfied(&self, state: &State, tasks: &[WorkflowTask]) -> bool {
        self.calculate_cost(state, tasks) <= self.max_cost
    }

    fn penalty(&self, state: &State, tasks: &[WorkflowTask]) -> f64 {
        let actual_cost = self.calculate_cost(state, tasks);
        if actual_cost <= self.max_cost {
            0.0
        } else {
            let violation = actual_cost - self.max_cost;
            (violation / self.max_cost) * self.weight * 100.0
        }
    }

    fn name(&self) -> &str {
        "CostConstraint"
    }

    fn weight(&self) -> f64 {
        self.weight
    }
}

/// Resource constraint (maximum resource utilization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConstraint {
    /// Maximum CPU utilization percentage (0.0-100.0)
    pub max_cpu: f64,

    /// Maximum memory utilization in MB
    pub max_memory: f64,

    /// Constraint weight
    pub weight: f64,
}

impl ResourceConstraint {
    /// Create new resource constraint
    pub fn new(max_cpu: f64) -> Self {
        Self {
            max_cpu: max_cpu.clamp(0.0, 100.0),
            max_memory: 8192.0, // Default 8GB
            weight: 1.0,
        }
    }

    /// Set maximum memory
    pub fn with_max_memory(mut self, max_memory: f64) -> Self {
        self.max_memory = max_memory;
        self
    }

    /// Set constraint weight
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    /// Calculate resource utilization
    fn calculate_utilization(&self, state: &State, tasks: &[WorkflowTask]) -> (f64, f64) {
        let mut total_cpu = 0.0;
        let mut total_memory = 0.0;

        for task_id in &state.execution_order {
            if let Some(task) = tasks.iter().find(|t| &t.id == task_id) {
                total_cpu += task.cpu_requirement;
                total_memory += task.memory_requirement;
            }
        }

        // Average utilization
        let count = state.execution_order.len().max(1) as f64;
        (total_cpu / count, total_memory / count)
    }
}

impl Constraint for ResourceConstraint {
    fn is_satisfied(&self, state: &State, tasks: &[WorkflowTask]) -> bool {
        let (cpu_util, mem_util) = self.calculate_utilization(state, tasks);
        cpu_util <= self.max_cpu && mem_util <= self.max_memory
    }

    fn penalty(&self, state: &State, tasks: &[WorkflowTask]) -> f64 {
        let (cpu_util, mem_util) = self.calculate_utilization(state, tasks);

        let cpu_penalty = if cpu_util > self.max_cpu {
            ((cpu_util - self.max_cpu) / self.max_cpu) * 50.0
        } else {
            0.0
        };

        let mem_penalty = if mem_util > self.max_memory {
            ((mem_util - self.max_memory) / self.max_memory) * 50.0
        } else {
            0.0
        };

        (cpu_penalty + mem_penalty) * self.weight
    }

    fn name(&self) -> &str {
        "ResourceConstraint"
    }

    fn weight(&self) -> f64 {
        self.weight
    }
}

/// Constraint manager for combining multiple constraints
#[derive(Debug)]
pub struct ConstraintManager {
    constraints: Vec<Box<dyn Constraint>>,
}

impl ConstraintManager {
    /// Create new constraint manager
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    /// Add a constraint
    pub fn add_constraint(&mut self, constraint: Box<dyn Constraint>) -> QuantumResult<()> {
        self.constraints.push(constraint);
        Ok(())
    }

    /// Check if all constraints are satisfied
    pub fn all_satisfied(&self, state: &State, tasks: &[WorkflowTask]) -> bool {
        self.constraints
            .iter()
            .all(|c| c.is_satisfied(state, tasks))
    }

    /// Calculate total penalty
    pub fn total_penalty(&self, state: &State, tasks: &[WorkflowTask]) -> f64 {
        self.constraints
            .iter()
            .map(|c| c.penalty(state, tasks))
            .sum()
    }

    /// Get constraint count
    pub fn len(&self) -> usize {
        self.constraints.len()
    }

    /// Check if manager is empty
    pub fn is_empty(&self) -> bool {
        self.constraints.is_empty()
    }
}

impl Default for ConstraintManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_task(duration_ms: u64, cost: f64, cpu: f64) -> WorkflowTask {
        WorkflowTask::new("test-task")
            .with_duration(duration_ms)
            .with_cost(cost)
            .with_cpu(cpu)
    }

    fn create_test_state(tasks: &[WorkflowTask]) -> State {
        let mut state = State::new();
        state.execution_order = tasks.iter().map(|t| t.id).collect();
        state.energy = 100.0;
        state.constraint_satisfaction = 0.9;
        state
    }

    #[test]
    fn test_latency_constraint_satisfied() {
        let tasks = vec![
            create_test_task(50, 1.0, 50.0),
            create_test_task(30, 1.0, 50.0),
        ];
        let state = create_test_state(&tasks);

        let constraint = LatencyConstraint::new(100);
        assert!(constraint.is_satisfied(&state, &tasks));
        assert_eq!(constraint.penalty(&state, &tasks), 0.0);
    }

    #[test]
    fn test_latency_constraint_violated() {
        let tasks = vec![
            create_test_task(60, 1.0, 50.0),
            create_test_task(50, 1.0, 50.0),
        ];
        let state = create_test_state(&tasks);

        let constraint = LatencyConstraint::new(100);
        assert!(!constraint.is_satisfied(&state, &tasks));
        assert!(constraint.penalty(&state, &tasks) > 0.0);
    }

    #[test]
    fn test_cost_constraint() {
        let tasks = vec![
            create_test_task(50, 40.0, 50.0),
            create_test_task(50, 40.0, 50.0),
        ];
        let state = create_test_state(&tasks);

        let constraint = CostConstraint::new(100.0);
        assert!(!constraint.is_satisfied(&state, &tasks));
        assert!(constraint.penalty(&state, &tasks) > 0.0);
    }

    #[test]
    fn test_resource_constraint() {
        let tasks = vec![
            create_test_task(50, 1.0, 90.0),
            create_test_task(50, 1.0, 90.0),
        ];
        let state = create_test_state(&tasks);

        let constraint = ResourceConstraint::new(80.0);
        assert!(!constraint.is_satisfied(&state, &tasks));
        assert!(constraint.penalty(&state, &tasks) > 0.0);
    }

    #[test]
    fn test_constraint_manager() {
        let mut manager = ConstraintManager::new();

        manager
            .add_constraint(Box::new(LatencyConstraint::new(100)))
            .unwrap();
        manager
            .add_constraint(Box::new(CostConstraint::new(100.0)))
            .unwrap();
        manager
            .add_constraint(Box::new(ResourceConstraint::new(80.0)))
            .unwrap();

        assert_eq!(manager.len(), 3);

        let tasks = vec![
            create_test_task(30, 30.0, 50.0),
            create_test_task(20, 20.0, 40.0),
        ];
        let state = create_test_state(&tasks);

        assert!(manager.all_satisfied(&state, &tasks));
        assert_eq!(manager.total_penalty(&state, &tasks), 0.0);
    }
}
