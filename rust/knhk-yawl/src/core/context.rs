//! Execution context structures
//!
//! Maintains runtime context for workflow execution.

use crate::core::NetState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Execution context for a workflow instance
///
/// Contains all runtime state and metadata for a specific workflow execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionContext {
    /// Workflow ID being executed
    pub workflow_id: String,

    /// Unique instance ID for this execution
    pub instance_id: String,

    /// Current net state (tokens, active tasks, history)
    pub state: NetState,

    /// Workflow variables (global scope)
    pub variables: HashMap<String, String>,

    /// Execution metadata
    pub metadata: HashMap<String, String>,

    /// Start timestamp (ISO 8601)
    pub start_time: Option<String>,

    /// End timestamp (ISO 8601)
    pub end_time: Option<String>,

    /// Execution status
    pub status: ExecutionStatus,
}

/// Status of workflow execution
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// Execution not yet started
    Pending,
    /// Currently executing
    Running,
    /// Execution completed successfully
    Completed,
    /// Execution failed
    Failed,
    /// Execution was cancelled
    Cancelled,
}

impl fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Running => write!(f, "Running"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl ExecutionContext {
    /// Create a new execution context builder
    #[must_use]
    pub fn builder() -> ContextBuilder {
        ContextBuilder::default()
    }

    /// Get a variable value
    #[must_use]
    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }

    /// Set a variable value
    pub fn set_variable(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(name.into(), value.into());
    }

    /// Mark execution as started
    pub fn mark_started(&mut self) {
        self.status = ExecutionStatus::Running;
        self.start_time = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Mark execution as completed
    pub fn mark_completed(&mut self) {
        self.status = ExecutionStatus::Completed;
        self.end_time = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Mark execution as failed
    pub fn mark_failed(&mut self) {
        self.status = ExecutionStatus::Failed;
        self.end_time = Some(chrono::Utc::now().to_rfc3339());
    }
}

impl fmt::Display for ExecutionContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ExecutionContext(workflow={}, instance={}, status={})",
            self.workflow_id, self.instance_id, self.status
        )
    }
}

/// Builder for constructing ExecutionContext
#[derive(Default)]
pub struct ContextBuilder {
    workflow_id: Option<String>,
    instance_id: Option<String>,
    state: Option<NetState>,
    variables: HashMap<String, String>,
    metadata: HashMap<String, String>,
}

impl ContextBuilder {
    /// Set workflow ID
    #[must_use]
    pub fn workflow_id(mut self, workflow_id: impl Into<String>) -> Self {
        self.workflow_id = Some(workflow_id.into());
        self
    }

    /// Set instance ID
    #[must_use]
    pub fn instance_id(mut self, instance_id: impl Into<String>) -> Self {
        self.instance_id = Some(instance_id.into());
        self
    }

    /// Set net state
    #[must_use]
    pub fn state(mut self, state: NetState) -> Self {
        self.state = Some(state);
        self
    }

    /// Add a variable
    #[must_use]
    pub fn variable(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(name.into(), value.into());
        self
    }

    /// Add metadata
    #[must_use]
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the execution context
    ///
    /// # Panics
    /// Panics if required fields are not set
    #[must_use]
    pub fn build(self) -> ExecutionContext {
        ExecutionContext {
            workflow_id: self.workflow_id.expect("Workflow ID is required"),
            instance_id: self.instance_id.expect("Instance ID is required"),
            state: self.state.unwrap_or_default(),
            variables: self.variables,
            metadata: self.metadata,
            start_time: None,
            end_time: None,
            status: ExecutionStatus::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder() {
        let context = ExecutionContext::builder()
            .workflow_id("wf1")
            .instance_id("inst1")
            .variable("var1", "value1")
            .metadata("key1", "meta1")
            .build();

        assert_eq!(context.workflow_id, "wf1");
        assert_eq!(context.instance_id, "inst1");
        assert_eq!(context.status, ExecutionStatus::Pending);
        assert_eq!(context.get_variable("var1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_execution_lifecycle() {
        let mut context = ExecutionContext::builder()
            .workflow_id("wf1")
            .instance_id("inst1")
            .build();

        assert_eq!(context.status, ExecutionStatus::Pending);

        context.mark_started();
        assert_eq!(context.status, ExecutionStatus::Running);
        assert!(context.start_time.is_some());

        context.mark_completed();
        assert_eq!(context.status, ExecutionStatus::Completed);
        assert!(context.end_time.is_some());
    }

    #[test]
    fn test_variables() {
        let mut context = ExecutionContext::builder()
            .workflow_id("wf1")
            .instance_id("inst1")
            .build();

        context.set_variable("x", "10");
        context.set_variable("y", "20");

        assert_eq!(context.get_variable("x"), Some(&"10".to_string()));
        assert_eq!(context.get_variable("y"), Some(&"20".to_string()));
        assert_eq!(context.get_variable("z"), None);
    }
}
