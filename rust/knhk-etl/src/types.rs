// rust/knhk-etl/src/types.rs
// Common types for ETL pipeline

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
#[derive(Debug, Clone, Default)]
pub struct PipelineMetrics {
    pub stage: PipelineStage,
    pub delta_count: usize,
    pub triples_processed: usize,
    pub ticks_elapsed: u32,
    pub errors: usize,
}

