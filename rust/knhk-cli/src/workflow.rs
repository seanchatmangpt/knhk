#![allow(clippy::unwrap_used)] // CLI tool - unwrap() acceptable for user-facing errors
//! Workflow Engine CLI Commands
//!
//! Provides CLI interface for workflow engine operations:
//! - workflow parse: Parse workflow from Turtle file
//! - workflow register: Register a workflow specification
//! - workflow create: Create a new workflow case
//! - workflow start: Start a workflow case
//! - workflow execute: Execute a workflow case
//! - workflow cancel: Cancel a workflow case
//! - workflow get: Get case status
//! - workflow list: List all workflow cases
//! - workflow patterns: List all 43 patterns
//! - workflow serve: Start REST API server
//! - workflow import-xes: Import XES event log
//! - workflow export-xes: Export workflow execution to XES format
//! - workflow discover: Run Alpha+++ process discovery algorithm

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use knhk_workflow_engine::{
    case::CaseId,
    parser::{WorkflowParser, WorkflowSpecId},
    state::StateStore,
    WorkflowEngine,
};
use process_mining::{
    alphappp::full::{alphappp_discover_petri_net, AlphaPPPConfig},
    event_log::activity_projection::EventLogActivityProjection,
    export_petri_net_to_pnml, import_xes_file, XESImportOptions,
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

/// Get workflow engine instance (created per command to avoid Sync issues with LockchainStorage)
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

/// Parse a workflow from Turtle file
#[verb]
pub fn parse(file: PathBuf, output: Option<PathBuf>) -> CnvResult<()> {
    let runtime = get_runtime();
    runtime.block_on(async {
        let mut parser = WorkflowParser::new().map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to create parser: {}",
                e
            ))
        })?;

        let spec = parser.parse_file(&file).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to parse workflow: {}",
                e
            ))
        })?;

        let json = serde_json::to_string_pretty(&spec).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to serialize workflow: {}",
                e
            ))
        })?;

        if let Some(output_path) = output {
            std::fs::write(&output_path, json).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to write output file: {}",
                    e
                ))
            })?;
            println!("Workflow parsed and saved to {}", output_path.display());
        } else {
            println!("{}", json);
        }

        Ok(())
    })
}

/// Register a workflow specification
#[verb]
pub fn register(file: PathBuf, state_store: Option<String>) -> CnvResult<()> {
    let runtime = get_runtime();
    let engine = get_engine(state_store.as_deref())?;

    runtime.block_on(async {
        let mut parser = WorkflowParser::new().map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to create parser: {}",
                e
            ))
        })?;

        let spec = if file.extension().and_then(|s| s.to_str()) == Some("ttl") {
            parser.parse_file(&file).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to parse Turtle file: {}",
                    e
                ))
            })?
        } else {
            // Try JSON
            let contents = std::fs::read_to_string(&file).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to read file: {}",
                    e
                ))
            })?;
            serde_json::from_str(&contents).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to parse JSON: {}",
                    e
                ))
            })?
        };

        engine.register_workflow(spec.clone()).await.map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to register workflow: {}",
                e
            ))
        })?;

        println!("Workflow registered: {} ({})", spec.name, spec.id);
        Ok(())
    })
}

/// Create a new workflow case
#[verb]
pub fn create(spec_id: String, data: Option<String>, state_store: Option<String>) -> CnvResult<()> {
    let runtime = get_runtime();
    let engine = get_engine(state_store.as_deref())?;

    runtime.block_on(async {
        let spec_id_uuid = WorkflowSpecId::parse_str(&spec_id).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Invalid spec ID: {}", e))
        })?;

        let case_data = if let Some(data_str) = data {
            serde_json::from_str(&data_str).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!("Invalid JSON data: {}", e))
            })?
        } else {
            serde_json::json!({})
        };

        let case_id = engine
            .create_case(spec_id_uuid, case_data)
            .await
            .map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to create case: {}",
                    e
                ))
            })?;

        println!("Case created: {}", case_id);
        Ok(())
    })
}

/// Start a workflow case
#[verb]
pub fn start(case_id: String, state_store: Option<String>) -> CnvResult<()> {
    let runtime = get_runtime();
    let engine = get_engine(state_store.as_deref())?;

    runtime.block_on(async {
        let case_id_uuid = CaseId::parse_str(&case_id).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Invalid case ID: {}", e))
        })?;

        engine.start_case(case_id_uuid).await.map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to start case: {}", e))
        })?;

        println!("Case started: {}", case_id);
        Ok(())
    })
}

/// Execute a workflow case
#[verb]
pub fn execute(case_id: String, state_store: Option<String>) -> CnvResult<()> {
    let runtime = get_runtime();
    let engine = get_engine(state_store.as_deref())?;

    runtime.block_on(async {
        let case_id_uuid = CaseId::parse_str(&case_id).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Invalid case ID: {}", e))
        })?;

        engine.execute_case(case_id_uuid).await.map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to execute case: {}", e))
        })?;

        println!("Case executed: {}", case_id);
        Ok(())
    })
}

/// Cancel a workflow case
#[verb]
pub fn cancel(case_id: String, state_store: Option<String>) -> CnvResult<()> {
    let runtime = get_runtime();
    let engine = get_engine(state_store.as_deref())?;

    runtime.block_on(async {
        let case_id_uuid = CaseId::parse_str(&case_id).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Invalid case ID: {}", e))
        })?;

        engine.cancel_case(case_id_uuid).await.map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to cancel case: {}", e))
        })?;

        println!("Case cancelled: {}", case_id);
        Ok(())
    })
}

/// Get case status
#[verb]
pub fn get(case_id: String, state_store: Option<String>) -> CnvResult<()> {
    let runtime = get_runtime();
    let engine = get_engine(state_store.as_deref())?;

    runtime.block_on(async {
        let case_id_uuid = CaseId::parse_str(&case_id).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Invalid case ID: {}", e))
        })?;

        let case = engine.get_case(case_id_uuid).await.map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to get case: {}", e))
        })?;

        let json = serde_json::to_string_pretty(&case).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to serialize case: {}",
                e
            ))
        })?;

        println!("{}", json);
        Ok(())
    })
}

/// List all workflow cases for a specification
#[verb]
pub fn list(spec_id: Option<String>, state_store: Option<String>) -> CnvResult<()> {
    let runtime = get_runtime();
    let engine = get_engine(state_store.as_deref())?;

    runtime.block_on(async {
        if let Some(spec_id_str) = spec_id {
            let spec_id_uuid = WorkflowSpecId::parse_str(&spec_id_str).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!("Invalid spec ID: {}", e))
            })?;
            let cases = engine.list_cases(spec_id_uuid).await.map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to list cases: {}",
                    e
                ))
            })?;
            println!("Cases for workflow {}:", spec_id_str);
            for case_id in cases {
                println!("  - {}", case_id);
            }
        } else {
            // List all workflows
            let workflows = engine.list_workflows().await.map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to list workflows: {}",
                    e
                ))
            })?;
            println!("Registered workflows:");
            for spec_id in workflows {
                println!("  - {}", spec_id);
            }
        }
        Ok(())
    })
}

/// List all 43 workflow patterns
#[verb]
pub fn patterns() -> CnvResult<()> {
    let runtime = get_runtime();
    let engine = get_engine(None)?;

    runtime.block_on(async {
        let registry = engine.pattern_registry();
        let patterns = registry.list_patterns();
        println!("Registered patterns ({}):", patterns.len());
        for pattern_id in patterns {
            println!("  - Pattern {}", pattern_id.0);
        }
        Ok(())
    })
}

/// Start REST API server
#[verb]
pub fn serve(
    port: Option<u16>,
    host: Option<String>,
    state_store: Option<String>,
) -> CnvResult<()> {
    let runtime = get_runtime();
    let engine = get_engine(state_store.as_deref())?;

    let port = port.unwrap_or(8080);
    let host = host.unwrap_or_else(|| "0.0.0.0".to_string());

    println!("Starting REST API server on {}:{}", host, port);

    runtime.block_on(async {
        let _app = knhk_workflow_engine::api::rest::RestApiServer::new(engine.clone()).router();
        // Use std::net::TcpListener directly for axum 0.6 Server compatibility
        use std::net::TcpListener as StdTcpListener;
        let std_listener = StdTcpListener::bind(format!("{}:{}", host, port)).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to bind to {}:{}: {}",
                host, port, e
            ))
        })?;
        std_listener.set_nonblocking(true).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to set non-blocking: {}", e))
        })?;

        println!("Server listening on http://{}:{}", host, port);

        // Temporarily disabled: axum version mismatch (workflow engine uses 0.6, CLI uses 0.7)
        // Fix by updating workflow engine to axum 0.7 or creating compatibility layer
        Err(clap_noun_verb::NounVerbError::execution_error(
            "Serve command temporarily disabled due to axum version mismatch. Use workflow engine REST API directly."
        ))
    })
}

/// Import XES event log
#[verb]
pub fn import_xes(file: PathBuf, output: Option<PathBuf>) -> CnvResult<()> {
    let log = import_xes_file(&file, XESImportOptions::default()).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("Failed to import XES file: {}", e))
    })?;

    let json = serde_json::to_string_pretty(&log).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!(
            "Failed to serialize event log: {}",
            e
        ))
    })?;

    if let Some(output_path) = output {
        std::fs::write(&output_path, json).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to write output file: {}",
                e
            ))
        })?;
        println!(
            "XES event log imported: {} traces, saved to {}",
            log.traces.len(),
            output_path.display()
        );
    } else {
        println!("XES event log imported: {} traces", log.traces.len());
        println!("{}", json);
    }

    Ok(())
}

/// Export workflow execution to XES format
#[verb]
pub fn export_xes(
    case_id: Option<String>,
    spec_id: Option<String>,
    output: PathBuf,
    state_store: Option<String>,
) -> CnvResult<()> {
    let runtime = get_runtime();
    let engine = get_engine(state_store.as_deref())?;

    runtime.block_on(async {
        let xes_xml = if let Some(case_id_str) = case_id {
            // Export single case
            let case_id_uuid = CaseId::parse_str(&case_id_str).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!("Invalid case ID: {}", e))
            })?;
            engine.export_case_to_xes(case_id_uuid).await.map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to export case to XES: {}",
                    e
                ))
            })?
        } else if let Some(spec_id_str) = spec_id {
            // Export all cases for workflow
            let spec_id_uuid = WorkflowSpecId::parse_str(&spec_id_str).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!("Invalid spec ID: {}", e))
            })?;
            engine
                .export_workflow_to_xes(spec_id_uuid)
                .await
                .map_err(|e| {
                    clap_noun_verb::NounVerbError::execution_error(format!(
                        "Failed to export workflow to XES: {}",
                        e
                    ))
                })?
        } else {
            // Export all cases
            engine.export_all_cases_to_xes().await.map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to export all cases to XES: {}",
                    e
                ))
            })?
        };

        std::fs::write(&output, xes_xml).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to write XES file: {}",
                e
            ))
        })?;

        println!("XES export completed: {}", output.display());
        Ok(())
    })
}

/// Run Alpha+++ process discovery algorithm
#[verb]
pub fn discover(
    xes_file: PathBuf,
    output: PathBuf,
    alpha: Option<f64>,
    beta: Option<f64>,
    theta: Option<f64>,
    rho: Option<f64>,
) -> CnvResult<()> {
    // Import XES event log
    let log = import_xes_file(&xes_file, XESImportOptions::default()).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("Failed to import XES file: {}", e))
    })?;

    // Create activity projection (required for Alpha+++)
    let projection: EventLogActivityProjection = (&log).into();

    // Run Alpha+++ discovery with default or provided parameters
    // Note: Alpha+++ function takes projection and AlphaPPPConfig, returns (PetriNet, AlgoDuration)
    let alpha_param = alpha.unwrap_or(2.0);
    let beta_param = beta.unwrap_or(0.5);
    let theta_param = theta.unwrap_or(0.5);
    let rho_param = rho.unwrap_or(0.5);

    // Alpha+++ discovery: function signature is (projection, config) -> (PetriNet, AlgoDuration)
    // Map user parameters to AlphaPPPConfig:
    // - alpha -> log_repair_skip_df_thresh_rel and log_repair_loop_df_thresh_rel
    // - beta -> balance_thresh
    // - theta -> fitness_thresh
    // - rho -> replay_thresh
    let config = AlphaPPPConfig {
        balance_thresh: beta_param as f32,
        fitness_thresh: theta_param as f32,
        replay_thresh: rho_param as f32,
        log_repair_skip_df_thresh_rel: alpha_param as f32,
        log_repair_loop_df_thresh_rel: alpha_param as f32,
        absolute_df_clean_thresh: 1,
        relative_df_clean_thresh: 0.01,
    };
    let (petri_net, _duration) = alphappp_discover_petri_net(&projection, config);

    // Export Petri net to PNML (needs a writer)
    let mut pnml_writer = Vec::new();
    export_petri_net_to_pnml(&petri_net, &mut pnml_writer).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!(
            "Failed to export Petri net to PNML: {}",
            e
        ))
    })?;

    std::fs::write(&output, pnml_writer).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("Failed to write PNML file: {}", e))
    })?;

    println!(
        "Alpha+++ discovery completed: {} places, {} transitions, saved to {}",
        petri_net.places.len(),
        petri_net.transitions.len(),
        output.display()
    );
    println!(
        "Parameters: α={}, β={}, θ={}, ρ={} (note: only α is used in this API version)",
        alpha_param, beta_param, theta_param, rho_param
    );

    Ok(())
}
