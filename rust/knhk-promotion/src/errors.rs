//! Error types for promotion pipeline

use thiserror::Error;
use knhk_ontology::{SnapshotError, ValidationError};

/// Errors that can occur during promotion
#[derive(Error, Debug, Clone)]
pub enum PromotionError {
    /// Snapshot not production-ready
    #[error("Snapshot not production-ready: {0}")]
    NotProductionReady(String),

    /// Invariants not preserved
    #[error("Invariants not preserved: {0}")]
    InvariantsViolated(String),

    /// Compilation artifacts not ready
    #[error("Compilation not complete: {0}")]
    CompilationNotReady(String),

    /// Atomic operation failed
    #[error("Atomic promotion failed: {0}")]
    AtomicOperationFailed(String),

    /// Snapshot error
    #[error("Snapshot error: {0}")]
    Snapshot(#[from] SnapshotError),

    /// Validation error
    #[error("Validation failed: {0}")]
    Validation(String),

    /// Hot path not initialized
    #[error("Hot path not initialized - call init_hot_path() first")]
    HotPathNotInitialized,

    /// Telemetry error
    #[error("Telemetry error: {0}")]
    Telemetry(String),
}

impl From<ValidationError> for PromotionError {
    fn from(e: ValidationError) -> Self {
        PromotionError::Validation(format!("{}: {}", e.code, e.message))
    }
}

/// Result type for promotion operations
pub type Result<T> = std::result::Result<T, PromotionError>;
