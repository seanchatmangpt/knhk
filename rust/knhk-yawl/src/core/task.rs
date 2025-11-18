//! Task data structures
//!
//! Tasks represent units of work in a YAWL workflow.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Type of task in a workflow
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskType {
    /// Atomic task - single unit of work
    Atomic,
    /// Composite task - contains sub-workflow
    Composite,
    /// Multiple instance task - creates multiple instances
    MultipleInstance,
}

impl fmt::Display for TaskType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atomic => write!(f, "Atomic"),
            Self::Composite => write!(f, "Composite"),
            Self::MultipleInstance => write!(f, "MultipleInstance"),
        }
    }
}

/// A YAWL Task
///
/// Represents a single task in a workflow.
/// Aligned with `yawl:Task` in the YAWL ontology.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    /// Unique task identifier
    pub id: String,

    /// Human-readable task name
    pub name: String,

    /// Type of task
    pub task_type: TaskType,

    /// Input parameters (name -> type mapping)
    pub input_params: HashMap<String, String>,

    /// Output parameters (name -> type mapping)
    pub output_params: HashMap<String, String>,

    /// Task-level metadata
    pub metadata: HashMap<String, String>,

    /// Join type for this task (if applicable)
    pub join_type: Option<super::JoinType>,

    /// Split type for this task (if applicable)
    pub split_type: Option<super::SplitType>,
}

impl Task {
    /// Create a new task
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, task_type: TaskType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            task_type,
            input_params: HashMap::new(),
            output_params: HashMap::new(),
            metadata: HashMap::new(),
            join_type: None,
            split_type: None,
        }
    }

    /// Create a new task builder
    #[must_use]
    pub fn builder() -> TaskBuilder {
        TaskBuilder::default()
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Task(id={}, name={}, type={})",
            self.id, self.name, self.task_type
        )
    }
}

/// Builder for constructing Tasks
#[derive(Default)]
pub struct TaskBuilder {
    id: Option<String>,
    name: Option<String>,
    task_type: Option<TaskType>,
    input_params: HashMap<String, String>,
    output_params: HashMap<String, String>,
    metadata: HashMap<String, String>,
    join_type: Option<super::JoinType>,
    split_type: Option<super::SplitType>,
}

impl TaskBuilder {
    /// Set task ID
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set task name
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set task type
    #[must_use]
    pub fn task_type(mut self, task_type: TaskType) -> Self {
        self.task_type = Some(task_type);
        self
    }

    /// Add input parameter
    #[must_use]
    pub fn input_param(mut self, name: impl Into<String>, param_type: impl Into<String>) -> Self {
        self.input_params.insert(name.into(), param_type.into());
        self
    }

    /// Add output parameter
    #[must_use]
    pub fn output_param(mut self, name: impl Into<String>, param_type: impl Into<String>) -> Self {
        self.output_params.insert(name.into(), param_type.into());
        self
    }

    /// Add metadata
    #[must_use]
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set join type
    #[must_use]
    pub fn join_type(mut self, join_type: super::JoinType) -> Self {
        self.join_type = Some(join_type);
        self
    }

    /// Set split type
    #[must_use]
    pub fn split_type(mut self, split_type: super::SplitType) -> Self {
        self.split_type = Some(split_type);
        self
    }

    /// Build the task
    ///
    /// # Panics
    /// Panics if required fields are not set
    #[must_use]
    pub fn build(self) -> Task {
        Task {
            id: self.id.expect("Task ID is required"),
            name: self.name.expect("Task name is required"),
            task_type: self.task_type.expect("Task type is required"),
            input_params: self.input_params,
            output_params: self.output_params,
            metadata: self.metadata,
            join_type: self.join_type,
            split_type: self.split_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("task1", "Test Task", TaskType::Atomic);
        assert_eq!(task.id, "task1");
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.task_type, TaskType::Atomic);
    }

    #[test]
    fn test_task_builder() {
        let task = Task::builder()
            .id("task1")
            .name("Test Task")
            .task_type(TaskType::Composite)
            .input_param("input1", "string")
            .output_param("output1", "int")
            .metadata("key", "value")
            .build();

        assert_eq!(task.id, "task1");
        assert_eq!(task.input_params.get("input1"), Some(&"string".to_string()));
        assert_eq!(task.output_params.get("output1"), Some(&"int".to_string()));
        assert_eq!(task.metadata.get("key"), Some(&"value".to_string()));
    }
}
