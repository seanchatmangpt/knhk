//! Error types for workflow engine
//!
//! Comprehensive error handling with proper error types and context.
//!
//! # Error Categories
//!
//! - **Parse Errors**: Workflow definition parsing failures
//! - **Validation Errors**: Input validation failures
//! - **State Errors**: Invalid state transitions
//! - **Execution Errors**: Task execution failures
//! - **Configuration Errors**: Invalid configuration
//! - **External Errors**: External system failures

use thiserror::Error;

/// Result type for workflow operations
pub type WorkflowResult<T> = Result<T, WorkflowError>;

/// Comprehensive error types for workflow engine
#[derive(Error, Debug, Clone)]
pub enum WorkflowError {
    /// Parsing errors
    #[error("Parse error: {0}")]
    Parse(String),

    /// Pattern not found
    #[error("Pattern {0} not found")]
    PatternNotFound(u32),

    /// Invalid workflow specification
    #[error("Invalid workflow specification: {0}")]
    InvalidSpecification(String),

    /// Case not found
    #[error("Case {0} not found")]
    CaseNotFound(String),

    /// Case already exists
    #[error("Case {0} already exists")]
    CaseExists(String),

    /// Invalid case state transition
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        /// Source state
        from: String,
        /// Target state
        to: String,
    },

    /// Task execution failed
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),

    /// Cancellation failed
    #[error("Cancellation failed: {0}")]
    CancellationFailed(String),

    /// State persistence error
    #[error("State persistence error: {0}")]
    StatePersistence(String),

    /// External system error
    #[error("External system error: {0}")]
    ExternalSystem(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Resource not available
    #[error("Resource not available: {0}")]
    ResourceUnavailable(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<std::io::Error> for WorkflowError {
    fn from(err: std::io::Error) -> Self {
        WorkflowError::StatePersistence(err.to_string())
    }
}

impl From<serde_json::Error> for WorkflowError {
    fn from(err: serde_json::Error) -> Self {
        WorkflowError::Parse(format!("JSON parse error: {}", err))
    }
}

impl From<std::fmt::Error> for WorkflowError {
    fn from(err: std::fmt::Error) -> Self {
        WorkflowError::Internal(format!("Formatting error: {}", err))
    }
}
