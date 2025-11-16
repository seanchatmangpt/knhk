//! Builder patterns with compile-time type safety
//!
//! Provides type-state builders that enforce correct construction sequences
//! at compile time, preventing runtime errors from incomplete or invalid objects.

pub mod type_state;

pub use type_state::{
    BuilderState, CaseBuilder, HasSpecId, NeedsSpecId, ReadyToBuild, TaskCompleted,
    TaskExecution, TaskExecuting, TaskExecutionState, TaskPending, WorkflowBuilder,
    WorkflowConfigured, WorkflowCreated, WorkflowState, WorkflowValidated,
};
