// knhk-unrdf: Error types and error codes
// Error handling for unrdf integration layer

/// Error types for unrdf integration
#[derive(Debug, thiserror::Error)]
pub enum UnrdfError {
    #[error("Failed to initialize unrdf: {0}")]
    InitializationFailed(String),
    #[error("Query execution failed: {0}")]
    QueryFailed(String),
    #[error("Store operation failed: {0}")]
    StoreFailed(String),
    #[error("Hook execution failed: {0}")]
    HookFailed(String),
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Constitution violation: {0}")]
    ConstitutionViolation(String),
    #[error("Typing violation: {0}")]
    TypingViolation(String),
    #[error("Order violation: {0}")]
    OrderViolation(String),
    #[error("Guard violation: {0}")]
    GuardViolation(String),
    #[error("Invariant violation: {0}")]
    InvariantViolation(String),
    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),
}

/// Error code enumeration for FFI
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnrdfErrorCode {
    Success = 0,
    InitializationFailed = -1,
    QueryFailed = -2,
    StoreFailed = -3,
    HookFailed = -4,
    TransactionFailed = -5,
    ValidationFailed = -6,
    SerializationFailed = -7,
    InvalidInput = -8,
}

impl From<&UnrdfError> for UnrdfErrorCode {
    fn from(err: &UnrdfError) -> Self {
        match err {
            UnrdfError::InitializationFailed(_) => UnrdfErrorCode::InitializationFailed,
            UnrdfError::QueryFailed(_) => UnrdfErrorCode::QueryFailed,
            UnrdfError::StoreFailed(_) => UnrdfErrorCode::StoreFailed,
            UnrdfError::HookFailed(_) => UnrdfErrorCode::HookFailed,
            UnrdfError::TransactionFailed(_) => UnrdfErrorCode::TransactionFailed,
            UnrdfError::ValidationFailed(_) => UnrdfErrorCode::ValidationFailed,
            UnrdfError::SerializationFailed(_) => UnrdfErrorCode::SerializationFailed,
            UnrdfError::InvalidInput(_) => UnrdfErrorCode::InvalidInput,
            UnrdfError::ConstitutionViolation(_) => UnrdfErrorCode::ValidationFailed,
            UnrdfError::TypingViolation(_) => UnrdfErrorCode::ValidationFailed,
            UnrdfError::OrderViolation(_) => UnrdfErrorCode::ValidationFailed,
            UnrdfError::GuardViolation(_) => UnrdfErrorCode::ValidationFailed,
            UnrdfError::InvariantViolation(_) => UnrdfErrorCode::ValidationFailed,
            UnrdfError::LockPoisoned(_) => UnrdfErrorCode::InvalidInput,
        }
    }
}

/// Result type for unrdf operations
pub type UnrdfResult<T> = Result<T, UnrdfError>;
