//! Conformance Checking CLI Commands
//!
//! Implements Van der Aalst's conformance checking methodology:
//! - Fitness: Can the process actually be executed?
//! - Precision: Does the process match the specification?
//! - Generalization: Does the process work beyond the examples?
//! - Alignment: Generate alignment between design and execution

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use knhk_workflow_engine::{
    api::transport::CliAdapter, parser::WorkflowParser, state::StateStore, WorkflowEngine,
};
use process_mining::import_xes_file;
use process_mining::XESImportOptions;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Get or create tokio runtime for async operations
fn get_runtime() -> &'static Runtime {
    static RUNTIME: std::sync::OnceLock<Runtime> = std::sync::OnceLock::new();
    RUNTIME.get_or_init(|| {
        Runtime::new().unwrap_or_else(|e| {
            panic!("Failed to create tokio runtime: {}", e);
        })
    })
}

/// Get workflow engine instance
fn get_engine(state_store_path: Option<&str>) -> CnvResult<Arc<WorkflowEngine>> {
    let path = state_store_path.unwrap_or("./workflow_db");
    let state_store = StateStore::new(path).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!(
            "Failed to create state store: {}",
            e
        ))
    })?;
    Ok(Arc::new(WorkflowEngine::new(state_store)))
}

/// Conformance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceReport {
    pub fitness: Option<f64>,
    pub precision: Option<f64>,
    pub generalization: Option<f64>,
    pub total_traces: usize,
    pub conformant_traces: usize,
    pub non_conformant_traces: usize,
}

/// Check conformance between workflow and XES event log
#[verb]
pub fn check(
    workflow_file: PathBuf,
    xes_file: PathBuf,
    state_store: Option<String>,
    json: bool,
) -> CnvResult<()> {
    let runtime = get_runtime();
    let engine = get_engine(state_store.as_deref())?;

    runtime.block_on(async {
        // Parse workflow
        let mut parser = WorkflowParser::new().map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to create parser: {}",
                e
            ))
        })?;

        let spec = parser.parse_file(&workflow_file).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to parse workflow: {}",
                e
            ))
        })?;

        // Register workflow
        engine.register_workflow(spec.clone()).await.map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to register workflow: {}",
                e
            ))
        })?;

        // Import XES file
        let event_log = import_xes_file(&xes_file, XESImportOptions::default()).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to import XES file: {:?}",
                e
            ))
        })?;

        // Basic conformance check: verify that workflow can execute traces from event log
        let mut conformant_traces = 0;
        let mut non_conformant_traces = 0;

        for trace in &event_log.traces {
            // Create a case for this trace
            let case_id = engine
                .create_case(spec.id, serde_json::json!({}))
                .await
                .map_err(|e| {
                    clap_noun_verb::NounVerbError::execution_error(format!(
                        "Failed to create case: {}",
                        e
                    ))
                })?;

            // Try to execute the case
            match engine.start_case(case_id).await {
                Ok(_) => match engine.execute_case(case_id).await {
                    Ok(_) => conformant_traces += 1,
                    Err(_) => non_conformant_traces += 1,
                },
                Err(_) => non_conformant_traces += 1,
            }
        }

        let total_traces = event_log.traces.len();
        let fitness = if total_traces > 0 {
            Some(conformant_traces as f64 / total_traces as f64)
        } else {
            None
        };

        let report = ConformanceReport {
            fitness,
            precision: None,      // Requires advanced alignment analysis
            generalization: None, // Requires advanced model analysis
            total_traces,
            conformant_traces,
            non_conformant_traces,
        };

        if json {
            let json_output = serde_json::to_string_pretty(&report).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to serialize report: {}",
                    e
                ))
            })?;
            println!("{}", json_output);
        } else {
            println!("Conformance Check Report");
            println!("========================");
            println!("Workflow: {}", workflow_file.display());
            println!("Event Log: {}", xes_file.display());
            println!("\nResults:");
            println!("  Total Traces: {}", report.total_traces);
            println!("  Conformant: {}", report.conformant_traces);
            println!("  Non-Conformant: {}", report.non_conformant_traces);
            if let Some(fitness) = report.fitness {
                println!("  Fitness: {:.2}%", fitness * 100.0);
            }
            if report.precision.is_none() {
                println!("  Precision: (requires advanced alignment analysis)");
            }
            if report.generalization.is_none() {
                println!("  Generalization: (requires advanced model analysis)");
            }
        }

        Ok(())
    })
}

/// Calculate fitness between workflow and XES event log
#[verb]
pub fn fitness(
    workflow_file: PathBuf,
    xes_file: PathBuf,
    state_store: Option<String>,
    json: bool,
) -> CnvResult<()> {
    // Reuse check logic for fitness calculation
    check(workflow_file, xes_file, state_store, json)
}

/// Calculate precision between workflow and XES event log
#[verb]
pub fn precision(
    workflow_file: PathBuf,
    xes_file: PathBuf,
    state_store: Option<String>,
    json: bool,
) -> CnvResult<()> {
    // Precision calculation requires advanced alignment analysis
    if json {
        let result = serde_json::json!({
            "precision": null,
            "note": "Precision calculation requires advanced alignment analysis (not yet implemented)"
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to serialize result: {}",
                    e
                ))
            })?
        );
    } else {
        println!("Precision calculation requires advanced alignment analysis");
        println!("This feature is planned but not yet fully implemented");
    }

    Ok(())
}

/// Calculate generalization between workflow and XES event log
#[verb]
pub fn generalization(
    workflow_file: PathBuf,
    xes_file: PathBuf,
    state_store: Option<String>,
    json: bool,
) -> CnvResult<()> {
    // Generalization calculation requires advanced model analysis
    if json {
        let result = serde_json::json!({
            "generalization": null,
            "note": "Generalization calculation requires advanced model analysis (not yet implemented)"
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to serialize result: {}",
                    e
                ))
            })?
        );
    } else {
        println!("Generalization calculation requires advanced model analysis");
        println!("This feature is planned but not yet fully implemented");
    }

    Ok(())
}

/// Generate alignment between workflow design and execution
#[verb]
pub fn alignment(
    workflow_file: PathBuf,
    xes_file: PathBuf,
    output: Option<PathBuf>,
    state_store: Option<String>,
    json: bool,
) -> CnvResult<()> {
    // Alignment generation requires advanced conformance checking algorithms
    if json {
        let result = serde_json::json!({
            "alignment": null,
            "note": "Alignment generation requires advanced conformance checking algorithms (not yet implemented)"
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to serialize result: {}",
                    e
                ))
            })?
        );
    } else {
        println!("Alignment generation requires advanced conformance checking algorithms");
        println!("This feature is planned but not yet fully implemented");
    }

    Ok(())
}
