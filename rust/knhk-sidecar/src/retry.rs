// rust/knhk-sidecar/src/retry.rs
// Retry logic with exponential backoff

use crate::error::{Result, SidecarError};
use std::time::Duration;
use tokio::time::sleep;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial backoff delay in milliseconds
    pub initial_backoff_ms: u64,
    /// Maximum backoff delay in milliseconds
    pub max_backoff_ms: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 800,
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Validate retry configuration
    pub fn validate(&self) -> Result<()> {
        if self.max_attempts == 0 {
            return Err(SidecarError::ValidationFailed(
                "max_attempts must be > 0".to_string()
            ));
        }
        if self.initial_backoff_ms == 0 {
            return Err(SidecarError::ValidationFailed(
                "initial_backoff_ms must be > 0".to_string()
            ));
        }
        if self.backoff_multiplier <= 1.0 {
            return Err(SidecarError::ValidationFailed(
                "backoff_multiplier must be > 1.0".to_string()
            ));
        }
        Ok(())
    }

    /// Calculate backoff delay for attempt number (0-indexed)
    pub fn backoff_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_millis(self.initial_backoff_ms);
        }
        
        let multiplier = self.backoff_multiplier.powi(attempt as i32 - 1);
        let delay_ms = (self.initial_backoff_ms as f64 * multiplier) as u64;
        let delay_ms = delay_ms.min(self.max_backoff_ms);
        
        Duration::from_millis(delay_ms)
    }
}

/// Retry executor - executes function with retry logic
pub struct RetryExecutor {
    config: RetryConfig,
}

impl RetryExecutor {
    /// Create new retry executor
    pub fn new(config: RetryConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Execute function with retry logic
    /// 
    /// The function is retried with exponential backoff if it returns an error.
    /// Retries are safe because A = μ(O) ensures idempotency.
    pub async fn execute<F, T, E>(&self, mut f: F) -> Result<T>
    where
        F: FnMut() -> std::result::Result<T, E>,
        E: std::fmt::Display,
    {
        let mut last_error: Option<String> = None;
        
        for attempt in 0..self.config.max_attempts {
            match f() {
                Ok(result) => {
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    
                    // Don't sleep after last attempt
                    if attempt < self.config.max_attempts - 1 {
                        let delay = self.config.backoff_delay(attempt);
                        sleep(delay).await;
                    }
                }
            }
        }
        
        Err(SidecarError::RetryExhausted(
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        ))
    }

    /// Execute async function with retry logic
    pub async fn execute_async<F, Fut, T, E>(&self, mut f: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut last_error: Option<String> = None;
        
        for attempt in 0..self.config.max_attempts {
            match f().await {
                Ok(result) => {
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    
                    // Don't sleep after last attempt
                    if attempt < self.config.max_attempts - 1 {
                        let delay = self.config.backoff_delay(attempt);
                        sleep(delay).await;
                    }
                }
            }
        }
        
        Err(SidecarError::RetryExhausted(
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_success() {
        let config = RetryConfig::default();
        let executor = RetryExecutor::new(config).unwrap();
        
        let mut attempts = 0;
        let result = executor.execute(|| {
            attempts += 1;
            if attempts < 2 {
                Err("temporary error")
            } else {
                Ok(42)
            }
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 2);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_backoff_ms: 10,
            max_backoff_ms: 100,
            backoff_multiplier: 2.0,
        };
        let executor = RetryExecutor::new(config).unwrap();
        
        let result = executor.execute(|| Err::<i32, _>("always fails")).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            SidecarError::RetryExhausted(_) => {}
            _ => panic!("Expected RetryExhausted error"),
        }
    }

    #[tokio::test]
    async fn test_backoff_delay() {
        let config = RetryConfig {
            max_attempts: 4,
            initial_backoff_ms: 100,
            max_backoff_ms: 800,
            backoff_multiplier: 2.0,
        };
        
        // Attempt 0: initial delay
        assert_eq!(config.backoff_delay(0), Duration::from_millis(100));
        
        // Attempt 1: 100ms * 2^0 = 100ms
        assert_eq!(config.backoff_delay(1), Duration::from_millis(100));
        
        // Attempt 2: 100ms * 2^1 = 200ms
        assert_eq!(config.backoff_delay(2), Duration::from_millis(200));
        
        // Attempt 3: 100ms * 2^2 = 400ms
        assert_eq!(config.backoff_delay(3), Duration::from_millis(400));
    }

    #[test]
    fn test_retry_config_validation() {
        // Valid config
        let config = RetryConfig::default();
        assert!(config.validate().is_ok());
        
        // Invalid: zero max_attempts
        let config = RetryConfig {
            max_attempts: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        
        // Invalid: zero initial_backoff_ms
        let config = RetryConfig {
            initial_backoff_ms: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        
        // Invalid: backoff_multiplier <= 1.0
        let config = RetryConfig {
            backoff_multiplier: 1.0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }
}

