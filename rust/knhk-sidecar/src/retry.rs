// knhk-sidecar: Retry logic with exponential backoff

use tokio::time::{sleep, Duration};
use crate::error::{SidecarError, SidecarResult, is_retryable_error};

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    
    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,
    
    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,
    
    /// Backoff multiplier
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

/// Retry executor
pub struct RetryExecutor {
    config: RetryConfig,
}

impl RetryExecutor {
    /// Create new retry executor
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute function with retry logic
    pub async fn execute<F, Fut, T>(&self, mut f: F) -> SidecarResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = SidecarResult<T>>,
    {
        let mut last_error = None;
        let mut delay_ms = self.config.initial_delay_ms;

        for attempt in 0..=self.config.max_retries {
            match f().await {
                Ok(result) => {
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e.clone());

                    // Don't retry on non-retryable errors
                    if !is_retryable_error(&e) {
                        return Err(e);
                    }

                    // Don't retry if we've exhausted retries
                    if attempt >= self.config.max_retries {
                        return Err(SidecarError::RetryExhausted(
                            format!("Max retries ({}) exceeded. Last error: {}", self.config.max_retries, e)
                        ));
                    }

                    // Wait before retry with exponential backoff
                    sleep(Duration::from_millis(delay_ms)).await;
                    
                    // Calculate next delay
                    delay_ms = ((delay_ms as f64) * self.config.multiplier) as u64;
                    if delay_ms > self.config.max_delay_ms {
                        delay_ms = self.config.max_delay_ms;
                    }
                }
            }
        }

        // Should never reach here, but handle it anyway
        Err(last_error.unwrap_or_else(|| {
            SidecarError::InternalError("Retry execution failed without error".to_string())
        }))
    }

    /// Execute synchronous function with retry logic
    pub fn execute_sync<F, T>(&self, mut f: F) -> SidecarResult<T>
    where
        F: FnMut() -> SidecarResult<T>,
    {
        let mut last_error = None;
        let mut delay_ms = self.config.initial_delay_ms;

        for attempt in 0..=self.config.max_retries {
            match f() {
                Ok(result) => {
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e.clone());

                    // Don't retry on non-retryable errors
                    if !is_retryable_error(&e) {
                        return Err(e);
                    }

                    // Don't retry if we've exhausted retries
                    if attempt >= self.config.max_retries {
                        return Err(SidecarError::RetryExhausted(
                            format!("Max retries ({}) exceeded. Last error: {}", self.config.max_retries, e)
                        ));
                    }

                    // Wait before retry (blocking)
                    std::thread::sleep(Duration::from_millis(delay_ms));
                    
                    // Calculate next delay
                    delay_ms = ((delay_ms as f64) * self.config.multiplier) as u64;
                    if delay_ms > self.config.max_delay_ms {
                        delay_ms = self.config.max_delay_ms;
                    }
                }
            }
        }

        // Should never reach here, but handle it anyway
        Err(last_error.unwrap_or_else(|| {
            SidecarError::InternalError("Retry execution failed without error".to_string())
        }))
    }
}

