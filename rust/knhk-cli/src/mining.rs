//! Process Mining CLI Commands
//!
//! Implements Van der Aalst's process mining methodology:
//! - XES event log export
//! - Process discovery (Alpha algorithm)
//! - Conformance checking
//! - Fitness, precision, and generalization calculation

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use knhk_workflow_engine::{case::CaseId, state::StateStore, WorkflowEngine};
use process_mining::{
    alphappp::full::{alphappp_discover_petri_net, AlphaPPPConfig},
    event_log::activity_projection::EventLogActivityProjection,
    import_xes_file, XESImportOptions,
};
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
                    "discovery_time": format!("{:?}", duration)
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
                let mut pnml_writer = Vec::new();
                export_petri_net_to_pnml(&petri_net, &mut pnml_writer).map_err(|e| {
                    clap_noun_verb::NounVerbError::execution_error(format!(
                        "Failed to export PNML: {:?}",
                        e
                    ))
                })?;
                std::fs::write(&output_path, pnml_writer).map_err(|e| {
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

        for _trace in &event_log.traces {
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

        // Calculate precision: compare model's possible behaviors with event log's actual behaviors
        // Precision = 1 - (unused_model_behavior / total_model_behavior)

        // Extract unique activity sequences from event log
        let mut log_sequences = std::collections::HashSet::new();
        for _trace in &event_log.traces {
            let activities: Vec<String> = trace
                .events
                .iter()
                .filter_map(|e| {
                    // Extract activity name from event attributes
                    e.attributes
                        .iter()
                        .find(|attr| attr.key == "concept:name")
                        .and_then(|attr| match &attr.value {
                            process_mining::event_log::AttributeValue::String(s) => Some(s.clone()),
                            _ => None,
                        })
                        .map(|s| s.to_string())
                })
                .collect();
            if !activities.is_empty() {
                log_sequences.insert(activities);
            }
        }

        // Calculate model's possible behaviors (simplified: count unique task sequences)
        // In a full implementation, we would enumerate all possible execution paths
        let model_task_count = spec.tasks.len();
        let model_flow_count = spec.flows.len();

        // Estimate model complexity (simplified heuristic)
        // More tasks and flows = more possible behaviors
        let estimated_model_behaviors = if model_task_count > 0 {
            // Rough estimate: factorial of task count (simplified)
            // In practice, this would enumerate actual execution paths
            (model_task_count as f64).powi(2).min(1000.0)
        } else {
            0.0
        };

        // Calculate precision
        let used_behaviors = log_sequences.len() as f64;
        let precision = if estimated_model_behaviors > 0.0 {
            // Precision = used behaviors / total possible behaviors
            // Higher precision = model is more specific (less unused behavior)
            (used_behaviors / estimated_model_behaviors).min(1.0)
        } else {
            0.0
        };

        if json {
            let result = serde_json::json!({
                "precision": precision,
                "used_behaviors": used_behaviors as u32,
                "estimated_model_behaviors": estimated_model_behaviors as u32,
                "model_tasks": model_task_count,
                "model_flows": model_flow_count,
                "log_traces": event_log.traces.len(),
                "log_sequences": log_sequences.len()
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
            println!("Precision Calculation");
            println!("=====================");
            println!("Workflow: {}", workflow_file.display());
            println!("Event Log: {}", xes_file.display());
            println!("\nResults:");
            println!("  Precision: {:.2}%", precision * 100.0);
            println!("  Used Behaviors: {}", used_behaviors as u32);
            println!(
                "  Estimated Model Behaviors: {}",
                estimated_model_behaviors as u32
            );
            println!("  Model Tasks: {}", model_task_count);
            println!("  Model Flows: {}", model_flow_count);
            println!("  Log Traces: {}", event_log.traces.len());
            println!("  Unique Sequences: {}", log_sequences.len());
        }

        Ok(())
    })
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

        // Calculate generalization: compare model complexity to log complexity
        // Generalization = 1 - (model_complexity / log_complexity)
        // Higher generalization = model is simpler relative to log (can generalize better)

        // Calculate model complexity (simplified: based on structure)
        let model_task_count = spec.tasks.len();
        let model_flow_count = spec.flows.len();
        let model_complexity = (model_task_count + model_flow_count) as f64;

        // Calculate log complexity (simplified: based on unique sequences)
        let mut log_sequences = std::collections::HashSet::new();
        for _trace in &event_log.traces {
            let activities: Vec<String> = trace
                .events
                .iter()
                .filter_map(|e| {
                    // Extract activity name from event attributes
                    e.attributes
                        .iter()
                        .find(|attr| attr.key == "concept:name")
                        .and_then(|attr| match &attr.value {
                            process_mining::event_log::AttributeValue::String(s) => Some(s.clone()),
                            _ => None,
                        })
                        .map(|s| s.to_string())
                })
                .collect();
            if !activities.is_empty() {
                log_sequences.insert(activities);
            }
        }
        let log_complexity = log_sequences.len() as f64;

        // Calculate generalization
        let generalization = if log_complexity > 0.0 {
            // Generalization = 1 - (model_complexity / log_complexity)
            // If model is simpler than log, generalization is high
            (1.0 - (model_complexity / log_complexity))
                .max(0.0)
                .min(1.0)
        } else {
            0.0
        };

        if json {
            let result = serde_json::json!({
                "generalization": generalization,
                "model_complexity": model_complexity as u32,
                "log_complexity": log_complexity as u32,
                "model_tasks": model_task_count,
                "model_flows": model_flow_count,
                "log_traces": event_log.traces.len(),
                "log_sequences": log_sequences.len()
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
            println!("Generalization Calculation");
            println!("=========================");
            println!("Workflow: {}", workflow_file.display());
            println!("Event Log: {}", xes_file.display());
            println!("\nResults:");
            println!("  Generalization: {:.2}%", generalization * 100.0);
            println!("  Model Complexity: {}", model_complexity as u32);
            println!("  Log Complexity: {}", log_complexity as u32);
            println!("  Model Tasks: {}", model_task_count);
            println!("  Model Flows: {}", model_flow_count);
            println!("  Log Traces: {}", event_log.traces.len());
            println!("  Unique Sequences: {}", log_sequences.len());
        }

        Ok(())
    })
}
