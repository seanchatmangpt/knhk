//! Workflow data structures
//!
//! Covenant 1: Turtle is definition and cause (O ⊨ Σ)
//! - Workflows are projections from RDF ontology
//! - All properties must be observable via telemetry

use crate::core::{Task, Transition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A YAWL Workflow
///
/// Represents a complete workflow definition with tasks, transitions, and metadata.
/// Aligned with `yawl:Workflow` in the YAWL ontology.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Workflow {
    /// Unique workflow identifier
    pub id: String,

    /// Human-readable workflow name
    pub name: String,

    /// Workflow version (semantic versioning)
    pub version: String,

    /// Tasks in this workflow (indexed by task ID)
    pub tasks: HashMap<String, Task>,

    /// Transitions between tasks
    pub transitions: Vec<Transition>,

    /// Workflow-level metadata
    pub metadata: HashMap<String, String>,

    /// Net structure identifier
    pub net_id: Option<String>,
}

impl Workflow {
    /// Create a new workflow builder
    #[must_use]
    pub fn builder() -> WorkflowBuilder {
        WorkflowBuilder::default()
    }

    /// Get a task by ID
    #[must_use]
    pub fn get_task(&self, task_id: &str) -> Option<&Task> {
        self.tasks.get(task_id)
    }

    /// Get all outgoing transitions from a task
    #[must_use]
    pub fn outgoing_transitions(&self, task_id: &str) -> Vec<&Transition> {
        self.transitions
            .iter()
            .filter(|t| t.source == task_id)
            .collect()
    }

    /// Get all incoming transitions to a task
    #[must_use]
    pub fn incoming_transitions(&self, task_id: &str) -> Vec<&Transition> {
        self.transitions
            .iter()
            .filter(|t| t.target == task_id)
            .collect()
    }

    /// Validate workflow structure
    ///
    /// # Errors
    /// Returns error if workflow structure is invalid
    pub fn validate(&self) -> crate::Result<()> {
        // Check for at least one task
        if self.tasks.is_empty() {
            return Err(crate::Error::InvalidWorkflow(
                "Workflow must have at least one task".to_string(),
            ));
        }

        // Validate all transitions reference existing tasks
        for transition in &self.transitions {
            if !self.tasks.contains_key(&transition.source) {
                return Err(crate::Error::InvalidWorkflow(format!(
                    "Transition references non-existent source task: {}",
                    transition.source
                )));
            }
            if !self.tasks.contains_key(&transition.target) {
                return Err(crate::Error::InvalidWorkflow(format!(
                    "Transition references non-existent target task: {}",
                    transition.target
                )));
            }
        }

        Ok(())
    }
}

impl fmt::Display for Workflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Workflow(id={}, name={}, version={}, tasks={}, transitions={})",
            self.id,
            self.name,
            self.version,
            self.tasks.len(),
            self.transitions.len()
        )
    }
}

/// Builder for constructing Workflows
#[derive(Default)]
pub struct WorkflowBuilder {
    id: Option<String>,
    name: Option<String>,
    version: Option<String>,
    tasks: HashMap<String, Task>,
    transitions: Vec<Transition>,
    metadata: HashMap<String, String>,
    net_id: Option<String>,
}

impl WorkflowBuilder {
    /// Set workflow ID
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set workflow name
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set workflow version
    #[must_use]
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Add a task to the workflow
    #[must_use]
    pub fn add_task(mut self, task: Task) -> Self {
        self.tasks.insert(task.id.clone(), task);
        self
    }

    /// Add a transition to the workflow
    #[must_use]
    pub fn add_transition(mut self, transition: Transition) -> Self {
        self.transitions.push(transition);
        self
    }

    /// Add metadata key-value pair
    #[must_use]
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set net ID
    #[must_use]
    pub fn net_id(mut self, net_id: impl Into<String>) -> Self {
        self.net_id = Some(net_id.into());
        self
    }

    /// Build the workflow
    ///
    /// # Panics
    /// Panics if required fields (id, name, version) are not set
    #[must_use]
    pub fn build(self) -> Workflow {
        Workflow {
            id: self.id.expect("Workflow ID is required"),
            name: self.name.expect("Workflow name is required"),
            version: self.version.unwrap_or_else(|| "1.0.0".to_string()),
            tasks: self.tasks,
            transitions: self.transitions,
            metadata: self.metadata,
            net_id: self.net_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::TaskType;

    #[test]
    fn test_workflow_builder() {
        let task = Task::builder()
            .id("task1")
            .name("Test Task")
            .task_type(TaskType::Atomic)
            .build();

        let workflow = Workflow::builder()
            .id("wf1")
            .name("Test Workflow")
            .version("1.0.0")
            .add_task(task)
            .build();

        assert_eq!(workflow.id, "wf1");
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.version, "1.0.0");
        assert_eq!(workflow.tasks.len(), 1);
    }

    #[test]
    fn test_workflow_validation() {
        let task1 = Task::builder()
            .id("task1")
            .name("Task 1")
            .task_type(TaskType::Atomic)
            .build();

        let task2 = Task::builder()
            .id("task2")
            .name("Task 2")
            .task_type(TaskType::Atomic)
            .build();

        let transition = Transition::builder()
            .source("task1")
            .target("task2")
            .build();

        let workflow = Workflow::builder()
            .id("wf1")
            .name("Test")
            .add_task(task1)
            .add_task(task2)
            .add_transition(transition)
            .build();

        assert!(workflow.validate().is_ok());
    }

    #[test]
    fn test_workflow_invalid_transition() {
        let task1 = Task::builder()
            .id("task1")
            .name("Task 1")
            .task_type(TaskType::Atomic)
            .build();

        let transition = Transition::builder()
            .source("task1")
            .target("nonexistent")
            .build();

        let workflow = Workflow::builder()
            .id("wf1")
            .name("Test")
            .add_task(task1)
            .add_transition(transition)
            .build();

        assert!(workflow.validate().is_err());
    }
}
