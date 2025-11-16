//! Error backtrace support
//!
//! Provides detailed error chain tracking for debugging.
//!
//! # Examples
//!
//! ```rust
//! use knhk_workflow_engine::error::{ErrorChain, WorkflowError};
//!
//! fn deep_function() -> Result<(), WorkflowError> {
//!     Err(WorkflowError::Internal {
//!         message: "database connection failed".to_string(),
//!     })
//! }
//!
//! fn middle_function() -> Result<(), WorkflowError> {
//!     deep_function().map_err(|e| {
//!         let chain = ErrorChain::new(e)
//!             .add_context("Failed in middle_function".to_string());
//!         println!("Error chain:\n{}", chain.display_chain());
//!         WorkflowError::Internal {
//!             message: "middleware error".to_string(),
//!         }
//!     })
//! }
//! ```

use crate::error::WorkflowError;
use std::backtrace::Backtrace;

/// Error chain with backtrace support
///
/// Tracks the full error propagation chain for debugging.
#[derive(Debug)]
pub struct ErrorChain {
    /// Chain of error contexts
    errors: Vec<String>,
    /// Backtrace (if available)
    backtrace: Option<Backtrace>,
}

impl ErrorChain {
    /// Create a new error chain from a workflow error
    ///
    /// # Arguments
    ///
    /// * `error` - The workflow error to create a chain from
    ///
    /// # Returns
    ///
    /// New error chain
    pub fn new(error: WorkflowError) -> Self {
        Self {
            errors: vec![error.to_string()],
            backtrace: Some(Backtrace::capture()),
        }
    }

    /// Add context to the error chain
    ///
    /// # Arguments
    ///
    /// * `context` - Context message to add
    ///
    /// # Returns
    ///
    /// Self for chaining
    pub fn add_context(mut self, context: String) -> Self {
        self.errors.push(context);
        self
    }

    /// Display the error chain
    ///
    /// # Returns
    ///
    /// Formatted error chain
    pub fn display_chain(&self) -> String {
        self.errors
            .iter()
            .enumerate()
            .map(|(i, e)| format!("{}: {}", i, e))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Display the error chain with backtrace
    ///
    /// # Returns
    ///
    /// Formatted error chain with backtrace
    pub fn display_with_backtrace(&self) -> String {
        let mut output = self.display_chain();
        if let Some(bt) = &self.backtrace {
            let bt_str = bt.to_string();
            if !bt_str.is_empty() && bt_str != "disabled backtrace" {
                output.push_str(&format!("\n\nBacktrace:\n{}", bt));
            }
        }
        output
    }

    /// Get the root error
    ///
    /// # Returns
    ///
    /// The original error message
    pub fn root_error(&self) -> Option<&String> {
        self.errors.first()
    }

    /// Get all error contexts
    ///
    /// # Returns
    ///
    /// Vector of all error contexts
    pub fn contexts(&self) -> &[String] {
        &self.errors
    }

    /// Get the depth of the error chain
    ///
    /// # Returns
    ///
    /// Number of contexts in the chain
    pub fn depth(&self) -> usize {
        self.errors.len()
    }
}

/// Capture error with backtrace
///
/// # Arguments
///
/// * `error` - The error to capture
///
/// # Returns
///
/// Error chain with backtrace
pub fn capture_error(error: WorkflowError) -> ErrorChain {
    ErrorChain::new(error)
}

/// Macro to add context to an error and capture backtrace
#[macro_export]
macro_rules! error_context {
    ($result:expr, $context:expr) => {
        $result.map_err(|e| {
            let chain = $crate::error::ErrorChain::new(e).add_context($context.to_string());
            tracing::error!("Error: {}", chain.display_chain());
            e
        })
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_chain_creation() {
        let error = WorkflowError::CaseNotFound {
            case_id: "test-123".to_string(),
        };
        let chain = ErrorChain::new(error);

        assert_eq!(chain.depth(), 1);
        assert!(chain.root_error().is_some());
        assert!(chain.root_error().unwrap().contains("test-123"));
    }

    #[test]
    fn test_add_context() {
        let error = WorkflowError::Internal {
            message: "database error".to_string(),
        };
        let chain = ErrorChain::new(error)
            .add_context("Failed to save workflow state".to_string())
            .add_context("Workflow execution failed".to_string());

        assert_eq!(chain.depth(), 3);
        let contexts = chain.contexts();
        assert!(contexts[0].contains("database error"));
        assert!(contexts[1].contains("save workflow state"));
        assert!(contexts[2].contains("Workflow execution failed"));
    }

    #[test]
    fn test_display_chain() {
        let error = WorkflowError::Timeout {
            resource_type: "mutex".to_string(),
            duration_ms: 5000,
        };
        let chain = ErrorChain::new(error)
            .add_context("Resource allocation failed".to_string());

        let display = chain.display_chain();
        assert!(display.contains("0:"));
        assert!(display.contains("1:"));
        assert!(display.contains("Timeout"));
        assert!(display.contains("Resource allocation"));
    }

    #[test]
    fn test_capture_error() {
        let error = WorkflowError::PatternExecutionFailed {
            pattern_id: 42,
            source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "IO error")),
        };
        let chain = capture_error(error);

        assert_eq!(chain.depth(), 1);
        assert!(chain.root_error().is_some());
    }

    #[test]
    fn test_display_with_backtrace() {
        let error = WorkflowError::DeadlockDetected {
            cycles_count: 2,
            cycles: vec![
                vec!["resource1".to_string(), "resource2".to_string()],
                vec!["resource3".to_string(), "resource4".to_string()],
            ],
        };
        let chain = ErrorChain::new(error);

        let display = chain.display_with_backtrace();
        assert!(display.contains("Deadlock detected"));
        // Backtrace may or may not be available depending on environment
    }
}
