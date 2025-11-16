//! Error types for quantum-inspired optimization

use thiserror::Error;

/// Result type for quantum operations
pub type QuantumResult<T> = Result<T, QuantumError>;

/// Errors that can occur during quantum-inspired optimization
#[derive(Error, Debug, Clone)]
pub enum QuantumError {
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Optimization failed: {0}")]
    OptimizationFailed(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Timeout exceeded: {0}ms")]
    TimeoutExceeded(u64),

    #[error("Convergence failed after {0} iterations")]
    ConvergenceFailed(usize),

    #[error("Invalid energy function: {0}")]
    InvalidEnergyFunction(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl QuantumError {
    /// Create a constraint violation error
    pub fn constraint_violation(msg: impl Into<String>) -> Self {
        Self::ConstraintViolation(msg.into())
    }

    /// Create an invalid configuration error
    pub fn invalid_configuration(msg: impl Into<String>) -> Self {
        Self::InvalidConfiguration(msg.into())
    }

    /// Create an optimization failed error
    pub fn optimization_failed(msg: impl Into<String>) -> Self {
        Self::OptimizationFailed(msg.into())
    }

    /// Create an invalid state error
    pub fn invalid_state(msg: impl Into<String>) -> Self {
        Self::InvalidState(msg.into())
    }

    /// Create a resource exhausted error
    pub fn resource_exhausted(msg: impl Into<String>) -> Self {
        Self::ResourceExhausted(msg.into())
    }

    /// Create a timeout exceeded error
    pub fn timeout_exceeded(ms: u64) -> Self {
        Self::TimeoutExceeded(ms)
    }

    /// Create a convergence failed error
    pub fn convergence_failed(iterations: usize) -> Self {
        Self::ConvergenceFailed(iterations)
    }

    /// Create an invalid energy function error
    pub fn invalid_energy_function(msg: impl Into<String>) -> Self {
        Self::InvalidEnergyFunction(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::InternalError(msg.into())
    }
}
