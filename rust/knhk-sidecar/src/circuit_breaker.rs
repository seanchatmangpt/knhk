// rust/knhk-sidecar/src/circuit_breaker.rs
// Circuit breaker implementation for sidecar resilience

use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitBreakerState>>,
    failure_count: Arc<Mutex<u32>>,
    failure_threshold: u32,
    success_count: Arc<Mutex<u32>>,
    success_threshold: u32,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    reset_timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, reset_timeout_ms: u64) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitBreakerState::Closed)),
            failure_count: Arc::new(Mutex::new(0)),
            failure_threshold,
            success_count: Arc::new(Mutex::new(0)),
            success_threshold: 1,
            last_failure_time: Arc::new(Mutex::new(None)),
            reset_timeout: Duration::from_millis(reset_timeout_ms),
        }
    }

    pub async fn call<F, Fut, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: From<crate::error::SidecarError>,
    {
        let mut state = self.state.lock().await;
        let current_state = state.clone();

        match current_state {
            CircuitBreakerState::Open => {
                let last_failure = *self.last_failure_time.lock().await;
                if let Some(failure_time) = last_failure {
                    if failure_time.elapsed() >= self.reset_timeout {
                        *state = CircuitBreakerState::HalfOpen;
                        *self.success_count.lock().await = 0;
                    } else {
                        return Err(crate::error::SidecarError::CircuitBreakerOpen(
                            "Circuit breaker is open".to_string(),
                        )
                        .into());
                    }
                } else {
                    return Err(crate::error::SidecarError::CircuitBreakerOpen(
                        "Circuit breaker is open".to_string(),
                    )
                    .into());
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Already in half-open, proceed
            }
            CircuitBreakerState::Closed => {
                // Normal operation
            }
        }

        drop(state);

        match f().await {
            Ok(result) => {
                let mut state = self.state.lock().await;
                let mut success_count = self.success_count.lock().await;
                *success_count += 1;

                if *state == CircuitBreakerState::HalfOpen {
                    if *success_count >= self.success_threshold {
                        *state = CircuitBreakerState::Closed;
                        *self.failure_count.lock().await = 0;
                    }
                }

                Ok(result)
            }
            Err(e) => {
                let mut state = self.state.lock().await;
                let mut failure_count = self.failure_count.lock().await;
                *failure_count += 1;
                *self.last_failure_time.lock().await = Some(Instant::now());

                if *failure_count >= self.failure_threshold {
                    *state = CircuitBreakerState::Open;
                }

                Err(e)
            }
        }
    }

    pub async fn state(&self) -> CircuitBreakerState {
        self.state.lock().await.clone()
    }

    pub async fn reset(&self) {
        let mut state = self.state.lock().await;
        *state = CircuitBreakerState::Closed;
        *self.failure_count.lock().await = 0;
        *self.success_count.lock().await = 0;
        *self.last_failure_time.lock().await = None;
    }
}

