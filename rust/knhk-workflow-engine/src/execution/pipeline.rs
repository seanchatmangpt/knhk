//! Execution Pipeline - Pattern composition and optimization
//!
//! Provides:
//! - Pattern composition
//! - Execution optimization
//! - Pipeline stages

use crate::error::WorkflowResult;
use crate::patterns::{PatternExecutionContext, PatternId, PatternRegistry};

/// Execution pipeline for pattern composition
pub struct ExecutionPipeline {
    /// Pipeline stages
    stages: Vec<Box<dyn PipelineStage>>,
}

/// Pipeline stage trait
pub trait PipelineStage: Send + Sync {
    /// Process execution context through this stage
    fn process(
        &self,
        registry: &PatternRegistry,
        pattern_id: PatternId,
        context: PatternExecutionContext,
    ) -> WorkflowResult<PatternExecutionContext>;
}

impl ExecutionPipeline {
    /// Create new execution pipeline
    pub fn new() -> Self {
        Self { stages: vec![] }
    }

    /// Add a pipeline stage
    pub fn add_stage(&mut self, stage: Box<dyn PipelineStage>) {
        self.stages.push(stage);
    }

    /// Execute through pipeline
    pub fn execute(
        &self,
        registry: &PatternRegistry,
        pattern_id: PatternId,
        context: PatternExecutionContext,
    ) -> WorkflowResult<PatternExecutionContext> {
        let mut ctx = context;

        // Run through pipeline stages
        for stage in &self.stages {
            ctx = stage.process(registry, pattern_id, ctx)?;
        }

        Ok(ctx)
    }
}

impl Default for ExecutionPipeline {
    fn default() -> Self {
        Self::new()
    }
}
