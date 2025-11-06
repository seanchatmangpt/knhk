// rust/knhk-sidecar/src/error.rs
// Error types for KGC Sidecar

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SidecarError {
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Circuit breaker open: {0}")]
    CircuitBreakerOpen(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Hook evaluation failed: {0}")]
    HookEvaluationFailed(String),

    #[error("Query execution failed: {0}")]
    QueryExecutionFailed(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<knhk_etl::PipelineError> for SidecarError {
    fn from(e: knhk_etl::PipelineError) -> Self {
        SidecarError::TransactionFailed(format!("Pipeline error: {:?}", e))
    }
}

impl From<knhk_unrdf::UnrdfError> for SidecarError {
    fn from(e: knhk_unrdf::UnrdfError) -> Self {
        SidecarError::HookEvaluationFailed(format!("Hook error: {:?}", e))
    }
}

impl From<tonic::Status> for SidecarError {
    fn from(s: tonic::Status) -> Self {
        SidecarError::Internal(format!("gRPC error: {}", s))
    }
}

pub type SidecarResult<T> = Result<T, SidecarError>;

