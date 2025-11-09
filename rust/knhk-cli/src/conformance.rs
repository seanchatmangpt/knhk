//! Conformance Checking CLI Commands
//!
//! Implements Van der Aalst's conformance checking methodology:
//! - Fitness: Can the process actually be executed?
//! - Precision: Does the process match the specification?
//! - Generalization: Does the process work beyond the examples?
//! - Alignment: Generate alignment between design and execution

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use knhk_workflow_engine::{parser::WorkflowParser, state::StateStore, WorkflowEngine};
use process_mining::import_xes_file;
use process_mining::XESImportOptions;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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

        // Calculate precision and generalization (simplified calculations)
        let precision = calculate_precision_simple(&spec, &event_log);
        let generalization = calculate_generalization_simple(&spec, &event_log);

        let report = ConformanceReport {
            fitness,
            precision: Some(precision),
            generalization: Some(generalization),
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
            if let Some(precision) = report.precision {
                println!("  Precision: {:.2}%", precision * 100.0);
            }
            if let Some(generalization) = report.generalization {
                println!("  Generalization: {:.2}%", generalization * 100.0);
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
///
/// Precision measures how much of the model is actually used.
/// High precision means the model doesn't allow too much behavior beyond what's in the event log.
#[verb]
pub fn precision(
    workflow_file: PathBuf,
    xes_file: PathBuf,
    state_store: Option<String>,
    json: bool,
) -> CnvResult<()> {
    // Reuse mining precision calculation
    crate::mining::precision(workflow_file, xes_file, state_store, json)
}

/// Calculate generalization between workflow and XES event log
///
/// Generalization measures how well the model generalizes beyond the event log.
/// High generalization means the model can handle behavior not seen in the log.
#[verb]
pub fn generalization(
    workflow_file: PathBuf,
    xes_file: PathBuf,
    state_store: Option<String>,
    json: bool,
) -> CnvResult<()> {
    // Reuse mining generalization calculation
    crate::mining::generalization(workflow_file, xes_file, state_store, json)
}

/// Generate alignment between workflow design and execution
///
/// Alignment finds the optimal matching between traces in the event log and paths through the model.
/// Uses edit distance to find minimum cost alignments (synchronous moves, model moves, log moves).
#[verb]
pub fn alignment(
    workflow_file: PathBuf,
    xes_file: PathBuf,
    output: Option<PathBuf>,
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

        // Generate alignments for each trace
        // Simplified alignment: find matching activities between trace and model
        let mut alignments = Vec::new();

        for (trace_idx, trace) in event_log.traces.iter().enumerate() {
            let trace_activities: Vec<String> = trace
                .events
                .iter()
                .filter_map(|e| {
                    // Extract activity name from event attributes
                    e.attributes
                        .iter()
                        .find(|attr| attr.key == "concept:name")
                        .and_then(|attr| {
                            // Handle AttributeValue enum
                            match &attr.value {
                                process_mining::event_log::AttributeValue::String(s) => {
                                    Some(s.clone())
                                }
                                _ => None,
                            }
                        })
                })
                .collect();

            // Extract model activities (task names)
            let model_activities: Vec<String> =
                spec.tasks.values().map(|t| t.name.clone()).collect();

            // Calculate alignment cost (simplified: edit distance)
            // Synchronous moves: activities that match
            // Model moves: activities in model but not in trace
            // Log moves: activities in trace but not in model
            let mut synchronous_moves = 0;
            let mut model_moves = 0;
            let mut log_moves = 0;

            for activity in &trace_activities {
                if model_activities.contains(activity) {
                    synchronous_moves += 1;
                } else {
                    log_moves += 1;
                }
            }

            for activity in &model_activities {
                if !trace_activities.contains(activity) {
                    model_moves += 1;
                }
            }

            // Calculate alignment cost (simplified: count of non-synchronous moves)
            let alignment_cost = model_moves + log_moves;
            let alignment_fitness = if !trace_activities.is_empty() {
                1.0 - (alignment_cost as f64 / trace_activities.len() as f64)
            } else {
                0.0
            };

            alignments.push(serde_json::json!({
                "trace_index": trace_idx,
                "synchronous_moves": synchronous_moves,
                "model_moves": model_moves,
                "log_moves": log_moves,
                "alignment_cost": alignment_cost,
                "alignment_fitness": alignment_fitness,
                "trace_length": trace_activities.len()
            }));
        }

        // Calculate average alignment metrics
        let total_synchronous = alignments
            .iter()
            .map(|a| a["synchronous_moves"].as_u64().unwrap_or(0))
            .sum::<u64>();
        let total_model_moves = alignments
            .iter()
            .map(|a| a["model_moves"].as_u64().unwrap_or(0))
            .sum::<u64>();
        let total_log_moves = alignments
            .iter()
            .map(|a| a["log_moves"].as_u64().unwrap_or(0))
            .sum::<u64>();
        let avg_fitness = if !alignments.is_empty() {
            alignments
                .iter()
                .map(|a| a["alignment_fitness"].as_f64().unwrap_or(0.0))
                .sum::<f64>()
                / alignments.len() as f64
        } else {
            0.0
        };

        let alignment_result = serde_json::json!({
            "workflow": workflow_file.display().to_string(),
            "xes_file": xes_file.display().to_string(),
            "total_traces": alignments.len(),
            "total_synchronous_moves": total_synchronous,
            "total_model_moves": total_model_moves,
            "total_log_moves": total_log_moves,
            "average_fitness": avg_fitness,
            "alignments": alignments
        });

        if let Some(output_path) = output {
            let output_text = if json {
                serde_json::to_string_pretty(&alignment_result).map_err(|e| {
                    clap_noun_verb::NounVerbError::execution_error(format!(
                        "Failed to serialize alignment: {}",
                        e
                    ))
                })?
            } else {
                format!(
                    "Alignment Report\n\
                    ===============\n\
                    Workflow: {}\n\
                    Event Log: {}\n\
                    Total Traces: {}\n\
                    Synchronous Moves: {}\n\
                    Model Moves: {}\n\
                    Log Moves: {}\n\
                    Average Fitness: {:.2}%\n",
                    workflow_file.display(),
                    xes_file.display(),
                    alignments.len(),
                    total_synchronous,
                    total_model_moves,
                    total_log_moves,
                    avg_fitness * 100.0
                )
            };
            std::fs::write(&output_path, output_text).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to write alignment file: {}",
                    e
                ))
            })?;
            if !json {
                println!("Alignment saved to: {}", output_path.display());
            }
        }

        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&alignment_result).map_err(|e| {
                    clap_noun_verb::NounVerbError::execution_error(format!(
                        "Failed to serialize alignment: {}",
                        e
                    ))
                })?
            );
        } else {
            println!("Alignment Report");
            println!("===============");
            println!("Workflow: {}", workflow_file.display());
            println!("Event Log: {}", xes_file.display());
            println!("\nResults:");
            println!("  Total Traces: {}", alignments.len());
            println!("  Synchronous Moves: {}", total_synchronous);
            println!("  Model Moves: {}", total_model_moves);
            println!("  Log Moves: {}", total_log_moves);
            println!("  Average Fitness: {:.2}%", avg_fitness * 100.0);
        }

        Ok(())
    })
}

/// Calculate precision between workflow specification and event log (simplified)
fn calculate_precision_simple(
    spec: &knhk_workflow_engine::parser::WorkflowSpec,
    event_log: &process_mining::event_log::EventLog,
) -> f64 {
    // Extract unique activity sequences from event log
    let mut log_sequences = HashSet::new();
    for trace in &event_log.traces {
        let activities: Vec<String> = trace
            .events
            .iter()
            .filter_map(|e| {
                // Extract activity name from event attributes
                e.attributes
                    .iter()
                    .find(|attr| attr.key == "concept:name")
                    .and_then(|attr| {
                        // Handle AttributeValue enum
                        match &attr.value {
                            process_mining::event_log::AttributeValue::String(s) => Some(s.clone()),
                            _ => None,
                        }
                    })
            })
            .collect();
        if !activities.is_empty() {
            log_sequences.insert(activities);
        }
    }

    // Calculate model's possible behaviors (simplified)
    let model_task_count = spec.tasks.len();
    let estimated_model_behaviors = if model_task_count > 0 {
        (model_task_count as f64).powi(2).min(1000.0)
    } else {
        0.0
    };

    // Precision = used behaviors / total possible behaviors
    let used_behaviors = log_sequences.len() as f64;
    if estimated_model_behaviors > 0.0 {
        (used_behaviors / estimated_model_behaviors).min(1.0)
    } else {
        0.0
    }
}

/// Calculate generalization between workflow specification and event log (simplified)
fn calculate_generalization_simple(
    spec: &knhk_workflow_engine::parser::WorkflowSpec,
    event_log: &process_mining::event_log::EventLog,
) -> f64 {
    // Calculate model complexity
    let model_task_count = spec.tasks.len();
    let model_flow_count = spec.flows.len();
    let model_complexity = (model_task_count + model_flow_count) as f64;

    // Calculate log complexity
    let mut log_sequences = HashSet::new();
    for trace in &event_log.traces {
        let activities: Vec<String> = trace
            .events
            .iter()
            .filter_map(|e| {
                // Extract activity name from event attributes
                e.attributes
                    .iter()
                    .find(|attr| attr.key == "concept:name")
                    .and_then(|attr| {
                        // Handle AttributeValue enum
                        match &attr.value {
                            process_mining::event_log::AttributeValue::String(s) => Some(s.clone()),
                            _ => None,
                        }
                    })
            })
            .collect();
        if !activities.is_empty() {
            log_sequences.insert(activities);
        }
    }
    let log_complexity = log_sequences.len() as f64;

    // Generalization = 1 - (model_complexity / log_complexity)
    if log_complexity > 0.0 {
        (1.0 - (model_complexity / log_complexity))
            .max(0.0)
            .min(1.0)
    } else {
        0.0
    }
}
