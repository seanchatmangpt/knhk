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
