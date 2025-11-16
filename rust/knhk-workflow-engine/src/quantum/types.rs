//! Core types for quantum-inspired optimization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a workflow task to be scheduled
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowTask {
    /// Unique task identifier
    pub id: Uuid,

    /// Task name
    pub name: String,

    /// Estimated execution time in milliseconds
    pub estimated_duration_ms: u64,

    /// CPU requirement (0.0-100.0)
    pub cpu_requirement: f64,

    /// Memory requirement in MB
    pub memory_requirement: f64,

    /// Cost per execution
    pub cost: f64,

    /// Task priority (higher = more important)
    pub priority: i32,

    /// Dependencies (task IDs that must complete first)
    pub dependencies: Vec<Uuid>,

    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl WorkflowTask {
    /// Create a new workflow task
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            estimated_duration_ms: 100,
            cpu_requirement: 50.0,
            memory_requirement: 256.0,
            cost: 1.0,
            priority: 0,
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set estimated duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.estimated_duration_ms = duration_ms;
        self
    }

    /// Set CPU requirement
    pub fn with_cpu(mut self, cpu: f64) -> Self {
        self.cpu_requirement = cpu.clamp(0.0, 100.0);
        self
    }

    /// Set memory requirement
    pub fn with_memory(mut self, memory_mb: f64) -> Self {
        self.memory_requirement = memory_mb;
        self
    }

    /// Set cost
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.cost = cost;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Add dependency
    pub fn with_dependency(mut self, task_id: Uuid) -> Self {
        self.dependencies.push(task_id);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Represents a state in the quantum optimization space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// Task assignments: task_id -> resource_id
    pub assignments: HashMap<Uuid, String>,

    /// Execution order
    pub execution_order: Vec<Uuid>,

    /// Total energy (cost) of this state
    pub energy: f64,

    /// Constraint satisfaction score (0.0-1.0)
    pub constraint_satisfaction: f64,
}

impl State {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            assignments: HashMap::new(),
            execution_order: Vec::new(),
            energy: f64::INFINITY,
            constraint_satisfaction: 0.0,
        }
    }

    /// Check if state is valid
    pub fn is_valid(&self) -> bool {
        self.energy.is_finite() && self.constraint_satisfaction >= 0.0
    }

    /// Get quality score (inverse of energy, normalized)
    pub fn quality_score(&self) -> f64 {
        if self.energy.is_infinite() || self.energy <= 0.0 {
            0.0
        } else {
            (1.0 / self.energy).min(1.0)
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

/// Energy function for quantum annealing
pub type EnergyFunction = Box<dyn Fn(&State) -> f64 + Send + Sync>;

/// Resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource identifier
    pub id: String,

    /// Available CPU (0.0-100.0)
    pub available_cpu: f64,

    /// Available memory in MB
    pub available_memory: f64,

    /// Cost per time unit
    pub cost_per_ms: f64,

    /// Resource location/zone
    pub location: String,
}

impl Resource {
    /// Create a new resource
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            available_cpu: 100.0,
            available_memory: 4096.0,
            cost_per_ms: 0.001,
            location: "default".to_string(),
        }
    }

    /// Check if resource can accommodate task
    pub fn can_accommodate(&self, task: &WorkflowTask) -> bool {
        self.available_cpu >= task.cpu_requirement
            && self.available_memory >= task.memory_requirement
    }
}

/// Temperature for simulated annealing
#[derive(Debug, Clone, Copy)]
pub struct Temperature {
    /// Current temperature
    pub value: f64,

    /// Initial temperature
    pub initial: f64,

    /// Final temperature
    pub final_temp: f64,

    /// Cooling rate
    pub cooling_rate: f64,
}

impl Temperature {
    /// Create new temperature schedule
    pub fn new(initial: f64, final_temp: f64, cooling_rate: f64) -> Self {
        Self {
            value: initial,
            initial,
            final_temp,
            cooling_rate,
        }
    }

    /// Cool down the temperature
    pub fn cool(&mut self) {
        self.value *= self.cooling_rate;
        self.value = self.value.max(self.final_temp);
    }

    /// Check if annealing is complete
    pub fn is_frozen(&self) -> bool {
        self.value <= self.final_temp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_task_creation() {
        let task = WorkflowTask::new("test-task")
            .with_duration(200)
            .with_cpu(75.0)
            .with_memory(512.0)
            .with_cost(2.5)
            .with_priority(10);

        assert_eq!(task.name, "test-task");
        assert_eq!(task.estimated_duration_ms, 200);
        assert_eq!(task.cpu_requirement, 75.0);
        assert_eq!(task.memory_requirement, 512.0);
        assert_eq!(task.cost, 2.5);
        assert_eq!(task.priority, 10);
    }

    #[test]
    fn test_state_validity() {
        let mut state = State::new();
        assert!(!state.is_valid()); // Infinite energy

        state.energy = 100.0;
        state.constraint_satisfaction = 0.95;
        assert!(state.is_valid());
    }

    #[test]
    fn test_temperature_cooling() {
        let mut temp = Temperature::new(100.0, 0.1, 0.95);
        assert_eq!(temp.value, 100.0);

        temp.cool();
        assert_eq!(temp.value, 95.0);

        for _ in 0..200 {
            temp.cool();
        }

        assert!(temp.is_frozen());
        assert!(temp.value >= temp.final_temp);
    }

    #[test]
    fn test_resource_accommodation() {
        let resource = Resource::new("resource-1");
        let task = WorkflowTask::new("task-1")
            .with_cpu(50.0)
            .with_memory(2048.0);

        assert!(resource.can_accommodate(&task));

        let heavy_task = WorkflowTask::new("task-2")
            .with_cpu(150.0)
            .with_memory(8192.0);

        assert!(!resource.can_accommodate(&heavy_task));
    }
}
