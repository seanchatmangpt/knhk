//! Circuit Breaker - Fault tolerance for external dependencies
//!
//! Provides:
//! - Circuit breaker pattern
//! - Automatic recovery
//! - Failure tracking

use crate::error::{WorkflowError, WorkflowResult};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed (normal operation)
    Closed,
    /// Circuit is open (failing, rejecting requests)
    Open,
    /// Circuit is half-open (testing recovery)
    HalfOpen,
}

/// Circuit breaker for fault tolerance
pub struct CircuitBreaker {
    /// Current state
    state: Arc<RwLock<CircuitState>>,
    /// Failure count
    failure_count: Arc<RwLock<u32>>,
    /// Success count (for half-open)
    success_count: Arc<RwLock<u32>>,
    /// Last failure time
    last_failure: Arc<RwLock<Option<Instant>>>,
    /// Failure threshold
    failure_threshold: u32,
    /// Success threshold (for half-open)
    success_threshold: u32,
    /// Timeout for open state
    timeout: Duration,
}

impl CircuitBreaker {
    /// Create new circuit breaker
    pub fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            success_count: Arc::new(RwLock::new(0)),
            last_failure: Arc::new(RwLock::new(None)),
            failure_threshold,
            success_threshold: 3,
            timeout,
        }
    }

    /// Execute operation with circuit breaker protection
    pub async fn execute<F, Fut, T>(&self, operation: F) -> WorkflowResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = WorkflowResult<T>>,
    {
        // Check circuit state
        let state = *self.state.read().await;

        match state {
            CircuitState::Open => {
                // Check if timeout has passed
                let last_failure = *self.last_failure.read().await;
                if let Some(last) = last_failure {
                    if last.elapsed() >= self.timeout {
                        // Transition to half-open
                        let mut state = self.state.write().await;
                        *state = CircuitState::HalfOpen;
                        let mut success_count = self.success_count.write().await;
                        *success_count = 0;
                    } else {
                        return Err(WorkflowError::ExternalSystem(
                            "Circuit breaker is open".to_string(),
                        ));
                    }
                } else {
                    return Err(WorkflowError::ExternalSystem(
                        "Circuit breaker is open".to_string(),
                    ));
                }
            }
            CircuitState::HalfOpen => {
                // Allow operation to test recovery
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }

        // Execute operation
        match operation().await {
            Ok(result) => {
                // Success - reset failure count
                let mut failure_count = self.failure_count.write().await;
                *failure_count = 0;

                // Update state if half-open
                let state = *self.state.read().await;
                if state == CircuitState::HalfOpen {
                    let mut success_count = self.success_count.write().await;
                    *success_count += 1;

                    if *success_count >= self.success_threshold {
                        // Transition to closed
                        let mut state = self.state.write().await;
                        *state = CircuitState::Closed;
                        *success_count = 0;
                    }
                }

                Ok(result)
            }
            Err(e) => {
                // Failure - increment failure count
                let mut failure_count = self.failure_count.write().await;
                *failure_count += 1;

                let mut last_failure = self.last_failure.write().await;
                *last_failure = Some(Instant::now());

                // Check if threshold exceeded
                if *failure_count >= self.failure_threshold {
                    let mut state = self.state.write().await;
                    *state = CircuitState::Open;
                }

                Err(e)
            }
        }
    }

    /// Get current state
    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }
}
