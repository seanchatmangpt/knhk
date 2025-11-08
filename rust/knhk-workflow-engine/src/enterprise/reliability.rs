//! Enterprise Reliability
//!
//! Provides reliability features for Fortune 5 deployments:
//! - SLO tracking and enforcement
//! - Circuit breakers
//! - Retry policies
//! - Promotion gates

use crate::error::WorkflowResult;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Reliability configuration
#[derive(Debug, Clone)]
pub struct ReliabilityConfig {
    /// SLO targets (operation -> target latency in ms)
    pub slo_targets: HashMap<String, u64>,
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,
    /// Circuit breaker timeout (seconds)
    pub circuit_breaker_timeout: u64,
    /// Retry max attempts
    pub retry_max_attempts: u32,
    /// Retry backoff multiplier
    pub retry_backoff_multiplier: f64,
    /// Enable promotion gates
    pub enable_promotion_gates: bool,
}

impl Default for ReliabilityConfig {
    fn default() -> Self {
        Self {
            slo_targets: HashMap::from([
                ("workflow.create".to_string(), 100),
                ("workflow.execute".to_string(), 500),
                ("pattern.execute".to_string(), 50),
            ]),
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout: 60,
            retry_max_attempts: 3,
            retry_backoff_multiplier: 2.0,
            enable_promotion_gates: true,
        }
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker for workflow operations
#[derive(Clone)]
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    threshold: u32,
    timeout: Duration,
}

impl CircuitBreaker {
    /// Create new circuit breaker
    pub fn new(threshold: u32, timeout_secs: u64) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            threshold,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Check if circuit is open
    pub async fn is_open(&self) -> bool {
        let state = *self.state.read().await;
        if state == CircuitState::Open {
            // Check if timeout has passed
            let last_failure = *self.last_failure_time.read().await;
            if let Some(time) = last_failure {
                if time.elapsed() >= self.timeout {
                    // Transition to half-open
                    *self.state.write().await = CircuitState::HalfOpen;
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// Record success
    pub async fn record_success(&self) {
        let mut state = self.state.write().await;
        let mut failure_count = self.failure_count.write().await;

        match *state {
            CircuitState::HalfOpen | CircuitState::Closed => {
                *failure_count = 0;
                *state = CircuitState::Closed;
            }
            CircuitState::Open => {}
        }
    }

    /// Record failure
    pub async fn record_failure(&self) {
        let mut state = self.state.write().await;
        let mut failure_count = self.failure_count.write().await;
        *self.last_failure_time.write().await = Some(Instant::now());

        *failure_count += 1;
        if *failure_count >= self.threshold {
            *state = CircuitState::Open;
        }
    }
}

/// Reliability manager for workflow engine
pub struct ReliabilityManager {
    config: ReliabilityConfig,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
}

impl ReliabilityManager {
    /// Create new reliability manager
    pub fn new(config: ReliabilityConfig) -> Self {
        Self {
            config,
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create circuit breaker for operation
    pub async fn get_circuit_breaker(&self, operation: &str) -> CircuitBreaker {
        let mut breakers = self.circuit_breakers.write().await;
        breakers
            .entry(operation.to_string())
            .or_insert_with(|| {
                CircuitBreaker::new(
                    self.config.circuit_breaker_threshold,
                    self.config.circuit_breaker_timeout,
                )
            })
            .clone()
    }

    /// Check SLO compliance
    pub fn check_slo(&self, operation: &str, latency_ms: u64) -> bool {
        if let Some(target) = self.config.slo_targets.get(operation) {
            latency_ms <= *target
        } else {
            true // No SLO defined, consider compliant
        }
    }

    /// Execute with retry
    pub async fn execute_with_retry<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> Result<T, E>,
        E: std::fmt::Debug,
    {
        let mut attempt = 0;
        let mut backoff = 1.0;

        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempt += 1;
                    if attempt >= self.config.retry_max_attempts {
                        return Err(e);
                    }
                    // Exponential backoff
                    let delay = Duration::from_millis((backoff * 100.0) as u64);
                    tokio::time::sleep(delay).await;
                    backoff *= self.config.retry_backoff_multiplier;
                }
            }
        }
    }
}
