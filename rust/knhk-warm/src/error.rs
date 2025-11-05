// knhk-warm error types
// Production-ready error handling with proper error messages

use alloc::string::String;

#[derive(Debug, Clone)]
pub enum WarmPathError {
    InvalidInput(String),
    GuardViolation(String),
    ExecutionFailed(String),
    TimeoutExceeded(String),
}

impl WarmPathError {
    pub fn message(&self) -> &str {
        match self {
            WarmPathError::InvalidInput(msg) => msg,
            WarmPathError::GuardViolation(msg) => msg,
            WarmPathError::ExecutionFailed(msg) => msg,
            WarmPathError::TimeoutExceeded(msg) => msg,
        }
    }
}

