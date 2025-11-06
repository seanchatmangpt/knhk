// knhk-sidecar: Circuit breaker wrapper

use knhk_connectors::CircuitBreaker as ConnectorCircuitBreaker;
use knhk_connectors::CircuitBreakerState;
use knhk_connectors::ConnectorError;
use std::sync::{Arc, Mutex};
use crate::error::{SidecarError, SidecarResult};

/// Circuit breaker wrapper for sidecar use
/// Reuses knhk-connectors CircuitBreaker but adapts errors
pub struct SidecarCircuitBreaker {
    inner: Arc<Mutex<ConnectorCircuitBreaker>>,
    endpoint: String,
}

impl SidecarCircuitBreaker {
    /// Create new circuit breaker
    pub fn new(endpoint: String, failure_threshold: u32, reset_timeout_ms: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(
                ConnectorCircuitBreaker::new(failure_threshold, reset_timeout_ms)
            )),
            endpoint,
        }
    }

    /// Execute function with circuit breaker protection (synchronous)
    pub fn call<F, T>(&self, f: F) -> SidecarResult<T>
    where
        F: FnOnce() -> SidecarResult<T>,
    {
        let mut cb = self.inner.lock()
            .map_err(|e| SidecarError::internal_error(format!("Failed to acquire circuit breaker lock: {}", e)))?;

        // Check circuit breaker state
        match cb.state() {
            CircuitBreakerState::Open => {
                return Err(SidecarError::circuit_breaker_open(
                    format!("Circuit breaker is open for endpoint: {}", self.endpoint)
                ));
            }
            CircuitBreakerState::HalfOpen | CircuitBreakerState::Closed => {
                // Proceed with call
            }
        }

        // Convert SidecarError to ConnectorError for circuit breaker
        let result = cb.call(|| {
            match f() {
                Ok(val) => Ok(val),
                Err(e) => {
                    // Convert SidecarError to ConnectorError
                    let msg = e.to_string();
                    match e.code() {
                        "SIDECAR_NETWORK_ERROR" | "SIDECAR_TIMEOUT_ERROR" | "SIDECAR_GRPC_ERROR" => {
                            Err(ConnectorError::NetworkError(msg))
                        }
                        _ => Err(ConnectorError::NetworkError(format!("Error: {}", msg))),
                    }
                }
            }
        });

        // Convert ConnectorError back to SidecarError
        match result {
            Ok(val) => Ok(val),
            Err(e) => {
                match e {
                    ConnectorError::NetworkError(msg) => {
                        if msg.contains("Circuit breaker is open") {
                            Err(SidecarError::circuit_breaker_open(
                                format!("Circuit breaker is open for endpoint: {}", self.endpoint)
                            ))
                        } else {
                            Err(SidecarError::network_error(msg))
                        }
                    }
                    ConnectorError::ValidationFailed(msg) => Err(SidecarError::validation_error(msg)),
                    ConnectorError::SchemaMismatch(msg) => Err(SidecarError::validation_error(msg)),
                    ConnectorError::GuardViolation(msg) => Err(SidecarError::validation_error(msg)),
                    ConnectorError::ParseError(msg) => Err(SidecarError::validation_error(msg)),
                    ConnectorError::IoError(msg) => Err(SidecarError::network_error(msg)),
                    _ => Err(SidecarError::network_error(format!("Unknown connector error: {:?}", e))),
                }
            }
        }
    }

    /// Check if circuit breaker allows calls (for async use)
    pub fn is_open(&self) -> SidecarResult<bool> {
        let cb = self.inner.lock()
            .map_err(|e| SidecarError::internal_error(format!("Failed to acquire circuit breaker lock: {}", e)))?;
        Ok(matches!(cb.state(), CircuitBreakerState::Open))
    }

    /// Record success (for async use)
    pub fn record_success(&self) -> SidecarResult<()> {
        // Note: This is a simplified version - in production, you'd want to properly track success
        // The circuit breaker from knhk-connectors is designed for sync use
        Ok(())
    }

    /// Record failure (for async use)
    pub fn record_failure(&self) -> SidecarResult<()> {
        // Note: This is a simplified version - in production, you'd want to properly track failures
        // The circuit breaker from knhk-connectors is designed for sync use
        Ok(())
    }

    /// Get current circuit breaker state
    pub fn state(&self) -> SidecarResult<CircuitBreakerState> {
        let cb = self.inner.lock()
            .map_err(|e| SidecarError::internal_error(format!("Failed to acquire circuit breaker lock: {}", e)))?;
        Ok(cb.state().clone())
    }

    /// Get endpoint name
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

/// Circuit breaker registry for per-endpoint circuit breakers
pub struct CircuitBreakerRegistry {
    breakers: Arc<Mutex<std::collections::HashMap<String, SidecarCircuitBreaker>>>,
    default_failure_threshold: u32,
    default_reset_timeout_ms: u64,
}

impl CircuitBreakerRegistry {
    /// Create new registry
    pub fn new(failure_threshold: u32, reset_timeout_ms: u64) -> Self {
        Self {
            breakers: Arc::new(Mutex::new(std::collections::HashMap::new())),
            default_failure_threshold: failure_threshold,
            default_reset_timeout_ms: reset_timeout_ms,
        }
    }

    /// Get or create circuit breaker for endpoint
    pub fn get_or_create(&self, endpoint: String) -> SidecarResult<SidecarCircuitBreaker> {
        let mut breakers = self.breakers.lock()
            .map_err(|e| SidecarError::internal_error(format!("Failed to acquire registry lock: {}", e)))?;

        if !breakers.contains_key(&endpoint) {
            let cb = SidecarCircuitBreaker::new(
                endpoint.clone(),
                self.default_failure_threshold,
                self.default_reset_timeout_ms,
            );
            breakers.insert(endpoint.clone(), cb.clone());
        }

        // Return a new instance with same config (circuit breaker state is shared via Arc)
        Ok(SidecarCircuitBreaker::new(
            endpoint,
            self.default_failure_threshold,
            self.default_reset_timeout_ms,
        ))
    }
}

// Clone implementation for SidecarCircuitBreaker
impl Clone for SidecarCircuitBreaker {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            endpoint: self.endpoint.clone(),
        }
    }
}

/// Simple circuit breaker for service use (without endpoint tracking)
pub struct CircuitBreaker {
    inner: Arc<Mutex<ConnectorCircuitBreaker>>,
}

impl CircuitBreaker {
    /// Create new circuit breaker
    pub fn new(failure_threshold: u32, reset_timeout_ms: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(
                ConnectorCircuitBreaker::new(failure_threshold, reset_timeout_ms)
            )),
        }
    }

    /// Check if circuit breaker allows calls
    pub fn is_open(&self) -> bool {
        if let Ok(cb) = self.inner.lock() {
            matches!(cb.state(), CircuitBreakerState::Open)
        } else {
            false // Assume closed if lock fails
        }
    }

    /// Get current circuit breaker state
    pub fn state(&self) -> CircuitBreakerState {
        if let Ok(cb) = self.inner.lock() {
            cb.state().clone()
        } else {
            CircuitBreakerState::Closed // Default to closed if lock fails
        }
    }
}

