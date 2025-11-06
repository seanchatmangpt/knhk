// rust/knhk-sidecar/src/circuit_breaker.rs
// Circuit breaker for warm orchestrator connection

use crate::error::{Result, SidecarError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitBreakerState {
    /// Normal operation - requests pass through
    Closed,
    /// Failing - requests rejected immediately
    Open,
    /// Testing recovery - single request allowed
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold before opening circuit
    pub failure_threshold: u32,
    /// Success threshold for half-open recovery
    pub success_threshold: u32,
    /// Reset timeout in milliseconds
    pub reset_timeout_ms: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 1,
            reset_timeout_ms: 60000, // 60 seconds
        }
    }
}

/// Circuit breaker for protecting warm orchestrator connection
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitBreakerState>>,
    config: CircuitBreakerConfig,
    failure_count: Arc<Mutex<u32>>,
    success_count: Arc<Mutex<u32>>,
    last_failure_time: Arc<Mutex<Option<SystemTime>>>,
}

impl CircuitBreaker {
    /// Create new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitBreakerState::Closed)),
            config,
            failure_count: Arc::new(Mutex::new(0)),
            success_count: Arc::new(Mutex::new(0)),
            last_failure_time: Arc::new(Mutex::new(None)),
        }
    }

    /// Execute function with circuit breaker protection
    pub async fn call<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Check circuit state
        let state = {
            let state_guard = self.state.lock()
                .map_err(|e| SidecarError::InternalError(
                    format!("Failed to lock circuit breaker state: {}", e)
                ))?;
            state_guard.clone()
        };

        match state {
            CircuitBreakerState::Open => {
                // Check if reset timeout has passed
                if self.should_attempt_reset().await {
                    // Transition to half-open
                    let mut state_guard = self.state.lock()
                        .map_err(|e| SidecarError::InternalError(
                            format!("Failed to lock circuit breaker state: {}", e)
                        ))?;
                    *state_guard = CircuitBreakerState::HalfOpen;
                    
                    let mut success_guard = self.success_count.lock()
                        .map_err(|e| SidecarError::InternalError(
                            format!("Failed to lock success count: {}", e)
                        ))?;
                    *success_guard = 0;
                } else {
                    return Err(SidecarError::CircuitBreakerOpen);
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Already in half-open, proceed with test
            }
            CircuitBreakerState::Closed => {
                // Normal operation
            }
        }

        // Execute function
        match f().await {
            Ok(result) => {
                self.record_success().await?;
                Ok(result)
            }
            Err(e) => {
                self.record_failure().await?;
                Err(e)
            }
        }
    }

    /// Check if circuit breaker should attempt reset
    async fn should_attempt_reset(&self) -> bool {
        let last_failure = {
            let guard = self.last_failure_time.lock().ok()?;
            guard.clone()
        };
        
        if let Some(last_failure_time) = last_failure {
            let elapsed = last_failure_time.elapsed().unwrap_or(Duration::MAX);
            elapsed.as_millis() as u64 >= self.config.reset_timeout_ms
        } else {
            true
        }
    }

    /// Record successful operation
    async fn record_success(&self) -> Result<()> {
        let state = {
            let state_guard = self.state.lock()
                .map_err(|e| SidecarError::InternalError(
                    format!("Failed to lock circuit breaker state: {}", e)
                ))?;
            state_guard.clone()
        };

        match state {
            CircuitBreakerState::HalfOpen => {
                let mut success_guard = self.success_count.lock()
                    .map_err(|e| SidecarError::InternalError(
                        format!("Failed to lock success count: {}", e)
                    ))?;
                *success_guard += 1;
                
                if *success_guard >= self.config.success_threshold {
                    // Transition to closed
                    let mut state_guard = self.state.lock()
                        .map_err(|e| SidecarError::InternalError(
                            format!("Failed to lock circuit breaker state: {}", e)
                        ))?;
                    *state_guard = CircuitBreakerState::Closed;
                    
                    let mut failure_guard = self.failure_count.lock()
                        .map_err(|e| SidecarError::InternalError(
                            format!("Failed to lock failure count: {}", e)
                        ))?;
                    *failure_guard = 0;
                }
            }
            CircuitBreakerState::Closed => {
                // Reset failure count on success
                let mut failure_guard = self.failure_count.lock()
                    .map_err(|e| SidecarError::InternalError(
                        format!("Failed to lock failure count: {}", e)
                    ))?;
                *failure_guard = 0;
            }
            CircuitBreakerState::Open => {
                // Should not happen, but handle gracefully
            }
        }
        
        Ok(())
    }

    /// Record failed operation
    async fn record_failure(&self) -> Result<()> {
        let mut failure_guard = self.failure_count.lock()
            .map_err(|e| SidecarError::InternalError(
                format!("Failed to lock failure count: {}", e)
            ))?;
        
        *failure_guard += 1;
        
        let mut last_failure_guard = self.last_failure_time.lock()
            .map_err(|e| SidecarError::InternalError(
                format!("Failed to lock last failure time: {}", e)
            ))?;
        *last_failure_guard = Some(SystemTime::now());
        
        if *failure_guard >= self.config.failure_threshold {
            // Transition to open
            let mut state_guard = self.state.lock()
                .map_err(|e| SidecarError::InternalError(
                    format!("Failed to lock circuit breaker state: {}", e)
                ))?;
            *state_guard = CircuitBreakerState::Open;
        }
        
        Ok(())
    }

    /// Get current circuit breaker state
    pub fn state(&self) -> Result<CircuitBreakerState> {
        let state_guard = self.state.lock()
            .map_err(|e| SidecarError::InternalError(
                format!("Failed to lock circuit breaker state: {}", e)
            ))?;
        Ok(state_guard.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_circuit_breaker_closed() {
        let config = CircuitBreakerConfig::default();
        let cb = CircuitBreaker::new(config);
        
        // Successful call
        let result = cb.call(|| async { Ok::<i32, SidecarError>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(cb.state().unwrap(), CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 1,
            reset_timeout_ms: 100,
        };
        let cb = CircuitBreaker::new(config);
        
        // Failures
        for _ in 0..3 {
            let result = cb.call(|| async {
                Err::<i32, _>(SidecarError::NetworkError("test".to_string()))
            }).await;
            assert!(result.is_err());
        }
        
        // Circuit should be open
        assert_eq!(cb.state().unwrap(), CircuitBreakerState::Open);
        
        // Next call should fail immediately
        let result = cb.call(|| async { Ok::<i32, _>(42) }).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SidecarError::CircuitBreakerOpen => {}
            _ => panic!("Expected CircuitBreakerOpen error"),
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            reset_timeout_ms: 50,
        };
        let cb = CircuitBreaker::new(config);
        
        // Failures to open circuit
        for _ in 0..2 {
            let _ = cb.call(|| async {
                Err::<i32, _>(SidecarError::NetworkError("test".to_string()))
            }).await;
        }
        
        assert_eq!(cb.state().unwrap(), CircuitBreakerState::Open);
        
        // Wait for reset timeout
        sleep(Duration::from_millis(60)).await;
        
        // Successful call should transition to half-open, then closed
        let result = cb.call(|| async { Ok::<i32, _>(42) }).await;
        assert!(result.is_ok());
        
        // Should be closed after success
        assert_eq!(cb.state().unwrap(), CircuitBreakerState::Closed);
    }
}

