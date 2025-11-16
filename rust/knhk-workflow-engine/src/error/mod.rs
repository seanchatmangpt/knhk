//! Advanced error handling with thiserror/anyhow patterns
//!
//! This module provides comprehensive error handling for the workflow engine:
//! - **Custom Error Types**: Detailed error variants with context
//! - **Error Recovery**: Automatic recovery strategies for transient errors
//! - **Backtraces**: Full error chain tracking for debugging
//! - **User-Friendly Messages**: Clear error messages for end users
//!
//! # Error Hierarchy
//!
//! ```text
//! WorkflowError (top-level)
//! ├── StateStoreError (storage layer)
//! ├── RdfValidationError (RDF/semantic layer)
//! ├── ConnectorError (external systems)
//! └── PatternError (pattern execution)
//! ```
//!
//! # Examples
//!
//! ```rust,no_run
//! use knhk_workflow_engine::error::{WorkflowError, WorkflowResult};
//!
//! fn execute_workflow(case_id: &str) -> WorkflowResult<()> {
//!     // Errors automatically include context
//!     let case = get_case(case_id)
//!         .map_err(|e| WorkflowError::CaseNotFound {
//!             case_id: case_id.to_string()
//!         })?;
//!
//!     // Recovery strategies automatically applied
//!     if let Err(e) = execute(&case) {
//!         if e.is_recoverable() {
//!             // Automatic retry with exponential backoff
//!             return retry_with_backoff(|| execute(&case));
//!         }
//!         return Err(e);
//!     }
//!
//!     Ok(())
//! }
//! # fn get_case(id: &str) -> Result<(), ()> { Ok(()) }
//! # fn execute(case: &()) -> Result<(), WorkflowError> { Ok(()) }
//! # fn retry_with_backoff<F>(f: F) -> WorkflowResult<()>
//! # where F: Fn() -> Result<(), WorkflowError> { Ok(()) }
//! ```

use thiserror::Error;

pub mod backtrace;
pub mod context;
pub mod recovery;
pub mod sources;
pub mod try_blocks;

pub use backtrace::{capture_error, ErrorChain};
pub use context::{error_with_context, ErrorContext, IntoWorkflowResult};
pub use recovery::{recover_from_error, retry_sync, Recoverable, RecoveryStrategy};
pub use sources::{
    CircuitBreakerError, ConnectorError, PoolError, RdfValidationError, RegistryError,
    RetryError, StateStoreError,
};
pub use try_blocks::{
    try_execute, try_execute_all, try_execute_all_or_fail, try_execute_with_recovery,
};

/// Result type for workflow operations
pub type WorkflowResult<T> = Result<T, WorkflowError>;

/// Comprehensive error types for workflow engine
///
/// This enum covers all possible error conditions in the workflow engine,
/// from parsing to execution to state management.
///
/// # Error Categories
///
/// - **Specification Errors**: Invalid workflow definitions
/// - **Case Errors**: Case lifecycle issues
/// - **Pattern Errors**: Pattern execution failures
/// - **Resource Errors**: Resource allocation/deadlock issues
/// - **State Errors**: State persistence failures
/// - **Connector Errors**: External system failures
/// - **Validation Errors**: RDF/semantic validation failures
#[derive(Error, Debug)]
pub enum WorkflowError {
    // Specification Errors
    /// Workflow specification not found
    #[error("Workflow specification not found: {spec_id}")]
    SpecNotFound {
        /// Specification ID that was not found
        spec_id: String,
    },

    /// Invalid workflow specification
    #[error("Invalid workflow specification: {reason}")]
    InvalidSpecification {
        /// Reason for invalidity
        reason: String,
    },

    // Case Errors
    /// Case not found
    #[error("Case not found: {case_id}")]
    CaseNotFound {
        /// Case ID that was not found
        case_id: String,
    },

    /// Case already exists
    #[error("Case already exists: {case_id}")]
    CaseExists {
        /// Case ID that already exists
        case_id: String,
    },

    /// Invalid case state transition
    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition {
        /// Source state
        from: String,
        /// Target state
        to: String,
    },

    // Pattern Errors
    /// Pattern not found
    #[error("Pattern {pattern_id} not found")]
    PatternNotFound {
        /// Pattern ID that was not found
        pattern_id: u32,
    },

    /// Pattern execution failed
    #[error("Pattern execution failed: pattern {pattern_id}")]
    PatternExecutionFailed {
        /// Pattern ID that failed
        pattern_id: u32,
        /// Source error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // Resource Errors
    /// Resource allocation failed
    #[error("Resource allocation failed: {resource_id} - {reason}")]
    ResourceAllocationFailed {
        /// Resource ID
        resource_id: String,
        /// Reason for failure
        reason: String,
    },

    /// Resource not available
    #[error("Resource not available: {resource_id}")]
    ResourceUnavailable {
        /// Resource ID
        resource_id: String,
    },

    /// Deadlock detected
    #[error("Deadlock detected: {cycles_count} cycles found")]
    DeadlockDetected {
        /// Number of deadlock cycles
        cycles_count: usize,
        /// Actual cycles (resource IDs)
        cycles: Vec<Vec<String>>,
    },

    // Timeout Errors
    /// Timeout waiting for resource
    #[error("Timeout waiting for {resource_type} after {duration_ms}ms")]
    Timeout {
        /// Type of resource being waited for
        resource_type: String,
        /// Duration in milliseconds
        duration_ms: u64,
    },

    // State Store Errors
    /// State store error
    #[error("State store error")]
    StateStoreError(#[from] StateStoreError),

    /// State persistence error
    #[error("State persistence error: {message}")]
    StatePersistence {
        /// Error message
        message: String,
    },

    // Connector Errors
    /// Connector error
    #[error("Connector error: {connector_name}")]
    ConnectorError {
        /// Connector name
        connector_name: String,
        /// Source error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// External system error
    #[error("External system error: {system_name} - {message}")]
    ExternalSystem {
        /// External system name
        system_name: String,
        /// Error message
        message: String,
    },

    // Validation Errors
    /// RDF validation error
    #[error("RDF validation error")]
    RdfValidationError(#[from] RdfValidationError),

    /// Validation error
    #[error("Validation error: {message}")]
    Validation {
        /// Validation error message
        message: String,
    },

    // Task Errors
    /// Task execution failed
    #[error("Task execution failed: {task_id} - {reason}")]
    TaskExecutionFailed {
        /// Task ID
        task_id: String,
        /// Reason for failure
        reason: String,
    },

    /// Cancellation failed
    #[error("Cancellation failed: {task_id} - {reason}")]
    CancellationFailed {
        /// Task ID
        task_id: String,
        /// Reason for failure
        reason: String,
    },

    // Parse Errors
    /// Parse error
    #[error("Parse error: {message}")]
    Parse {
        /// Parse error message
        message: String,
    },

    // Recoverable Errors
    /// Recoverable error (can be retried)
    #[error("Recoverable error: {message}")]
    Recoverable {
        /// Error message
        message: String,
    },

    // Internal Errors
    /// Internal error
    #[error("Internal error: {message}")]
    Internal {
        /// Error message
        message: String,
    },
}

impl WorkflowError {
    /// Check if this error is recoverable
    ///
    /// Recoverable errors can be retried with appropriate strategies.
    ///
    /// # Returns
    ///
    /// `true` if the error is recoverable, `false` otherwise.
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Recoverable { .. }
                | Self::Timeout { .. }
                | Self::ResourceUnavailable { .. }
                | Self::ExternalSystem { .. }
        )
    }

    /// Get a user-friendly error message
    ///
    /// Converts technical errors into messages suitable for end users.
    ///
    /// # Returns
    ///
    /// A user-friendly error message.
    pub fn user_message(&self) -> String {
        match self {
            Self::SpecNotFound { spec_id } => {
                format!(
                    "Workflow specification '{}' not found. Please check the specification ID.",
                    spec_id
                )
            }
            Self::CaseNotFound { case_id } => {
                format!(
                    "Case '{}' not found. The case may have expired or been deleted.",
                    case_id
                )
            }
            Self::PatternExecutionFailed { pattern_id, .. } => {
                format!(
                    "Pattern {} execution failed. Please check the workflow logs for details.",
                    pattern_id
                )
            }
            Self::DeadlockDetected { cycles_count, .. } => {
                format!(
                    "Deadlock detected in workflow: {} resource cycles found. Please review resource allocation.",
                    cycles_count
                )
            }
            Self::Timeout {
                resource_type,
                duration_ms,
            } => {
                format!(
                    "Timeout waiting for {} after {}ms. Please try again later.",
                    resource_type, duration_ms
                )
            }
            Self::ResourceUnavailable { resource_id } => {
                format!(
                    "Resource '{}' is currently unavailable. Please try again later.",
                    resource_id
                )
            }
            Self::InvalidStateTransition { from, to } => {
                format!(
                    "Cannot transition from state '{}' to '{}'. Please check workflow state.",
                    from, to
                )
            }
            _ => self.to_string(),
        }
    }

    /// Get error severity level
    ///
    /// # Returns
    ///
    /// Severity level: "critical", "error", "warning", or "info"
    pub fn severity(&self) -> &str {
        match self {
            Self::DeadlockDetected { .. } | Self::Internal { .. } => "critical",
            Self::PatternExecutionFailed { .. }
            | Self::TaskExecutionFailed { .. }
            | Self::StateStoreError(_)
            | Self::StatePersistence { .. } => "error",
            Self::Timeout { .. }
            | Self::ResourceUnavailable { .. }
            | Self::Recoverable { .. } => "warning",
            _ => "info",
        }
    }
}

// Standard library conversions
impl From<std::io::Error> for WorkflowError {
    fn from(err: std::io::Error) -> Self {
        WorkflowError::StatePersistence {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for WorkflowError {
    fn from(err: serde_json::Error) -> Self {
        WorkflowError::Parse {
            message: format!("JSON parse error: {}", err),
        }
    }
}

#[cfg(feature = "rdf")]
impl From<oxigraph::sparql::QueryEvaluationError> for WorkflowError {
    fn from(err: oxigraph::sparql::QueryEvaluationError) -> Self {
        WorkflowError::RdfValidationError(RdfValidationError::QueryError(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_is_recoverable() {
        let timeout = WorkflowError::Timeout {
            resource_type: "mutex".to_string(),
            duration_ms: 5000,
        };
        assert!(timeout.is_recoverable());

        let parse_error = WorkflowError::Parse {
            message: "invalid JSON".to_string(),
        };
        assert!(!parse_error.is_recoverable());
    }

    #[test]
    fn test_user_message() {
        let error = WorkflowError::CaseNotFound {
            case_id: "case-123".to_string(),
        };
        let msg = error.user_message();
        assert!(msg.contains("case-123"));
        assert!(msg.contains("not found"));
    }

    #[test]
    fn test_severity() {
        let deadlock = WorkflowError::DeadlockDetected {
            cycles_count: 2,
            cycles: vec![],
        };
        assert_eq!(deadlock.severity(), "critical");

        let timeout = WorkflowError::Timeout {
            resource_type: "resource".to_string(),
            duration_ms: 1000,
        };
        assert_eq!(timeout.severity(), "warning");
    }
}
