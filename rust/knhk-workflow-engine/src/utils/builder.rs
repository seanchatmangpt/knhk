//! Builder pattern utilities
//!
//! Provides builder patterns for complex object construction.

/// Builder trait for fluent object construction
pub trait Builder<T> {
    /// Build the final object
    fn build(self) -> T;
}

/// Workflow spec builder
pub struct WorkflowSpecBuilder {
    name: Option<String>,
    tasks: Vec<String>,
    conditions: Vec<String>,
}

impl WorkflowSpecBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            name: None,
            tasks: Vec::new(),
            conditions: Vec::new(),
        }
    }

    /// Set workflow name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Add task
    pub fn add_task(mut self, task: impl Into<String>) -> Self {
        self.tasks.push(task.into());
        self
    }

    /// Add condition
    pub fn add_condition(mut self, condition: impl Into<String>) -> Self {
        self.conditions.push(condition.into());
        self
    }
}

impl Default for WorkflowSpecBuilder {
    fn default() -> Self {
        Self::new()
    }
}
