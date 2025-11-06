// rust/knhk-validation/src/error.rs
// Error types for policy engine and validation

use alloc::string::String;

#[cfg(feature = "policy-engine")]
use thiserror::Error;

#[cfg(feature = "policy-engine")]
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum PolicyError {
    #[error("Policy evaluation failed: {0}")]
    EvaluationFailed(String),
    
    #[error("Policy parse error: {0}")]
    ParseError(String),
    
    #[error("Policy not found: {0}")]
    PolicyNotFound(String),
    
    #[error("Invalid policy input: {0}")]
    InvalidInput(String),
    
    #[error("Rego engine error: {0}")]
    RegoEngineError(String),
}

#[cfg(feature = "policy-engine")]
impl From<regorus::Error> for PolicyError {
    fn from(err: regorus::Error) -> Self {
        PolicyError::RegoEngineError(err.to_string())
    }
}

