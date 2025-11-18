// knhk-yawl/src/error.rs
// Error types for YAWL pattern execution

use thiserror::Error;

/// Errors that can occur during YAWL pattern execution
#[derive(Error, Debug, Clone)]
pub enum YawlError {
    /// Pattern validation failed
    #[error("Pattern validation failed: {0}")]
    ValidationFailed(String),

    /// Pattern execution failed
    #[error("Pattern execution failed: {0}")]
    ExecutionFailed(String),

    /// Invalid pattern configuration
    #[error("Invalid pattern configuration: {0}")]
    InvalidConfiguration(String),

    /// Resource allocation failed
    #[error("Resource allocation failed: {0}")]
    ResourceAllocationFailed(String),

    /// Timeout occurred during pattern execution
    #[error("Timeout after {0}ms")]
    Timeout(u64),

    /// Pattern was cancelled
    #[error("Pattern execution cancelled: {0}")]
    Cancelled(String),

    /// Data flow error
    #[error("Data flow error: {0}")]
    DataFlowError(String),

    /// Synchronization error
    #[error("Synchronization error: {0}")]
    SynchronizationError(String),

    /// Loop iteration limit exceeded (Q3: Bounded recursion)
    #[error("Loop iteration limit exceeded: max {0}")]
    IterationLimitExceeded(u32),

    /// Performance constraint violated (Chatman constant)
    #[error("Performance constraint violated: operation took {0} ticks (max 8)")]
    PerformanceViolation(u32),
}

/// Result type for YAWL pattern operations
pub type YawlResult<T> = Result<T, YawlError>;
