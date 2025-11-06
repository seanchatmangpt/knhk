// rust/knhk-sidecar/src/error.rs
// Error types for KGC Sidecar

use std::fmt;

/// Sidecar error types
#[derive(Debug)]
pub enum SidecarError {
    /// Validation error (guard violation, invalid input)
    ValidationFailed(String),
    /// Network error (connection failed, timeout)
    NetworkError(String),
    /// gRPC error
    GrpcError(String),
    /// Batch error (batch full, flush failed)
    BatchError(String),
    /// Circuit breaker open
    CircuitBreakerOpen,
    /// Retry exhausted
    RetryExhausted(String),
    /// Internal error
    InternalError(String),
}

impl fmt::Display for SidecarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SidecarError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            SidecarError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            SidecarError::GrpcError(msg) => write!(f, "gRPC error: {}", msg),
            SidecarError::BatchError(msg) => write!(f, "Batch error: {}", msg),
            SidecarError::CircuitBreakerOpen => write!(f, "Circuit breaker is open"),
            SidecarError::RetryExhausted(msg) => write!(f, "Retry exhausted: {}", msg),
            SidecarError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for SidecarError {}

impl From<tonic::Status> for SidecarError {
    fn from(status: tonic::Status) -> Self {
        SidecarError::GrpcError(status.message().to_string())
    }
}

impl From<tonic::transport::Error> for SidecarError {
    fn from(err: tonic::transport::Error) -> Self {
        SidecarError::NetworkError(err.to_string())
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, SidecarError>;

