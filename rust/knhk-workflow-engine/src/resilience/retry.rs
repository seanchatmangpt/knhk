#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Retry logic with exponential backoff

use crate::error::{WorkflowError, WorkflowResult};
use std::time::Duration;
use tokio::time::sleep;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay before first retry (milliseconds)
    pub initial_delay_ms: u64,
    /// Maximum delay between retries (milliseconds)
    pub max_delay_ms: u64,
    /// Exponential backoff multiplier
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            multiplier: 2.0,
        }
    }
}

/// Retry policy trait
pub trait RetryPolicy {
    /// Check if an error should be retried
    fn should_retry(&self, error: &WorkflowError) -> bool;
}

/// Default retry policy
pub struct DefaultRetryPolicy;

impl RetryPolicy for DefaultRetryPolicy {
    fn should_retry(&self, error: &WorkflowError) -> bool {
        // Retry transient errors only
        matches!(
            error,
            WorkflowError::ExternalSystem(_)
                | WorkflowError::Timeout
                | WorkflowError::ResourceUnavailable(_)
        )
    }
}

/// Execute a function with retry logic
pub async fn retry_with_backoff<F, Fut, T>(
    config: &RetryConfig,
    policy: &dyn RetryPolicy,
    mut f: F,
) -> WorkflowResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = WorkflowResult<T>>,
{
    let mut delay = Duration::from_millis(config.initial_delay_ms);
    let mut last_error: Option<WorkflowError> = None;

    for attempt in 0..=config.max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(match &e {
                    WorkflowError::Parse(s) => WorkflowError::Parse(s.clone()),
                    WorkflowError::PatternNotFound(n) => WorkflowError::PatternNotFound(*n),
                    WorkflowError::InvalidSpecification(s) => {
                        WorkflowError::InvalidSpecification(s.clone())
                    }
                    WorkflowError::CaseNotFound(s) => WorkflowError::CaseNotFound(s.clone()),
                    WorkflowError::CaseExists(s) => WorkflowError::CaseExists(s.clone()),
                    WorkflowError::Validation(s) => WorkflowError::Validation(s.clone()),
                    WorkflowError::InvalidStateTransition { from, to } => {
                        WorkflowError::InvalidStateTransition {
                            from: from.clone(),
                            to: to.clone(),
                        }
                    }
                    WorkflowError::TaskExecutionFailed(s) => {
                        WorkflowError::TaskExecutionFailed(s.clone())
                    }
                    WorkflowError::CancellationFailed(s) => {
                        WorkflowError::CancellationFailed(s.clone())
                    }
                    WorkflowError::StatePersistence(s) => {
                        WorkflowError::StatePersistence(s.clone())
                    }
                    WorkflowError::ExternalSystem(s) => WorkflowError::ExternalSystem(s.clone()),
                    WorkflowError::Timeout => WorkflowError::Timeout,
                    WorkflowError::ResourceUnavailable(s) => {
                        WorkflowError::ResourceUnavailable(s.clone())
                    }
                    WorkflowError::Internal(s) => WorkflowError::Internal(s.clone()),
                });

                // Check if we should retry
                if attempt < config.max_retries && policy.should_retry(&e) {
                    // Wait before retrying
                    sleep(delay).await;

                    // Calculate next delay with exponential backoff
                    delay = Duration::from_millis(
                        ((delay.as_millis() as f64 * config.multiplier) as u64)
                            .min(config.max_delay_ms),
                    );
                } else {
                    // Don't retry: either max retries reached or non-retryable error
                    break;
                }
            }
        }
    }

    Err(last_error
        .unwrap_or_else(|| WorkflowError::Internal("Retry exhausted without error".to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_success() {
        let config = RetryConfig::default();
        let policy = DefaultRetryPolicy;
        let attempts = Arc::new(std::sync::Mutex::new(0));

        let attempts_clone = attempts.clone();
        let result = retry_with_backoff(&config, &policy, move || {
            let attempts = attempts_clone.clone();
            async move {
                let mut count = attempts.lock().unwrap();
                *count += 1;
                if *count < 2 {
                    Err(WorkflowError::Timeout)
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(*attempts.lock().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_retry_max_attempts() {
        let config = RetryConfig {
            max_retries: 2,
            ..Default::default()
        };
        let policy = DefaultRetryPolicy;

        let result =
            retry_with_backoff(&config, &policy, || async { Err(WorkflowError::Timeout) }).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_retry_non_retryable_error() {
        let config = RetryConfig::default();
        let policy = DefaultRetryPolicy;

        let result = retry_with_backoff(&config, &policy, || async {
            Err(WorkflowError::Validation("Invalid input".to_string()))
        })
        .await;

        assert!(result.is_err());
    }
}
