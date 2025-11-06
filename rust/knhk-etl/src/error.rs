// rust/knhk-etl/src/error.rs
// Pipeline error types

extern crate alloc;

use alloc::string::String;
use crate::slo_monitor::SloViolation;

/// Pipeline error
#[derive(Debug)]
pub enum PipelineError {
    IngestError(String),
    TransformError(String),
    LoadError(String),
    ReflexError(String),
    EmitError(String),
    GuardViolation(String),
    ParseError(String), // RDF parsing errors from oxigraph
    RuntimeClassError(String), // Runtime class classification failures
    SloViolation(SloViolation), // SLO threshold exceeded
    R1FailureError(String), // R1 failure handling errors
    W1FailureError(String), // W1 failure handling errors
    C1FailureError(String), // C1 failure handling errors
}

// Error conversion handled inline in parsing code

impl PipelineError {
    pub fn message(&self) -> &str {
        match self {
            PipelineError::IngestError(msg) => msg,
            PipelineError::TransformError(msg) => msg,
            PipelineError::LoadError(msg) => msg,
            PipelineError::ReflexError(msg) => msg,
            PipelineError::EmitError(msg) => msg,
            PipelineError::GuardViolation(msg) => msg,
            PipelineError::ParseError(msg) => msg,
            PipelineError::RuntimeClassError(msg) => msg,
            PipelineError::SloViolation(_v) => {
                // Return violation message (requires allocation)
                // For no_std compatibility, return static string
                "SLO violation"
            },
            PipelineError::R1FailureError(msg) => msg,
            PipelineError::W1FailureError(msg) => msg,
            PipelineError::C1FailureError(msg) => msg,
        }
    }
}

