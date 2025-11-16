//! Type-State Pattern for Builders
//!
//! Uses type-state pattern to enforce correct builder usage at compile time,
//! preventing invalid object construction and ensuring all required fields are set.

use crate::case::{Case, CaseState};
use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpecId;
use std::marker::PhantomData;

/// Marker trait for builder states
pub trait BuilderState: sealed::Sealed {}

mod sealed {
    pub trait Sealed {}
}

/// Initial state: needs workflow specification ID
#[derive(Debug, Clone, Copy)]
pub struct NeedsSpecId;

/// State: has specification ID, needs data
#[derive(Debug, Clone, Copy)]
pub struct HasSpecId;

/// State: has all required fields, ready to build
#[derive(Debug, Clone, Copy)]
pub struct ReadyToBuild;

impl sealed::Sealed for NeedsSpecId {}
impl sealed::Sealed for HasSpecId {}
impl sealed::Sealed for ReadyToBuild {}

impl BuilderState for NeedsSpecId {}
impl BuilderState for HasSpecId {}
impl BuilderState for ReadyToBuild {}

/// Type-safe builder for workflow cases
///
/// Uses the type-state pattern to enforce that all required fields are set
/// before building. The compiler prevents calling `build()` on an incomplete builder.
///
/// # Example
///
/// ```rust,ignore
/// use knhk_workflow_engine::builders::type_state::CaseBuilder;
/// use knhk_workflow_engine::parser::WorkflowSpecId;
///
/// let case = CaseBuilder::new()
///     .with_spec_id(WorkflowSpecId::new("workflow-123"))
///     .with_data(serde_json::json!({"key": "value"}))
///     .build();
/// ```
#[derive(Debug)]
pub struct CaseBuilder<S: BuilderState> {
    spec_id: Option<WorkflowSpecId>,
    data: Option<serde_json::Value>,
    _state: PhantomData<S>,
}

impl CaseBuilder<NeedsSpecId> {
    /// Create a new case builder in the initial state
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            spec_id: None,
            data: None,
            _state: PhantomData,
        }
    }

    /// Set the workflow specification ID, transitioning to HasSpecId state
    #[inline]
    pub fn with_spec_id(self, spec_id: WorkflowSpecId) -> CaseBuilder<HasSpecId> {
        CaseBuilder {
            spec_id: Some(spec_id),
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl CaseBuilder<HasSpecId> {
    /// Set the case data, transitioning to ReadyToBuild state
    #[inline]
    pub fn with_data(self, data: serde_json::Value) -> CaseBuilder<ReadyToBuild> {
        CaseBuilder {
            spec_id: self.spec_id,
            data: Some(data),
            _state: PhantomData,
        }
    }

    /// Build with empty data
    #[inline]
    pub fn with_empty_data(self) -> CaseBuilder<ReadyToBuild> {
        self.with_data(serde_json::json!({}))
    }
}

impl CaseBuilder<ReadyToBuild> {
    /// Build the case (only available when all required fields are set)
    ///
    /// The type system ensures this method can only be called after setting
    /// both spec_id and data.
    #[inline]
    pub fn build(self) -> Case {
        Case::new(
            self.spec_id.expect("spec_id must be set in ReadyToBuild state"),
            self.data.expect("data must be set in ReadyToBuild state"),
        )
    }
}

impl Default for CaseBuilder<NeedsSpecId> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Marker for workflow states
pub trait WorkflowState: sealed::Sealed {}

/// Initial workflow state
#[derive(Debug, Clone, Copy)]
pub struct WorkflowCreated;

/// Workflow is configured and ready
#[derive(Debug, Clone, Copy)]
pub struct WorkflowConfigured;

/// Workflow is validated and ready to execute
#[derive(Debug, Clone, Copy)]
pub struct WorkflowValidated;

impl sealed::Sealed for WorkflowCreated {}
impl sealed::Sealed for WorkflowConfigured {}
impl sealed::Sealed for WorkflowValidated {}

impl WorkflowState for WorkflowCreated {}
impl WorkflowState for WorkflowConfigured {}
impl WorkflowState for WorkflowValidated {}

/// Type-state builder for workflow specifications
///
/// Ensures workflows are properly configured and validated before execution.
#[derive(Debug)]
pub struct WorkflowBuilder<S: WorkflowState> {
    spec_id: Option<WorkflowSpecId>,
    definition: Option<serde_json::Value>,
    patterns: Vec<u32>,
    _state: PhantomData<S>,
}

impl WorkflowBuilder<WorkflowCreated> {
    /// Create a new workflow builder
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            spec_id: None,
            definition: None,
            patterns: Vec::new(),
            _state: PhantomData,
        }
    }

    /// Set the workflow spec ID
    #[inline]
    pub fn with_spec_id(mut self, spec_id: WorkflowSpecId) -> Self {
        self.spec_id = Some(spec_id);
        self
    }

    /// Set the workflow definition
    #[inline]
    pub fn with_definition(mut self, definition: serde_json::Value) -> Self {
        self.definition = Some(definition);
        self
    }

    /// Add a pattern to the workflow
    #[inline]
    pub fn add_pattern(mut self, pattern_id: u32) -> Self {
        self.patterns.push(pattern_id);
        self
    }

    /// Finalize configuration, transitioning to WorkflowConfigured
    #[inline]
    pub fn configure(self) -> WorkflowResult<WorkflowBuilder<WorkflowConfigured>> {
        if self.spec_id.is_none() {
            return Err(WorkflowError::InvalidSpecification(
                "spec_id is required".to_string(),
            ));
        }

        Ok(WorkflowBuilder {
            spec_id: self.spec_id,
            definition: self.definition,
            patterns: self.patterns,
            _state: PhantomData,
        })
    }
}

impl WorkflowBuilder<WorkflowConfigured> {
    /// Validate the workflow, transitioning to WorkflowValidated
    #[inline]
    pub fn validate(self) -> WorkflowResult<WorkflowBuilder<WorkflowValidated>> {
        // Validation logic would go here
        // For now, we just transition the state

        if self.patterns.is_empty() {
            return Err(WorkflowError::InvalidSpecification(
                "workflow must have at least one pattern".to_string(),
            ));
        }

        Ok(WorkflowBuilder {
            spec_id: self.spec_id,
            definition: self.definition,
            patterns: self.patterns,
            _state: PhantomData,
        })
    }
}

impl WorkflowBuilder<WorkflowValidated> {
    /// Build the final workflow specification
    ///
    /// Only available after configuration and validation.
    #[inline]
    pub fn build(self) -> (WorkflowSpecId, serde_json::Value, Vec<u32>) {
        (
            self.spec_id.expect("spec_id must be set"),
            self.definition.unwrap_or(serde_json::json!({})),
            self.patterns,
        )
    }
}

impl Default for WorkflowBuilder<WorkflowCreated> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Type-state pattern for task execution
pub trait TaskExecutionState: sealed::Sealed {}

#[derive(Debug, Clone, Copy)]
pub struct TaskPending;

#[derive(Debug, Clone, Copy)]
pub struct TaskExecuting;

#[derive(Debug, Clone, Copy)]
pub struct TaskCompleted;

impl sealed::Sealed for TaskPending {}
impl sealed::Sealed for TaskExecuting {}
impl sealed::Sealed for TaskCompleted {}

impl TaskExecutionState for TaskPending {}
impl TaskExecutionState for TaskExecuting {}
impl TaskExecutionState for TaskCompleted {}

/// Type-safe task execution tracker
///
/// Prevents invalid state transitions at compile time.
#[derive(Debug)]
pub struct TaskExecution<S: TaskExecutionState> {
    task_id: String,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
    result: Option<serde_json::Value>,
    _state: PhantomData<S>,
}

impl TaskExecution<TaskPending> {
    /// Create a new pending task
    #[inline(always)]
    pub fn new(task_id: String) -> Self {
        Self {
            task_id,
            started_at: None,
            completed_at: None,
            result: None,
            _state: PhantomData,
        }
    }

    /// Start task execution
    #[inline]
    pub fn start(self) -> TaskExecution<TaskExecuting> {
        TaskExecution {
            task_id: self.task_id,
            started_at: Some(chrono::Utc::now()),
            completed_at: None,
            result: None,
            _state: PhantomData,
        }
    }
}

impl TaskExecution<TaskExecuting> {
    /// Complete the task with a result
    #[inline]
    pub fn complete(self, result: serde_json::Value) -> TaskExecution<TaskCompleted> {
        TaskExecution {
            task_id: self.task_id,
            started_at: self.started_at,
            completed_at: Some(chrono::Utc::now()),
            result: Some(result),
            _state: PhantomData,
        }
    }

    /// Get task ID
    #[inline(always)]
    pub fn task_id(&self) -> &str {
        &self.task_id
    }
}

impl TaskExecution<TaskCompleted> {
    /// Get the task result
    #[inline(always)]
    pub fn result(&self) -> &serde_json::Value {
        self.result.as_ref().expect("result must be set")
    }

    /// Get execution duration
    #[inline]
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_builder_type_state() {
        let spec_id = WorkflowSpecId::new("test-spec".to_string());
        let case = CaseBuilder::new()
            .with_spec_id(spec_id)
            .with_data(serde_json::json!({"test": "data"}))
            .build();

        assert_eq!(case.state, CaseState::Created);
    }

    #[test]
    fn test_workflow_builder_validation() {
        let spec_id = WorkflowSpecId::new("test-workflow".to_string());

        let result = WorkflowBuilder::new()
            .with_spec_id(spec_id)
            .add_pattern(1)
            .configure()
            .and_then(|b| b.validate())
            .map(|b| b.build());

        assert!(result.is_ok());
    }

    #[test]
    fn test_task_execution_flow() {
        let task = TaskExecution::new("task-1".to_string());
        let executing = task.start();
        let completed = executing.complete(serde_json::json!({"status": "success"}));

        assert!(completed.duration().is_some());
    }
}
