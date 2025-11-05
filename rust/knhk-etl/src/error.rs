// rust/knhk-etl/src/error.rs
// Pipeline error types

/// Pipeline error
#[derive(Debug)]
pub enum PipelineError {
    IngestError(String),
    TransformError(String),
    LoadError(String),
    ReflexError(String),
    EmitError(String),
    GuardViolation(String),
    ParseError(String), // RDF parsing errors from rio_turtle
}

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
        }
    }
}

