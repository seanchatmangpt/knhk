//! Error recovery strategies
//!
//! Provides automatic recovery mechanisms for transient errors.
//!
//! # Examples
//!
//! ```rust,no_run
//! use knhk_workflow_engine::error::{WorkflowError, WorkflowResult, recover_from_error};
//! use std::future::Future;
//! use std::pin::Pin;
//!
//! async fn risky_operation() -> WorkflowResult<String> {
//!     // Might fail with timeout
//!     Err(WorkflowError::Timeout {
//!         resource_type: "database".to_string(),
//!         duration_ms: 5000,
//!     })
//! }
//!
//! async fn execute_with_recovery() -> WorkflowResult<String> {
//!     let error = WorkflowError::Timeout {
//!         resource_type: "database".to_string(),
//!         duration_ms: 5000,
//!     };
//!
//!     recover_from_error(&error, || {
//!         Box::pin(risky_operation())
//!     }).await
//! }
//! ```

use crate::error::{WorkflowError, WorkflowResult};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tracing::{debug, warn};

/// Type alias for boxed future
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Trait for recoverable errors
pub trait Recoverable {
    /// Check if the error is recoverable
    fn is_recoverable(&self) -> bool;

    /// Get the recovery strategy for this error
    fn recovery_strategy(&self) -> RecoveryStrategy;
}

/// Recovery strategy for errors
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry with exponential backoff
    Retry {
        /// Maximum number of retry attempts
        max_attempts: u32,
        /// Initial backoff in milliseconds
        backoff_ms: u64,
        /// Maximum backoff in milliseconds
        max_backoff_ms: u64,
    },
    /// Fallback to alternative resource/path
    Fallback {
        /// Alternative resource identifier
        alternative: String,
    },
    /// Wait and retry once
    Wait {
        /// Duration to wait in milliseconds
        duration_ms: u64,
    },
    /// Graceful degradation
    Degrade {
        /// Description of reduced capability
        reduced_capability: String,
    },
    /// Fail fast (no recovery)
    FailFast,
}

impl Recoverable for WorkflowError {
    fn is_recoverable(&self) -> bool {
        self.is_recoverable()
    }

    fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            Self::Timeout { .. } => RecoveryStrategy::Retry {
                max_attempts: 3,
                backoff_ms: 1000,
                max_backoff_ms: 10000,
            },
            Self::ResourceUnavailable { .. } => RecoveryStrategy::Wait { duration_ms: 2000 },
            Self::ExternalSystem { .. } => RecoveryStrategy::Retry {
                max_attempts: 5,
                backoff_ms: 500,
                max_backoff_ms: 5000,
            },
            Self::Recoverable { .. } => RecoveryStrategy::Retry {
                max_attempts: 3,
                backoff_ms: 100,
                max_backoff_ms: 1000,
            },
            Self::ConnectorError { .. } => RecoveryStrategy::Retry {
                max_attempts: 3,
                backoff_ms: 1000,
                max_backoff_ms: 5000,
            },
            _ => RecoveryStrategy::FailFast,
        }
    }
}

/// Recover from an error using appropriate strategy
///
/// # Arguments
///
/// * `error` - The error to recover from
/// * `operation` - The operation to retry
///
/// # Returns
///
/// Result of the recovery attempt
pub async fn recover_from_error<F, T>(error: &WorkflowError, operation: F) -> WorkflowResult<T>
where
    F: Fn() -> BoxFuture<'static, WorkflowResult<T>>,
{
    let strategy = error.recovery_strategy();

    match strategy {
        RecoveryStrategy::Retry {
            max_attempts,
            backoff_ms,
            max_backoff_ms,
        } => {
            retry_with_backoff(operation, max_attempts, backoff_ms, max_backoff_ms).await
        }
        RecoveryStrategy::Fallback { alternative } => {
            warn!("Using fallback: {}", alternative);
            operation().await
        }
        RecoveryStrategy::Wait { duration_ms } => {
            debug!("Waiting {}ms before retry", duration_ms);
            tokio::time::sleep(Duration::from_millis(duration_ms)).await;
            operation().await
        }
        RecoveryStrategy::Degrade { reduced_capability } => {
            warn!("Degrading to reduced capability: {}", reduced_capability);
            operation().await
        }
        RecoveryStrategy::FailFast => operation().await,
    }
}

/// Retry operation with exponential backoff
async fn retry_with_backoff<F, T>(
    operation: F,
    max_attempts: u32,
    initial_backoff_ms: u64,
    max_backoff_ms: u64,
) -> WorkflowResult<T>
where
    F: Fn() -> BoxFuture<'static, WorkflowResult<T>>,
{
    let mut attempt = 0;
    let mut current_backoff = initial_backoff_ms;

    loop {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    debug!("Operation succeeded after {} retries", attempt);
                }
                return Ok(result);
            }
            Err(e) if attempt < max_attempts => {
                attempt += 1;
                warn!(
                    "Operation failed (attempt {}/{}): {}. Retrying after {}ms",
                    attempt, max_attempts, e, current_backoff
                );
                tokio::time::sleep(Duration::from_millis(current_backoff)).await;
                current_backoff = (current_backoff * 2).min(max_backoff_ms);
            }
            Err(e) => {
                warn!("Operation failed after {} attempts", max_attempts);
                return Err(e);
            }
        }
    }
}

/// Retry a synchronous operation
///
/// # Arguments
///
/// * `operation` - The operation to retry
/// * `max_attempts` - Maximum number of attempts
///
/// # Returns
///
/// Result of the retry attempt
pub fn retry_sync<F, T>(operation: F, max_attempts: u32) -> WorkflowResult<T>
where
    F: Fn() -> WorkflowResult<T>,
{
    let mut attempt = 0;

    loop {
        match operation() {
            Ok(result) => {
                if attempt > 0 {
                    debug!("Operation succeeded after {} retries", attempt);
                }
                return Ok(result);
            }
            Err(e) if attempt < max_attempts => {
                attempt += 1;
                warn!("Operation failed (attempt {}/{}): {}", attempt, max_attempts, e);
            }
            Err(e) => {
                warn!("Operation failed after {} attempts", max_attempts);
                return Err(e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_eventually_succeeds() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let operation = move || {
            let counter = counter_clone.clone();
            Box::pin(async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(WorkflowError::Timeout {
                        resource_type: "test".to_string(),
                        duration_ms: 100,
                    })
                } else {
                    Ok(42)
                }
            }) as BoxFuture<'static, WorkflowResult<i32>>
        };

        let error = WorkflowError::Timeout {
            resource_type: "test".to_string(),
            duration_ms: 100,
        };

        let result = recover_from_error(&error, operation).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_max_attempts() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let operation = move || {
            let counter = counter_clone.clone();
            Box::pin(async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err(WorkflowError::Timeout {
                    resource_type: "test".to_string(),
                    duration_ms: 100,
                })
            }) as BoxFuture<'static, WorkflowResult<i32>>
        };

        let error = WorkflowError::Timeout {
            resource_type: "test".to_string(),
            duration_ms: 100,
        };

        let result = recover_from_error(&error, operation).await;
        assert!(result.is_err());
        // Initial attempt + 3 retries = 4 total
        assert_eq!(counter.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn test_retry_sync() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let operation = move || {
            let count = counter_clone.fetch_add(1, Ordering::SeqCst);
            if count < 1 {
                Err(WorkflowError::Recoverable {
                    message: "temporary failure".to_string(),
                })
            } else {
                Ok(100)
            }
        };

        let result = retry_sync(operation, 3);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_recovery_strategy() {
        let timeout = WorkflowError::Timeout {
            resource_type: "lock".to_string(),
            duration_ms: 5000,
        };
        match timeout.recovery_strategy() {
            RecoveryStrategy::Retry { max_attempts, .. } => {
                assert_eq!(max_attempts, 3);
            }
            _ => panic!("Expected Retry strategy"),
        }

        let parse_error = WorkflowError::Parse {
            message: "invalid".to_string(),
        };
        match parse_error.recovery_strategy() {
            RecoveryStrategy::FailFast => {}
            _ => panic!("Expected FailFast strategy"),
        }
    }
}
