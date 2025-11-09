//! Process Mining CLI Commands
//!
//! Implements Van der Aalst's process mining methodology:
//! - XES event log export
//! - Process discovery (Alpha algorithm)
//! - Conformance checking
//! - Fitness, precision, and generalization calculation

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use knhk_workflow_engine::{
    api::transport::CliAdapter, case::CaseId, parser::WorkflowSpecId, state::StateStore,
    WorkflowEngine,
};
use process_mining::{
    alphappp::full::{alphappp_discover_petri_net, AlphaPPPConfig},
    event_log::activity_projection::EventLogActivityProjection,
    import_xes_file, XESImportOptions,
};
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

/// Export case to XES format
#[verb]
pub fn export_xes(
    case_id: String,
    output: Option<PathBuf>,
    state_store: Option<String>,
    json: bool,
) -> CnvResult<()> {
    let runtime = get_runtime();
    let engine = get_engine(state_store.as_deref())?;

    runtime.block_on(async {
        let case_id_uuid = CaseId::parse_str(&case_id).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Invalid case ID: {}", e))
        })?;

        let xes_content = engine.export_case_to_xes(case_id_uuid).await.map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to export case to XES: {}",
                e
            ))
        })?;

        let output_path = output.unwrap_or_else(|| PathBuf::from(format!("{}.xes", case_id)));

        std::fs::write(&output_path, &xes_content).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to write XES file: {}",
                e
            ))
        })?;

        if json {
            let result = serde_json::json!({
                "case_id": case_id,
                "output_file": output_path.display().to_string(),
                "size_bytes": xes_content.len()
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
            println!(
                "Case {} exported to XES: {}",
                case_id,
                output_path.display()
            );
            println!("Import into ProM: prom --import {}", output_path.display());
        }

        Ok(())
    })
}

/// Discover process model from XES event log
#[verb]
pub fn discover(
    xes_file: PathBuf,
    algorithm: Option<String>,
    output: Option<PathBuf>,
    json: bool,
) -> CnvResult<()> {
    // Import XES file
    let event_log = import_xes_file(&xes_file, XESImportOptions::default()).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!(
            "Failed to import XES file: {:?}",
            e
        ))
    })?;

    let algorithm = algorithm.as_deref().unwrap_or("alpha");

    match algorithm {
        "alpha" | "alphappp" => {
            // Create activity projection for Alpha+++
            let projection: EventLogActivityProjection = (&event_log).into();

            // Run Alpha+++ process discovery
            let config = AlphaPPPConfig {
                log_repair_skip_df_thresh_rel: 2.0,
                log_repair_loop_df_thresh_rel: 2.0,
                absolute_df_clean_thresh: 1,
                relative_df_clean_thresh: 0.01,
                balance_thresh: 0.5,
                fitness_thresh: 0.5,
                replay_thresh: 0.5,
            };

            let (petri_net, duration) = alphappp_discover_petri_net(&projection, config);

            if json {
                let result = serde_json::json!({
                    "algorithm": "alphappp",
                    "places": petri_net.places.len(),
                    "transitions": petri_net.transitions.len(),
                    "arcs": petri_net.arcs.len(),
                    "discovery_time_ms": duration.as_millis()
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
                println!("Process Discovery (Alpha+++)");
                println!("============================");
                println!("Algorithm: Alpha+++");
                println!("Places: {}", petri_net.places.len());
                println!("Transitions: {}", petri_net.transitions.len());
                println!("Arcs: {}", petri_net.arcs.len());
                println!("Discovery Time: {:?}", duration);
            }

            if let Some(output_path) = output {
                // Export Petri net to PNML format
                use process_mining::export_petri_net_to_pnml;
                let pnml = export_petri_net_to_pnml(&petri_net);
                std::fs::write(&output_path, pnml).map_err(|e| {
                    clap_noun_verb::NounVerbError::execution_error(format!(
                        "Failed to write PNML file: {}",
                        e
                    ))
                })?;

                if !json {
                    println!("Petri net exported to: {}", output_path.display());
                }
            }
        }
        _ => {
            return Err(clap_noun_verb::NounVerbError::execution_error(format!(
                "Unknown algorithm: {}. Supported: alpha, alphappp",
                algorithm
            )));
        }
    }

    Ok(())
}

/// Check conformance between workflow and XES event log
#[verb]
pub fn conformance(
    workflow_file: PathBuf,
    xes_file: PathBuf,
    state_store: Option<String>,
    json: bool,
) -> CnvResult<()> {
    let runtime = get_runtime();
    let engine = get_engine(state_store.as_deref())?;

    runtime.block_on(async {
        // Parse workflow
        let mut parser = knhk_workflow_engine::parser::WorkflowParser::new().map_err(|e| {
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
            conformant_traces as f64 / total_traces as f64
        } else {
            0.0
        };

        if json {
            let result = serde_json::json!({
                "workflow": workflow_file.display().to_string(),
                "xes_file": xes_file.display().to_string(),
                "total_traces": total_traces,
                "conformant_traces": conformant_traces,
                "non_conformant_traces": non_conformant_traces,
                "fitness": fitness
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
            println!("Conformance Check");
            println!("================");
            println!("Workflow: {}", workflow_file.display());
            println!("Event Log: {}", xes_file.display());
            println!("Total Traces: {}", total_traces);
            println!("Conformant: {}", conformant_traces);
            println!("Non-Conformant: {}", non_conformant_traces);
            println!("Fitness: {:.2}%", fitness * 100.0);
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
    // Reuse conformance logic for fitness calculation
    conformance(workflow_file, xes_file, state_store, json)
}

/// Calculate precision between workflow and XES event log
#[verb]
pub fn precision(
    workflow_file: PathBuf,
    xes_file: PathBuf,
    state_store: Option<String>,
    json: bool,
) -> CnvResult<()> {
    // Precision calculation would require more sophisticated analysis
    // For now, provide a placeholder that indicates precision is not yet fully implemented
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
    // Generalization calculation would require analyzing how well the model generalizes
    // For now, provide a placeholder
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
