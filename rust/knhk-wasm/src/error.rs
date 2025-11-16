//! Error types for WASM workflow execution

use thiserror::Error;

/// Result type for WASM operations
pub type WasmResult<T> = Result<T, WasmError>;

/// Errors that can occur during WASM workflow execution
#[derive(Error, Debug, Clone)]
pub enum WasmError {
    /// Workflow parsing failed
    #[error("Failed to parse workflow: {0}")]
    ParseError(String),

    /// Invalid workflow specification
    #[error("Invalid workflow specification: {0}")]
    InvalidSpec(String),

    /// Unsupported workflow pattern
    #[error("Unsupported workflow pattern: {0}")]
    UnsupportedPattern(String),

    /// Workflow execution failed
    #[error("Workflow execution failed: {0}")]
    ExecutionFailed(String),

    /// Workflow execution timeout
    #[error("Workflow execution timeout after {0}ms")]
    ExecutionTimeout(u32),

    /// Task execution failed
    #[error("Task execution failed: {0}")]
    TaskFailed(String),

    /// Validation failed
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// State not found
    #[error("State not found for case: {0}")]
    StateNotFound(String),

    /// Invalid configuration
    #[error("Invalid configuration")]
    InvalidConfig,

    /// No matching branch in choice pattern
    #[error("No matching branch in choice pattern")]
    NoMatchingBranch,

    /// Maximum iterations exceeded in loop
    #[error("Maximum iterations exceeded in loop")]
    MaxIterationsExceeded,

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Host function call failed
    #[error("Host function call failed: {0}")]
    HostFunctionError(String),

    /// JavaScript interop error
    #[error("JavaScript interop error: {0}")]
    JsInteropError(String),

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
}

impl From<serde_json::Error> for WasmError {
    fn from(e: serde_json::Error) -> Self {
        WasmError::SerializationError(e.to_string())
    }
}

impl From<WasmError> for wasm_bindgen::JsValue {
    fn from(e: WasmError) -> Self {
        wasm_bindgen::JsValue::from_str(&e.to_string())
    }
}
