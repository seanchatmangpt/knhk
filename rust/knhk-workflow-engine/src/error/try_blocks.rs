//! Try block patterns for cleaner error handling
//!
//! Provides helper functions and patterns for cleaner error handling.
//!
//! # Examples
//!
//! ```rust,no_run
//! use knhk_workflow_engine::error::{WorkflowError, WorkflowResult, try_execute};
//!
//! async fn complex_operation(case_id: &str) -> WorkflowResult<String> {
//!     try_execute(async {
//!         // Multiple operations that can fail
//!         let case = get_case(case_id).await?;
//!         validate_case(&case)?;
//!         let result = execute_case(&case).await?;
//!         save_result(result.clone()).await?;
//!         Ok(result)
//!     }).await
//! }
//! # async fn get_case(id: &str) -> WorkflowResult<String> { Ok(String::new()) }
//! # fn validate_case(case: &str) -> WorkflowResult<()> { Ok(()) }
//! # async fn execute_case(case: &str) -> WorkflowResult<String> { Ok(String::new()) }
//! # async fn save_result(result: String) -> WorkflowResult<()> { Ok(()) }
//! ```

use crate::error::{ErrorChain, WorkflowError, WorkflowResult};
use std::future::Future;
use tracing::{error, warn};

/// Execute an async operation with automatic error handling
///
/// # Arguments
///
/// * `operation` - The async operation to execute
///
/// # Returns
///
/// Result of the operation with error context
pub async fn try_execute<F, T>(operation: F) -> WorkflowResult<T>
where
    F: Future<Output = WorkflowResult<T>>,
{
    match operation.await {
        Ok(result) => Ok(result),
        Err(e) => {
            let chain = ErrorChain::new(e.clone());
            error!("Operation failed: {}", chain.display_chain());
            Err(e)
        }
    }
}

/// Execute with automatic recovery
///
/// # Arguments
///
/// * `operation` - The operation to execute
///
/// # Returns
///
/// Result with automatic recovery on transient errors
pub async fn try_execute_with_recovery<F, T>(operation: F) -> WorkflowResult<T>
where
    F: Future<Output = WorkflowResult<T>> + Clone,
{
    match operation.clone().await {
        Ok(result) => Ok(result),
        Err(e) if e.is_recoverable() => {
            warn!("Recoverable error, attempting recovery: {}", e);
            crate::error::recovery::recover_from_error(&e, || Box::pin(operation.clone())).await
        }
        Err(e) => {
            error!("Non-recoverable error: {}", e);
            Err(e)
        }
    }
}

/// Execute multiple operations in sequence with error aggregation
///
/// # Arguments
///
/// * `operations` - Vector of operations to execute
///
/// # Returns
///
/// Vector of results
pub async fn try_execute_all<F, T>(operations: Vec<F>) -> Vec<WorkflowResult<T>>
where
    F: Future<Output = WorkflowResult<T>>,
{
    let mut results = Vec::new();
    for operation in operations {
        results.push(operation.await);
    }
    results
}

/// Execute and collect first error
///
/// # Arguments
///
/// * `operations` - Vector of operations to execute
///
/// # Returns
///
/// Ok if all succeed, Err with first error otherwise
pub async fn try_execute_all_or_fail<F, T>(operations: Vec<F>) -> WorkflowResult<Vec<T>>
where
    F: Future<Output = WorkflowResult<T>>,
{
    let mut results = Vec::new();
    for operation in operations {
        results.push(operation.await?);
    }
    Ok(results)
}

/// Macro for try blocks with context
#[macro_export]
macro_rules! try_block {
    ($($body:tt)*) => {{
        (|| -> $crate::error::WorkflowResult<_> {
            $($body)*
        })()
    }};
}

/// Macro for async try blocks with context
#[macro_export]
macro_rules! async_try_block {
    ($($body:tt)*) => {{
        (async move || -> $crate::error::WorkflowResult<_> {
            $($body)*
        })()
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_try_execute_success() {
        let result = try_execute(async { Ok::<i32, WorkflowError>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_try_execute_failure() {
        let result = try_execute(async {
            Err::<i32, WorkflowError>(WorkflowError::Internal {
                message: "test error".to_string(),
            })
        })
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_try_execute_with_recovery() {
        let result = try_execute_with_recovery(async {
            Err::<i32, WorkflowError>(WorkflowError::Recoverable {
                message: "temporary error".to_string(),
            })
        })
        .await;
        // Will fail after retries, but recovery was attempted
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_try_execute_all() {
        let ops = vec![
            async { Ok::<i32, WorkflowError>(1) },
            async { Ok::<i32, WorkflowError>(2) },
            async { Ok::<i32, WorkflowError>(3) },
        ];

        let results = try_execute_all(ops).await;
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[tokio::test]
    async fn test_try_execute_all_or_fail_success() {
        let ops = vec![
            async { Ok::<i32, WorkflowError>(1) },
            async { Ok::<i32, WorkflowError>(2) },
            async { Ok::<i32, WorkflowError>(3) },
        ];

        let result = try_execute_all_or_fail(ops).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_try_execute_all_or_fail_failure() {
        let ops = vec![
            async { Ok::<i32, WorkflowError>(1) },
            async {
                Err::<i32, WorkflowError>(WorkflowError::Internal {
                    message: "error".to_string(),
                })
            },
            async { Ok::<i32, WorkflowError>(3) },
        ];

        let result = try_execute_all_or_fail(ops).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_try_block_macro() {
        let result: WorkflowResult<i32> = try_block! {
            let x = 10;
            let y = 20;
            Ok(x + y)
        };
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 30);
    }

    #[test]
    fn test_try_block_macro_with_error() {
        let result: WorkflowResult<i32> = try_block! {
            Err(WorkflowError::Internal {
                message: "test".to_string(),
            })?;
            Ok(42)
        };
        assert!(result.is_err());
    }
}
