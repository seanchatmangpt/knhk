#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Circuit breaker for workflow engine resilience

use crate::error::{WorkflowError, WorkflowResult};
use knhk_connectors::CircuitBreaker as ConnectorCircuitBreaker;
use knhk_connectors::CircuitBreakerState;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Circuit breaker for workflow operations
pub struct CircuitBreaker {
    inner: Arc<Mutex<ConnectorCircuitBreaker>>,
    name: String,
    failure_threshold: u32,
    reset_timeout: Duration,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(name: String, failure_threshold: u32, reset_timeout_ms: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ConnectorCircuitBreaker::new(
                failure_threshold,
                reset_timeout_ms,
            ))),
            name,
            failure_threshold,
            reset_timeout: Duration::from_millis(reset_timeout_ms),
        }
    }

    /// Execute a function with circuit breaker protection
    pub fn call<F, T>(&self, f: F) -> WorkflowResult<T>
    where
        F: FnOnce() -> WorkflowResult<T>,
    {
        let mut cb = self.inner.lock().map_err(|e| {
            WorkflowError::Internal(format!("Failed to acquire circuit breaker lock: {}", e))
        })?;

        // Check circuit breaker state
        match cb.state() {
            CircuitBreakerState::Open => {
                return Err(WorkflowError::ExternalSystem(format!(
                    "Circuit breaker is open for {}",
                    self.name
                )));
            }
            CircuitBreakerState::HalfOpen | CircuitBreakerState::Closed => {
                // Proceed with call
            }
        }

        // Convert WorkflowError to ConnectorError for circuit breaker
        let result = cb.call(|| {
            match f() {
                Ok(val) => Ok(val),
                Err(e) => {
                    // Convert WorkflowError to ConnectorError
                    let msg = e.to_string();
                    match e {
                        WorkflowError::ExternalSystem(_)
                        | WorkflowError::Timeout
                        | WorkflowError::ResourceUnavailable(_) => {
                            Err(knhk_connectors::ConnectorError::NetworkError(msg))
                        }
                        _ => Err(knhk_connectors::ConnectorError::NetworkError(format!(
                            "Error: {}",
                            msg
                        ))),
                    }
                }
            }
        });

        // Convert ConnectorError back to WorkflowError
        match result {
            Ok(val) => Ok(val),
            Err(e) => Err(WorkflowError::ExternalSystem(format!("{:?}", e))),
        }
    }

    /// Get current circuit breaker state
    pub fn state(&self) -> CircuitBreakerState {
        self.inner
            .lock()
            .ok()
            .and_then(|cb| Some(cb.state().clone()))
            .unwrap_or_else(|| knhk_connectors::CircuitBreakerState::Open)
    }

    /// Get circuit breaker name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new("default".to_string(), 5, 60000)
    }
}
