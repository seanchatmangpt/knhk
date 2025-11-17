//! Unified API error types
//!
//! Provides a unified error type that can be converted to transport-specific errors.

use crate::error::WorkflowError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Unified API error type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Optional error details
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    /// Create a new API error
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Create a new API error with details
    pub fn with_details(
        code: impl Into<String>,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: Some(details),
        }
    }

    /// Convert to HTTP status code
    #[cfg(feature = "http")]
    pub fn to_http_status(&self) -> axum::http::StatusCode {
        match self.code.as_str() {
            "NOT_FOUND" => axum::http::StatusCode::NOT_FOUND,
            "BAD_REQUEST" => axum::http::StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR" => axum::http::StatusCode::BAD_REQUEST,
            "INTERNAL_ERROR" => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "TIMEOUT" => axum::http::StatusCode::REQUEST_TIMEOUT,
            "RESOURCE_UNAVAILABLE" => axum::http::StatusCode::SERVICE_UNAVAILABLE,
            _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Convert to gRPC status
    #[cfg(feature = "grpc")]
    pub fn to_grpc_status(&self) -> tonic::Status {
        match self.code.as_str() {
            "NOT_FOUND" => tonic::Status::not_found(&self.message),
            "BAD_REQUEST" => tonic::Status::invalid_argument(&self.message),
            "VALIDATION_ERROR" => tonic::Status::invalid_argument(&self.message),
            "INTERNAL_ERROR" => tonic::Status::internal(&self.message),
            "TIMEOUT" => tonic::Status::deadline_exceeded(&self.message),
            "RESOURCE_UNAVAILABLE" => tonic::Status::unavailable(&self.message),
            _ => tonic::Status::internal(&self.message),
        }
    }

    #[cfg(not(feature = "grpc"))]
    pub fn to_grpc_status(&self) -> String {
        format!("{}: {}", self.code, self.message)
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

/// Convert WorkflowError to ApiError
impl From<WorkflowError> for ApiError {
    fn from(err: WorkflowError) -> Self {
        match err {
            WorkflowError::CaseNotFound(id) => {
                ApiError::new("NOT_FOUND", format!("Case {} not found", id))
            }
            WorkflowError::CaseExists(id) => {
                ApiError::new("BAD_REQUEST", format!("Case {} already exists", id))
            }
            WorkflowError::InvalidSpecification(msg) => ApiError::new(
                "BAD_REQUEST",
                format!("Invalid workflow specification: {}", msg),
            ),
            WorkflowError::PatternNotFound(id) => {
                ApiError::new("NOT_FOUND", format!("Pattern {} not found", id))
            }
            WorkflowError::InvalidStateTransition { from, to } => ApiError::new(
                "BAD_REQUEST",
                format!("Invalid state transition from {} to {}", from, to),
            ),
            WorkflowError::Validation(msg) => ApiError::new("VALIDATION_ERROR", msg),
            WorkflowError::TaskExecutionFailed(msg) => {
                ApiError::new("INTERNAL_ERROR", format!("Task execution failed: {}", msg))
            }
            WorkflowError::CancellationFailed(msg) => {
                ApiError::new("INTERNAL_ERROR", format!("Cancellation failed: {}", msg))
            }
            WorkflowError::StatePersistence(msg) => ApiError::new(
                "INTERNAL_ERROR",
                format!("State persistence error: {}", msg),
            ),
            WorkflowError::ExternalSystem(msg) => {
                ApiError::new("INTERNAL_ERROR", format!("External system error: {}", msg))
            }
            WorkflowError::Timeout => ApiError::new("TIMEOUT", "Operation timed out"),
            WorkflowError::ResourceUnavailable(msg) => ApiError::new(
                "RESOURCE_UNAVAILABLE",
                format!("Resource not available: {}", msg),
            ),
            WorkflowError::Internal(msg) => ApiError::new("INTERNAL_ERROR", msg),
            WorkflowError::Parse(msg) => {
                ApiError::new("BAD_REQUEST", format!("Parse error: {}", msg))
            }
            WorkflowError::HookFailed(msg) => {
                ApiError::new("INTERNAL_ERROR", format!("Hook execution failed: {}", msg))
            }
            WorkflowError::GuardViolation(msg) => {
                ApiError::new("VALIDATION_ERROR", format!("Guard violation: {}", msg))
            }
            WorkflowError::ReceiptGenerationFailed(msg) => ApiError::new(
                "INTERNAL_ERROR",
                format!("Receipt generation failed: {}", msg),
            ),
            WorkflowError::SnapshotError(msg) => {
                ApiError::new("INTERNAL_ERROR", format!("Snapshot error: {}", msg))
            }
            WorkflowError::Io(msg) => ApiError::new("INTERNAL_ERROR", format!("IO error: {}", msg)),
            WorkflowError::Crypto(msg) => {
                ApiError::new("INTERNAL_ERROR", format!("Cryptographic error: {}", msg))
            }
        }
    }
}

/// Result type for API operations
pub type ApiResult<T> = Result<T, ApiError>;
