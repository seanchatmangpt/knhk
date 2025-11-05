// Error types for unrdf integration

use thiserror::Error;

/// Error types for unrdf integration
#[derive(Debug, Error)]
pub enum UnrdfError {
    #[error("Failed to initialize unrdf: {0}")]
    InitializationFailed(String),
    #[error("Query execution failed: {0}")]
    QueryFailed(String),
    #[error("Store operation failed: {0}")]
    StoreFailed(String),
    #[error("Hook execution failed: {0}")]
    HookFailed(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    #[error("Hook management failed: {0}")]
    HookManagementFailed(String),
    #[error("State management failed: {0}")]
    StateManagementFailed(String),
}

/// Result type for unrdf operations
pub type UnrdfResult<T> = Result<T, UnrdfError>;

