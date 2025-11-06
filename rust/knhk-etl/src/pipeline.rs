// rust/knhk-etl/src/pipeline.rs
// Complete ETL pipeline orchestrator

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::{String, ToString};

use crate::error::PipelineError;
use crate::ingest::IngestStage;
use crate::transform::TransformStage;
use crate::load::LoadStage;
use crate::reflex::ReflexStage;
use crate::emit::{EmitStage, EmitResult};

/// Complete ETL pipeline
pub struct Pipeline {
    ingest: IngestStage,
    transform: TransformStage,
    pub load: LoadStage,  // Public for tests
    pub reflex: ReflexStage,  // Public for tests
    emit: EmitStage,
}

impl Pipeline {
    pub fn new(
        connectors: Vec<String>,
        schema_iri: String,
        lockchain_enabled: bool,
        downstream_endpoints: Vec<String>,
    ) -> Self {
        Self {
            ingest: IngestStage::new(connectors, "rdf/turtle".to_string()),
            transform: TransformStage::new(schema_iri, true),
            load: LoadStage::new(),
            reflex: ReflexStage::new(),
            emit: EmitStage::new(lockchain_enabled, downstream_endpoints),
        }
    }

    /// Execute full pipeline
    pub fn execute(&mut self) -> Result<EmitResult, PipelineError> {
        // Stage 1: Ingest
        let ingest_result = self.ingest.ingest()?;

        // Stage 2: Transform
        let transform_result = self.transform.transform(ingest_result)?;

        // Stage 3: Load
        let load_result = self.load.load(transform_result)?;

        // Stage 4: Reflex
        let reflex_result = self.reflex.reflex(load_result)?;

        // Stage 5: Emit
        let emit_result = self.emit.emit(reflex_result)?;

        Ok(emit_result)
    }
}

