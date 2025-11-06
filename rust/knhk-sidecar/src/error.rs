// knhk-sidecar: Error types

use thiserror::Error;

/// Sidecar result type
pub type SidecarResult<T> = Result<T, SidecarError>;

/// Sidecar error types
#[derive(Debug, Error)]
pub enum SidecarError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Request timeout: {0}")]
    TimeoutError(String),

    #[error("Circuit breaker is open: {0}")]
    CircuitBreakerOpen(String),

    #[error("TLS error: {0}")]
    TlsError(String),

    #[error("Batch error: {0}")]
    BatchError(String),

    #[error("Retry exhausted: {0}")]
    RetryExhausted(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("gRPC error: {0}")]
    GrpcError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

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

/// Check if error is retryable (transient)
pub fn is_retryable_error(err: &SidecarError) -> bool {
    matches!(
        err,
        SidecarError::NetworkError(_)
            | SidecarError::TimeoutError(_)
            | SidecarError::CircuitBreakerOpen(_)
            | SidecarError::GrpcError(_)
    )
}

/// Check if error is a guard violation (non-retryable)
pub fn is_guard_violation(err: &SidecarError) -> bool {
    matches!(
        err,
        SidecarError::ValidationError(_) | SidecarError::BatchError(_)
    )
}

