//! Pipeline commands - ETL pipeline operations

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use crate::commands::pipeline as pipeline_impl;

#[derive(Serialize, Debug)]
struct PipelineResult {
    connectors: Option<String>,
    schema: Option<String>,
    success: bool,
}

/// Execute pipeline
#[verb] // Noun "pipeline" auto-inferred from filename "pipeline.rs"
fn run(connectors: Option<String>, schema: Option<String>) -> Result<PipelineResult> {
    pipeline_impl::run(connectors.clone(), schema.clone())
        .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed to run pipeline: {}", e)))
        .map(|_| PipelineResult { connectors, schema, success: true })
}

#[derive(Serialize, Debug)]
struct PipelineStatus {
    status: String,
}

/// Show pipeline status
#[verb] // Noun "pipeline" auto-inferred
fn status() -> Result<PipelineStatus> {
    pipeline_impl::status()
        .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed to get pipeline status: {}", e)))
        .map(|status| PipelineStatus { status })
}

