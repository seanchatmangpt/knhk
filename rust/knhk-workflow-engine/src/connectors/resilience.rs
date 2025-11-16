// Resilience Patterns: Retry and Circuit Breaker
//
// Implements resilience patterns for fault-tolerant connector operations.
// - RetryPolicy: Exponential backoff with jitter
// - CircuitBreaker: Prevents cascading failures

use crate::connectors::error::{RetryError, CircuitBreakerError};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn, info};

/// Backoff strategy for retries
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed { delay_ms: u64 },
    /// Exponential backoff: delay = base * multiplier^attempt
    Exponential { base_ms: u64, multiplier: f64, max_delay_ms: u64 },
    /// Linear backoff: delay = base + (increment * attempt)
    Linear { base_ms: u64, increment_ms: u64 },
}

impl BackoffStrategy {
    /// Calculate delay for the given attempt number
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        match self {
            Self::Fixed { delay_ms } => Duration::from_millis(*delay_ms),
            Self::Exponential { base_ms, multiplier, max_delay_ms } => {
                let delay = (*base_ms as f64) * multiplier.powi(attempt as i32);
                let capped_delay = delay.min(*max_delay_ms as f64) as u64;
                Duration::from_millis(capped_delay)
            }
            Self::Linear { base_ms, increment_ms } => {
                let delay = base_ms + (increment_ms * attempt as u64);
                Duration::from_millis(delay)
            }
        }
    }

    /// Add jitter to the delay (±25%)
    pub fn with_jitter(&self, attempt: u32) -> Duration {
        let base_delay = self.calculate_delay(attempt);
        let jitter_range = base_delay.as_millis() / 4; // ±25%
        let jitter = rand::random::<u64>() % (jitter_range as u64 * 2);
        let adjusted = base_delay.as_millis() as u64 + jitter - jitter_range as u64;
        Duration::from_millis(adjusted)
    }
}

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub backoff: BackoffStrategy,
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff: BackoffStrategy::Exponential {
                base_ms: 100,
                multiplier: 2.0,
                max_delay_ms: 10000,
            },
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Execute a function with retry logic
    pub async fn execute<F, Fut, T, E>(&self, mut f: F) -> Result<T, RetryError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let mut attempt = 0;

        loop {
            match f().await {
                Ok(result) => {
                    if attempt > 0 {
                        info!(attempt = attempt, "Operation succeeded after retry");
                    }
                    return Ok(result);
                }
                Err(err) => {
                    attempt += 1;

                    if attempt > self.max_retries {
                        warn!(
                            max_retries = self.max_retries,
                            error = %err,
                            "Max retries exceeded"
                        );
                        return Err(RetryError::MaxRetriesExceeded(self.max_retries));
                    }

                    let delay = if self.jitter {
                        self.backoff.with_jitter(attempt - 1)
                    } else {
                        self.backoff.calculate_delay(attempt - 1)
                    };

                    debug!(
                        attempt = attempt,
                        max_retries = self.max_retries,
                        delay_ms = delay.as_millis(),
                        error = %err,
                        "Retrying after error"
                    );

                    sleep(delay).await;
                }
            }
        }
    }

    /// Execute with custom retry decision function
    pub async fn execute_with_predicate<F, Fut, P, T, E>(
        &self,
        mut f: F,
        mut should_retry: P,
    ) -> Result<T, RetryError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        P: FnMut(&E) -> bool,
        E: std::error::Error + Send + Sync + 'static,
    {
        let mut attempt = 0;

        loop {
            match f().await {
                Ok(result) => {
                    if attempt > 0 {
                        info!(attempt = attempt, "Operation succeeded after retry");
                    }
                    return Ok(result);
                }
                Err(err) => {
                    if !should_retry(&err) {
                        warn!(error = %err, "Permanent failure detected, not retrying");
                        return Err(RetryError::PermanentFailure(err.to_string()));
                    }

                    attempt += 1;

                    if attempt > self.max_retries {
                        warn!(
                            max_retries = self.max_retries,
                            error = %err,
                            "Max retries exceeded"
                        );
                        return Err(RetryError::MaxRetriesExceeded(self.max_retries));
                    }

                    let delay = if self.jitter {
                        self.backoff.with_jitter(attempt - 1)
                    } else {
                        self.backoff.calculate_delay(attempt - 1)
                    };

                    debug!(
                        attempt = attempt,
                        max_retries = self.max_retries,
                        delay_ms = delay.as_millis(),
                        error = %err,
                        "Retrying after error"
                    );

                    sleep(delay).await;
                }
            }
        }
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CircuitState {
    Closed = 0,   // Normal operation
    Open = 1,     // Failing, reject requests
    HalfOpen = 2, // Testing if service recovered
}

impl From<u8> for CircuitState {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Closed,
            1 => Self::Open,
            2 => Self::HalfOpen,
            _ => Self::Closed,
        }
    }
}

/// Circuit breaker pattern implementation
///
/// Prevents cascading failures by stopping requests to failing services.
/// States: Closed (normal) -> Open (failing) -> HalfOpen (testing) -> Closed
#[derive(Debug)]
pub struct CircuitBreaker {
    state: Arc<AtomicU8>,
    failure_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU32>,
    threshold: u32,
    half_open_max_calls: u32,
    timeout: Duration,
    last_failure_time: Arc<tokio::sync::RwLock<Option<std::time::Instant>>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(threshold: u32, timeout: Duration) -> Self {
        Self {
            state: Arc::new(AtomicU8::new(CircuitState::Closed as u8)),
            failure_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU32::new(0)),
            threshold,
            half_open_max_calls: 3,
            timeout,
            last_failure_time: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Get current circuit state
    pub fn state(&self) -> CircuitState {
        self.state.load(Ordering::Acquire).into()
    }

    /// Execute a function through the circuit breaker
    pub async fn call<F, Fut, T, E>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        match self.state() {
            CircuitState::Open => {
                // Check if timeout has elapsed
                let should_attempt_reset = {
                    let last_failure = self.last_failure_time.read().await;
                    match *last_failure {
                        Some(time) => time.elapsed() >= self.timeout,
                        None => true,
                    }
                };

                if should_attempt_reset {
                    info!("Circuit breaker transitioning from Open to HalfOpen");
                    self.state.store(CircuitState::HalfOpen as u8, Ordering::Release);
                    self.success_count.store(0, Ordering::Release);
                } else {
                    debug!("Circuit breaker is open, rejecting request");
                    return Err(CircuitBreakerError::Open);
                }
            }
            CircuitState::HalfOpen => {
                // Limit number of test calls
                if self.success_count.load(Ordering::Acquire) >= self.half_open_max_calls {
                    debug!("Circuit breaker half-open limit reached");
                    return Err(CircuitBreakerError::Open);
                }
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }

        // Execute the function
        match f().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(err) => {
                self.on_failure().await;
                Err(CircuitBreakerError::Failure(Box::new(err)))
            }
        }
    }

    /// Record a successful call
    async fn on_success(&self) {
        match self.state() {
            CircuitState::HalfOpen => {
                let successes = self.success_count.fetch_add(1, Ordering::AcqRel) + 1;
                if successes >= self.half_open_max_calls {
                    info!("Circuit breaker transitioning from HalfOpen to Closed");
                    self.reset().await;
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Release);
            }
            CircuitState::Open => {
                // Should not happen
            }
        }
    }

    /// Record a failed call
    async fn on_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::AcqRel) + 1;

        match self.state() {
            CircuitState::Closed => {
                if failures >= self.threshold {
                    warn!(
                        failures = failures,
                        threshold = self.threshold,
                        "Circuit breaker opening due to failures"
                    );
                    self.trip().await;
                }
            }
            CircuitState::HalfOpen => {
                warn!("Circuit breaker test failed, reopening");
                self.trip().await;
            }
            CircuitState::Open => {
                // Already open
            }
        }
    }

    /// Trip the circuit breaker (transition to Open)
    async fn trip(&self) {
        self.state.store(CircuitState::Open as u8, Ordering::Release);
        *self.last_failure_time.write().await = Some(std::time::Instant::now());
    }

    /// Reset the circuit breaker (transition to Closed)
    async fn reset(&self) {
        self.state.store(CircuitState::Closed as u8, Ordering::Release);
        self.failure_count.store(0, Ordering::Release);
        self.success_count.store(0, Ordering::Release);
        *self.last_failure_time.write().await = None;
    }

    /// Get failure count
    pub fn failure_count(&self) -> u32 {
        self.failure_count.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_fixed() {
        let backoff = BackoffStrategy::Fixed { delay_ms: 100 };
        assert_eq!(backoff.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(backoff.calculate_delay(5), Duration::from_millis(100));
    }

    #[test]
    fn test_backoff_exponential() {
        let backoff = BackoffStrategy::Exponential {
            base_ms: 100,
            multiplier: 2.0,
            max_delay_ms: 10000,
        };
        assert_eq!(backoff.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(backoff.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(backoff.calculate_delay(2), Duration::from_millis(400));
        assert_eq!(backoff.calculate_delay(10), Duration::from_millis(10000)); // Capped
    }

    #[test]
    fn test_backoff_linear() {
        let backoff = BackoffStrategy::Linear {
            base_ms: 100,
            increment_ms: 50,
        };
        assert_eq!(backoff.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(backoff.calculate_delay(1), Duration::from_millis(150));
        assert_eq!(backoff.calculate_delay(2), Duration::from_millis(200));
    }

    #[tokio::test]
    async fn test_retry_policy_success() {
        let policy = RetryPolicy::default();
        let mut attempts = 0;

        let result = policy
            .execute(|| async {
                attempts += 1;
                if attempts < 3 {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, "fail"))
                } else {
                    Ok(42)
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 3);
    }

    #[tokio::test]
    async fn test_retry_policy_max_retries() {
        let policy = RetryPolicy {
            max_retries: 2,
            backoff: BackoffStrategy::Fixed { delay_ms: 10 },
            jitter: false,
        };

        let mut attempts = 0;

        let result = policy
            .execute(|| async {
                attempts += 1;
                Err::<(), _>(std::io::Error::new(std::io::ErrorKind::Other, "fail"))
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempts, 3); // Initial + 2 retries
    }

    #[tokio::test]
    async fn test_circuit_breaker_trip() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(1));

        assert_eq!(cb.state(), CircuitState::Closed);

        // Cause 3 failures
        for _ in 0..3 {
            let _ = cb
                .call(|| async { Err::<(), _>(std::io::Error::new(std::io::ErrorKind::Other, "fail")) })
                .await;
        }

        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(100));

        // Trip the circuit
        for _ in 0..2 {
            let _ = cb
                .call(|| async { Err::<(), _>(std::io::Error::new(std::io::ErrorKind::Other, "fail")) })
                .await;
        }

        assert_eq!(cb.state(), CircuitState::Open);

        // Wait for timeout
        sleep(Duration::from_millis(150)).await;

        // Next call should transition to HalfOpen
        let _ = cb.call(|| async { Ok::<_, std::io::Error>(()) }).await;

        // After successful calls, should close
        for _ in 0..2 {
            let _ = cb.call(|| async { Ok::<_, std::io::Error>(()) }).await;
        }

        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(1));

        // Cause failures but not enough to trip
        for _ in 0..2 {
            let _ = cb
                .call(|| async { Err::<(), _>(std::io::Error::new(std::io::ErrorKind::Other, "fail")) })
                .await;
        }

        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.failure_count(), 2);

        // Success should reset failure count
        let _ = cb.call(|| async { Ok::<_, std::io::Error>(()) }).await;
        assert_eq!(cb.failure_count(), 0);
    }
}
