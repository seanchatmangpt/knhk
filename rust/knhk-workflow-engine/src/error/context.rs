//! Error context helpers using anyhow patterns
//!
//! Provides contextual information for errors to aid in debugging.
//!
//! # Examples
//!
//! ```rust,no_run
//! use knhk_workflow_engine::error::{ErrorContext, WorkflowError, WorkflowResult};
//!
//! fn load_case(case_id: &str) -> WorkflowResult<()> {
//!     get_case(case_id)
//!         .context("Failed to retrieve case for execution")?;
//!     Ok(())
//! }
//!
//! fn execute_workflow(case_id: &str) -> WorkflowResult<()> {
//!     load_case(case_id)
//!         .with_context(|| format!("Workflow execution failed for case {}", case_id))?;
//!     Ok(())
//! }
//! # fn get_case(id: &str) -> WorkflowResult<()> { Ok(()) }
//! ```

use crate::error::{WorkflowError, WorkflowResult};
use anyhow::{anyhow, Context as AnyhowContext, Result};

/// Trait for adding context to errors
///
/// This trait provides methods to attach contextual information to errors,
/// making debugging and error reporting easier.
pub trait ErrorContext<T> {
    /// Add static context to the error
    ///
    /// # Arguments
    ///
    /// * `msg` - Static context message
    ///
    /// # Returns
    ///
    /// `Result` with added context
    fn context(self, msg: &str) -> Result<T>;

    /// Add dynamic context to the error
    ///
    /// The context is only computed if an error occurs.
    ///
    /// # Arguments
    ///
    /// * `f` - Function that produces context message
    ///
    /// # Returns
    ///
    /// `Result` with added context
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T> ErrorContext<T> for WorkflowResult<T> {
    fn context(self, msg: &str) -> Result<T> {
        self.map_err(|e| anyhow!("{}: {}", msg, e))
    }

    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| anyhow!("{}: {}", f(), e))
    }
}

/// Extension trait for anyhow::Result to convert to WorkflowResult
pub trait IntoWorkflowResult<T> {
    /// Convert anyhow::Result to WorkflowResult
    ///
    /// # Returns
    ///
    /// `WorkflowResult` with converted error
    fn into_workflow_result(self) -> WorkflowResult<T>;
}

impl<T> IntoWorkflowResult<T> for Result<T> {
    fn into_workflow_result(self) -> WorkflowResult<T> {
        self.map_err(|e| WorkflowError::Internal {
            message: e.to_string(),
        })
    }
}

/// Create a workflow error with context
///
/// # Arguments
///
/// * `context` - Context message
/// * `error` - Source error
///
/// # Returns
///
/// `WorkflowError` with context
pub fn error_with_context(context: &str, error: WorkflowError) -> WorkflowError {
    match error {
        WorkflowError::Internal { message } => WorkflowError::Internal {
            message: format!("{}: {}", context, message),
        },
        _ => error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context() {
        let result: WorkflowResult<()> = Err(WorkflowError::CaseNotFound {
            case_id: "test-123".to_string(),
        });

        let with_ctx = result.context("Failed to load case");
        assert!(with_ctx.is_err());
        let err_msg = with_ctx.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to load case"));
        assert!(err_msg.contains("test-123"));
    }

    #[test]
    fn test_with_context() {
        let case_id = "case-456";
        let result: WorkflowResult<()> = Err(WorkflowError::Timeout {
            resource_type: "lock".to_string(),
            duration_ms: 5000,
        });

        let with_ctx = result.with_context(|| format!("Execution failed for case {}", case_id));
        assert!(with_ctx.is_err());
        let err_msg = with_ctx.unwrap_err().to_string();
        assert!(err_msg.contains("case-456"));
        assert!(err_msg.contains("Timeout"));
    }

    #[test]
    fn test_into_workflow_result() {
        let anyhow_result: Result<i32> = Err(anyhow!("Something went wrong"));
        let workflow_result = anyhow_result.into_workflow_result();

        assert!(workflow_result.is_err());
        match workflow_result {
            Err(WorkflowError::Internal { message }) => {
                assert!(message.contains("Something went wrong"));
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[test]
    fn test_error_with_context() {
        let error = WorkflowError::Internal {
            message: "database error".to_string(),
        };

        let with_context = error_with_context("Failed to save state", error);

        match with_context {
            WorkflowError::Internal { message } => {
                assert!(message.contains("Failed to save state"));
                assert!(message.contains("database error"));
            }
            _ => panic!("Expected Internal error"),
        }
    }
}
