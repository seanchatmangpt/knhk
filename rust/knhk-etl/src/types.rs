// rust/knhk-etl/src/types.rs
// Common types for ETL pipeline

use alloc::string::String;
use alloc::vec::Vec;

/// Pipeline stage identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    Ingest,
    Transform,
    Load,
    Reflex,
    Emit,
}

/// Pipeline metrics
#[derive(Debug, Clone)]
pub struct PipelineMetrics {
    pub stage: PipelineStage,
    pub delta_count: usize,
    pub triples_processed: usize,
    pub ticks_elapsed: u32,
    pub errors: usize,
}

impl Default for PipelineMetrics {
    fn default() -> Self {
        Self {
            stage: PipelineStage::Ingest,
            delta_count: 0,
            triples_processed: 0,
            ticks_elapsed: 0,
            errors: 0,
        }
    }
}

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

