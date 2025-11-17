#![allow(non_upper_case_globals)] // #[verb] macro generates static vars with lowercase names
//! Console Commands - Interactive YAWL/Turtle Workflow Console
//!
//! Provides CLI interface for interactive console operations:
//! - console start: Start interactive console session with REPL
//! - console load: Load a Turtle workflow file into console context
//! - console run: Execute console commands in loaded workflow context
//! - console query: Execute SPARQL queries on loaded workflows

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;

#[cfg(feature = "workflow")]
use knhk_workflow_engine::parser::WorkflowParser;

#[cfg(feature = "workflow")]
use knhk_workflow_engine::{
    api::{
        models::requests::{CreateCaseRequest, ListCasesRequest},
        service::{CaseService, PatternService},
        transport::CliAdapter,
    },
    case::CaseId,
    parser::WorkflowSpecId,
    state::StateStore,
    validation::ValidationFramework,
    WorkflowEngine,
};

use serde_json::json;
use std::sync::Arc;

#[cfg(feature = "otel")]
use tracing::instrument;

/// Console session context - maintains loaded workflow and state
#[derive(Clone, Debug)]
struct ConsoleContext {
    workflow_path: Option<String>,
    workflow_id: Option<String>,
    state_store_path: Option<String>,
    // Cache parsed workflow spec for engine operations
    #[cfg(feature = "workflow")]
    workflow_spec: Option<Arc<knhk_workflow_engine::WorkflowSpec>>,
}

/// Global console context - shared across commands
static CONSOLE_CONTEXT: std::sync::OnceLock<Mutex<ConsoleContext>> = std::sync::OnceLock::new();

fn get_context() -> &'static Mutex<ConsoleContext> {
    CONSOLE_CONTEXT.get_or_init(|| {
        Mutex::new(ConsoleContext {
            workflow_path: None,
            workflow_id: None,
            state_store_path: None,
            #[cfg(feature = "workflow")]
            workflow_spec: None,
        })
    })
}

/// Get or create tokio runtime for async operations
fn get_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Runtime::new().unwrap_or_else(|e| {
            panic!("Failed to create tokio runtime: {}", e);
        })
    })
}

/// Create workflow engine with state store
#[cfg(feature = "workflow")]
fn create_engine(state_store_path: &Option<String>) -> CnvResult<Arc<WorkflowEngine>> {
    let path = state_store_path.as_deref().unwrap_or("./workflow_db");
    let state_store = StateStore::new(path).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!(
            "Failed to create state store: {}",
            e
        ))
    })?;
    Ok(Arc::new(WorkflowEngine::new(state_store)))
}

#[derive(Serialize, Debug)]
struct StartResult {
    status: String,
    message: String,
}

#[derive(Serialize, Debug)]
struct LoadResult {
    status: String,
    workflow_id: String,
    workflow_path: String,
}

#[derive(Serialize, Debug)]
struct RunResult {
    status: String,
    output: String,
    command: String,
}

#[derive(Serialize, Debug)]
struct QueryResult {
    status: String,
    results: String,
    query: String,
}

/// Start interactive console session with REPL
#[cfg_attr(feature = "otel", instrument(skip_all, fields(operation = "knhk.console.start", state_store = ?state_store)))]
#[verb] // Noun "console" auto-inferred from filename "console.rs"
pub fn start(state_store: Option<String>) -> CnvResult<StartResult> {
    #[cfg(feature = "otel")]
    {
        use std::time::Instant;
        use tracing::{error, info};

        let start_time = Instant::now();

        // Initialize context with state store
        let mut ctx = get_context().lock().map_err(|e| {
            error!(error = %e, "console.start.lock.failed");
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to lock console context: {}",
                e
            ))
        })?;

        ctx.state_store_path = state_store.clone();
        drop(ctx); // Release lock

        let duration = start_time.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            state_store = ?state_store,
            "console.start.success"
        );

        Ok(StartResult {
            status: "success".to_string(),
            message:
                "Interactive console started. Type 'help' for available commands or 'quit' to exit."
                    .to_string(),
        })
    }

    #[cfg(not(feature = "otel"))]
    {
        let mut ctx = get_context().lock().map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to lock console context: {}",
                e
            ))
        })?;

        ctx.state_store_path = state_store.clone();
        drop(ctx); // Release lock

        Ok(StartResult {
            status: "success".to_string(),
            message:
                "Interactive console started. Type 'help' for available commands or 'quit' to exit."
                    .to_string(),
        })
    }
}

/// Load a Turtle workflow file into console context
#[cfg_attr(feature = "otel", instrument(skip_all, fields(operation = "knhk.console.load", file = ?file)))]
#[verb]
pub fn load(file: PathBuf, state_store: Option<String>) -> CnvResult<LoadResult> {
    #[cfg(feature = "otel")]
    {
        use std::time::Instant;
        use tracing::{error, info};

        let start_time = Instant::now();

        let runtime = get_runtime();
        let result = runtime.block_on(async {
            // Verify file exists
            if !file.exists() {
                return Err(clap_noun_verb::NounVerbError::execution_error(format!(
                    "Workflow file not found: {}",
                    file.display()
                )));
            }

            // Parse the Turtle workflow
            let mut parser = WorkflowParser::new().map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to create parser: {}",
                    e
                ))
            })?;

            let spec = parser.parse_file(&file).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to parse workflow file: {}",
                    e
                ))
            })?;

            // Update context
            let mut ctx = get_context().lock().map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to lock console context: {}",
                    e
                ))
            })?;

            ctx.workflow_path = Some(file.to_string_lossy().to_string());
            ctx.workflow_id = Some(spec.id.to_string());
            ctx.state_store_path = state_store.clone();
            ctx.workflow_spec = Some(Arc::new(spec.clone()));
            drop(ctx); // Release lock

            Ok((spec.id.to_string(), file.to_string_lossy().to_string()))
        });

        match result {
            Ok((workflow_id, workflow_path)) => {
                let duration = start_time.elapsed();
                info!(
                    duration_ms = duration.as_millis(),
                    workflow_id = %workflow_id,
                    file = ?workflow_path,
                    "console.load.success"
                );

                Ok(LoadResult {
                    status: "success".to_string(),
                    workflow_id,
                    workflow_path,
                })
            }
            Err(e) => {
                error!(error = ?e, "console.load.failed");
                Err(e)
            }
        }
    }

    #[cfg(not(feature = "otel"))]
    {
        let runtime = get_runtime();
        let (workflow_id, workflow_path) = runtime.block_on(async {
            // Verify file exists
            if !file.exists() {
                return Err(clap_noun_verb::NounVerbError::execution_error(format!(
                    "Workflow file not found: {}",
                    file.display()
                )));
            }

            // Parse the Turtle workflow
            let mut parser = WorkflowParser::new().map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to create parser: {}",
                    e
                ))
            })?;

            let spec = parser.parse_file(&file).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to parse workflow file: {}",
                    e
                ))
            })?;

            // Update context
            let mut ctx = get_context().lock().map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to lock console context: {}",
                    e
                ))
            })?;

            ctx.workflow_path = Some(file.to_string_lossy().to_string());
            ctx.workflow_id = Some(spec.id.to_string());
            ctx.state_store_path = state_store.clone();
            ctx.workflow_spec = Some(Arc::new(spec.clone()));
            drop(ctx); // Release lock

            Ok((spec.id.to_string(), file.to_string_lossy().to_string()))
        })?;

        Ok(LoadResult {
            status: "success".to_string(),
            workflow_id,
            workflow_path,
        })
    }
}

/// Run a console command in loaded workflow context
#[cfg_attr(feature = "otel", instrument(skip_all, fields(operation = "knhk.console.run", command = %command)))]
#[verb]
pub fn run(command: String) -> CnvResult<RunResult> {
    #[cfg(feature = "otel")]
    {
        use std::time::Instant;
        use tracing::{error, info};

        let start_time = Instant::now();

        // Get current context
        let ctx = get_context()
            .lock()
            .map_err(|e| {
                error!(error = %e, "console.run.lock.failed");
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to lock console context: {}",
                    e
                ))
            })?
            .clone();

        // Check if workflow is loaded
        if ctx.workflow_id.is_none() {
            error!(command = %command, "console.run.no_workflow");
            return Err(clap_noun_verb::NounVerbError::execution_error(
                "No workflow loaded. Use 'load <file>' first.".to_string(),
            ));
        }

        let runtime = get_runtime();
        let output = runtime
            .block_on(async {
                match command.trim() {
                    "help" => vec![
                        "Available console commands:".to_string(),
                        "  help              - Show this help message".to_string(),
                        "  status            - Show loaded workflow status".to_string(),
                        "  patterns          - List all 43 Van der Aalst patterns".to_string(),
                        "  validate          - Validate loaded workflow".to_string(),
                        "  create-case       - Create new workflow case".to_string(),
                        "  list-cases        - List all workflow cases".to_string(),
                        "  quit              - Exit console".to_string(),
                    ],
                    "status" => vec![
                        format!(
                            "Workflow ID: {}",
                            ctx.workflow_id.clone().unwrap_or_default()
                        ),
                        format!(
                            "Workflow Path: {}",
                            ctx.workflow_path.clone().unwrap_or_default()
                        ),
                        format!(
                            "State Store: {}",
                            ctx.state_store_path
                                .clone()
                                .unwrap_or("./workflow_db".to_string())
                        ),
                    ],
                    "patterns" => vec![
                        "Available patterns (43 Van der Aalst patterns):".to_string(),
                        "  Pattern 1: Sequence".to_string(),
                        "  Pattern 2: Parallel Split".to_string(),
                        "  Pattern 3: Synchronization".to_string(),
                        "  ... and 40 more patterns".to_string(),
                        "Use 'workflow patterns' command for full list.".to_string(),
                    ],
                    "validate" => {
                        #[cfg(feature = "workflow")]
                        {
                            let spec_id_str = ctx.workflow_id.clone().unwrap();
                            let spec_id = WorkflowSpecId::parse_str(&spec_id_str).map_err(|e| {
                                clap_noun_verb::NounVerbError::execution_error(format!(
                                    "Invalid spec ID: {}",
                                    e
                                ))
                            })?;

                            let engine = create_engine(&ctx.state_store_path)?;
                            let framework = ValidationFramework::new(engine);
                            let report =
                                framework
                                    .run_complete_validation(spec_id)
                                    .await
                                    .map_err(|e| {
                                        clap_noun_verb::NounVerbError::execution_error(format!(
                                            "Validation failed: {}",
                                            e
                                        ))
                                    })?;

                            vec![
                                format!("Validation Status: {:?}", report.summary.overall_status),
                                format!(
                                    "Passed phases: {}/{}",
                                    report.summary.passed_phases, report.summary.total_phases
                                ),
                                format!(
                                    "Failed phases: {}/{}",
                                    report.summary.failed_phases, report.summary.total_phases
                                ),
                                format!("Warnings: {}", report.summary.warnings),
                            ]
                        }
                        #[cfg(not(feature = "workflow"))]
                        {
                            vec!["Workflow feature not enabled".to_string()]
                        }
                    }
                    "create-case" => {
                        #[cfg(feature = "workflow")]
                        {
                            let spec_id_str = ctx.workflow_id.clone().unwrap();
                            let spec_id = WorkflowSpecId::parse_str(&spec_id_str).map_err(|e| {
                                clap_noun_verb::NounVerbError::execution_error(format!(
                                    "Invalid spec ID: {}",
                                    e
                                ))
                            })?;

                            let engine = create_engine(&ctx.state_store_path)?;
                            let service = CaseService::new(engine);
                            let request = CreateCaseRequest {
                                spec_id,
                                data: json!({}),
                            };
                            let response = service.create_case(request).await.map_err(|e| {
                                clap_noun_verb::NounVerbError::execution_error(
                                    CliAdapter::format_error(&e),
                                )
                            })?;

                            vec![
                                format!("Case created: {}", response.case_id),
                                format!("Spec ID: {}", spec_id_str),
                            ]
                        }
                        #[cfg(not(feature = "workflow"))]
                        {
                            vec!["Workflow feature not enabled".to_string()]
                        }
                    }
                    "list-cases" => {
                        #[cfg(feature = "workflow")]
                        {
                            let spec_id_str = ctx.workflow_id.clone().unwrap();
                            let spec_id = WorkflowSpecId::parse_str(&spec_id_str).map_err(|e| {
                                clap_noun_verb::NounVerbError::execution_error(format!(
                                    "Invalid spec ID: {}",
                                    e
                                ))
                            })?;

                            let engine = create_engine(&ctx.state_store_path)?;
                            let service = CaseService::new(engine);
                            let request = ListCasesRequest {
                                spec_id: Some(spec_id),
                            };
                            let response = service.list_cases(request).await.map_err(|e| {
                                clap_noun_verb::NounVerbError::execution_error(
                                    CliAdapter::format_error(&e),
                                )
                            })?;

                            if response.cases.is_empty() {
                                vec!["No cases found for this workflow.".to_string()]
                            } else {
                                let mut lines = vec![
                                    format!("Total cases: {}", response.cases.len()),
                                    String::new(),
                                ];
                                for case in response.cases {
                                    lines.push(format!("  - {}", case));
                                }
                                lines
                            }
                        }
                        #[cfg(not(feature = "workflow"))]
                        {
                            vec!["Workflow feature not enabled".to_string()]
                        }
                    }
                    _ => vec![format!(
                        "Unknown command: '{}'. Type 'help' for available commands.",
                        command
                    )],
                }
            })
            .join("\n");

        let duration = start_time.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            command = %command,
            "console.run.success"
        );

        Ok(RunResult {
            status: "success".to_string(),
            output,
            command,
        })
    }

    #[cfg(not(feature = "otel"))]
    {
        // Get current context
        let ctx = get_context()
            .lock()
            .map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to lock console context: {}",
                    e
                ))
            })?
            .clone();

        // Check if workflow is loaded
        if ctx.workflow_id.is_none() {
            return Err(clap_noun_verb::NounVerbError::execution_error(
                "No workflow loaded. Use 'load <file>' first.".to_string(),
            ));
        }

        let runtime = get_runtime();
        let output = runtime
            .block_on(async {
                match command.trim() {
                    "help" => vec![
                        "Available console commands:".to_string(),
                        "  help              - Show this help message".to_string(),
                        "  status            - Show loaded workflow status".to_string(),
                        "  patterns          - List all 43 Van der Aalst patterns".to_string(),
                        "  validate          - Validate loaded workflow".to_string(),
                        "  create-case       - Create new workflow case".to_string(),
                        "  list-cases        - List all workflow cases".to_string(),
                        "  quit              - Exit console".to_string(),
                    ],
                    "status" => vec![
                        format!(
                            "Workflow ID: {}",
                            ctx.workflow_id.clone().unwrap_or_default()
                        ),
                        format!(
                            "Workflow Path: {}",
                            ctx.workflow_path.clone().unwrap_or_default()
                        ),
                        format!(
                            "State Store: {}",
                            ctx.state_store_path
                                .clone()
                                .unwrap_or("./workflow_db".to_string())
                        ),
                    ],
                    "patterns" => vec![
                        "Available patterns (43 Van der Aalst patterns):".to_string(),
                        "  Pattern 1: Sequence".to_string(),
                        "  Pattern 2: Parallel Split".to_string(),
                        "  Pattern 3: Synchronization".to_string(),
                        "  ... and 40 more patterns".to_string(),
                        "Use 'workflow patterns' command for full list.".to_string(),
                    ],
                    "validate" => {
                        #[cfg(feature = "workflow")]
                        {
                            let spec_id_str = ctx.workflow_id.clone().unwrap();
                            let spec_id = WorkflowSpecId::parse_str(&spec_id_str).map_err(|e| {
                                clap_noun_verb::NounVerbError::execution_error(format!(
                                    "Invalid spec ID: {}",
                                    e
                                ))
                            })?;

                            let engine = create_engine(&ctx.state_store_path)?;
                            let framework = ValidationFramework::new(engine);
                            let report =
                                framework
                                    .run_complete_validation(spec_id)
                                    .await
                                    .map_err(|e| {
                                        clap_noun_verb::NounVerbError::execution_error(format!(
                                            "Validation failed: {}",
                                            e
                                        ))
                                    })?;

                            vec![
                                format!("Validation Status: {:?}", report.summary.overall_status),
                                format!(
                                    "Passed phases: {}/{}",
                                    report.summary.passed_phases, report.summary.total_phases
                                ),
                                format!(
                                    "Failed phases: {}/{}",
                                    report.summary.failed_phases, report.summary.total_phases
                                ),
                                format!("Warnings: {}", report.summary.warnings),
                            ]
                        }
                        #[cfg(not(feature = "workflow"))]
                        {
                            vec!["Workflow feature not enabled".to_string()]
                        }
                    }
                    "create-case" => {
                        #[cfg(feature = "workflow")]
                        {
                            let spec_id_str = ctx.workflow_id.clone().unwrap();
                            let spec_id = WorkflowSpecId::parse_str(&spec_id_str).map_err(|e| {
                                clap_noun_verb::NounVerbError::execution_error(format!(
                                    "Invalid spec ID: {}",
                                    e
                                ))
                            })?;

                            let engine = create_engine(&ctx.state_store_path)?;
                            let service = CaseService::new(engine);
                            let request = CreateCaseRequest {
                                spec_id,
                                data: json!({}),
                            };
                            let response = service.create_case(request).await.map_err(|e| {
                                clap_noun_verb::NounVerbError::execution_error(
                                    CliAdapter::format_error(&e),
                                )
                            })?;

                            vec![
                                format!("Case created: {}", response.case_id),
                                format!("Spec ID: {}", spec_id_str),
                            ]
                        }
                        #[cfg(not(feature = "workflow"))]
                        {
                            vec!["Workflow feature not enabled".to_string()]
                        }
                    }
                    "list-cases" => {
                        #[cfg(feature = "workflow")]
                        {
                            let spec_id_str = ctx.workflow_id.clone().unwrap();
                            let spec_id = WorkflowSpecId::parse_str(&spec_id_str).map_err(|e| {
                                clap_noun_verb::NounVerbError::execution_error(format!(
                                    "Invalid spec ID: {}",
                                    e
                                ))
                            })?;

                            let engine = create_engine(&ctx.state_store_path)?;
                            let service = CaseService::new(engine);
                            let request = ListCasesRequest {
                                spec_id: Some(spec_id),
                            };
                            let response = service.list_cases(request).await.map_err(|e| {
                                clap_noun_verb::NounVerbError::execution_error(
                                    CliAdapter::format_error(&e),
                                )
                            })?;

                            if response.cases.is_empty() {
                                vec!["No cases found for this workflow.".to_string()]
                            } else {
                                let mut lines = vec![
                                    format!("Total cases: {}", response.cases.len()),
                                    String::new(),
                                ];
                                for case in response.cases {
                                    lines.push(format!("  - {}", case));
                                }
                                lines
                            }
                        }
                        #[cfg(not(feature = "workflow"))]
                        {
                            vec!["Workflow feature not enabled".to_string()]
                        }
                    }
                    _ => vec![format!(
                        "Unknown command: '{}'. Type 'help' for available commands.",
                        command
                    )],
                }
            })
            .join("\n");

        Ok(RunResult {
            status: "success".to_string(),
            output,
            command,
        })
    }
}

/// Execute SPARQL query on loaded workflow
#[cfg_attr(feature = "otel", instrument(skip_all, fields(operation = "knhk.console.query", query = %query)))]
#[verb]
pub fn query(query: String) -> CnvResult<QueryResult> {
    #[cfg(feature = "otel")]
    {
        use std::time::Instant;
        use tracing::{error, info};

        let start_time = Instant::now();
        let ctx = get_context()
            .lock()
            .map_err(|e| {
                error!(error = %e, "console.query.lock.failed");
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to lock console context: {}",
                    e
                ))
            })?
            .clone();

        if ctx.workflow_id.is_none() {
            error!(query = %query, "console.query.no_workflow");
            return Err(clap_noun_verb::NounVerbError::execution_error(
                "No workflow loaded. Use 'load <file>' first.".to_string(),
            ));
        }

        #[cfg(feature = "workflow")]
        {
            let runtime = get_runtime();
            let result = runtime.block_on(async {
                let spec_id_str = ctx.workflow_id.clone().unwrap();
                let engine = create_engine(&ctx.state_store_path).map_err(|e| {
                    error!(error = ?e, "console.query.engine_creation_failed");
                    e
                })?;

                let spec_id = WorkflowSpecId::parse_str(&spec_id_str).map_err(|e| {
                    error!(error = ?e, "console.query.invalid_spec_id");
                    clap_noun_verb::NounVerbError::execution_error(format!(
                        "Invalid spec ID: {}",
                        e
                    ))
                })?;

                // Execute SPARQL query on RDF store
                #[cfg(feature = "rdf")]
                let results = {
                    let result = engine.query_rdf(&spec_id, &query).await.map_err(|e| {
                        error!(error = ?e, "console.query.execution_failed");
                        clap_noun_verb::NounVerbError::execution_error(format!(
                            "Query execution failed: {}",
                            e
                        ))
                    })?;

                    format!("Query Results ({}): \n{:?}", result.len(), result)
                };

                #[cfg(not(feature = "rdf"))]
                let results = "RDF feature not enabled. Recompile with --features rdf".to_string();

                Ok::<String, clap_noun_verb::NounVerbError>(results)
            })?;

            let duration = start_time.elapsed();
            info!(
                duration_ms = duration.as_millis(),
                query = %query,
                "console.query.success"
            );

            Ok(QueryResult {
                status: "success".to_string(),
                results: result,
                query,
            })
        }

        #[cfg(not(feature = "workflow"))]
        {
            let duration = start_time.elapsed();
            info!(
                duration_ms = duration.as_millis(),
                query = %query,
                "console.query.feature_disabled"
            );

            Ok(QueryResult {
                status: "error".to_string(),
                results: "Workflow feature not enabled".to_string(),
                query,
            })
        }
    }

    #[cfg(not(feature = "otel"))]
    {
        let ctx = get_context()
            .lock()
            .map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to lock console context: {}",
                    e
                ))
            })?
            .clone();

        if ctx.workflow_id.is_none() {
            return Err(clap_noun_verb::NounVerbError::execution_error(
                "No workflow loaded. Use 'load <file>' first.".to_string(),
            ));
        }

        #[cfg(feature = "workflow")]
        {
            let runtime = get_runtime();
            let result = runtime.block_on(async {
                let spec_id_str = ctx.workflow_id.clone().unwrap();
                let engine = create_engine(&ctx.state_store_path)?;

                let spec_id = WorkflowSpecId::parse_str(&spec_id_str).map_err(|e| {
                    clap_noun_verb::NounVerbError::execution_error(format!(
                        "Invalid spec ID: {}",
                        e
                    ))
                })?;

                // Execute SPARQL query on RDF store
                #[cfg(feature = "rdf")]
                let results = {
                    let result = engine.query_rdf(&spec_id, &query).await.map_err(|e| {
                        clap_noun_verb::NounVerbError::execution_error(format!(
                            "Query execution failed: {}",
                            e
                        ))
                    })?;

                    format!("Query Results ({}): \n{:?}", result.len(), result)
                };

                #[cfg(not(feature = "rdf"))]
                let results = "RDF feature not enabled. Recompile with --features rdf".to_string();

                Ok::<String, clap_noun_verb::NounVerbError>(results)
            })?;

            Ok(QueryResult {
                status: "success".to_string(),
                results: result,
                query,
            })
        }

        #[cfg(not(feature = "workflow"))]
        {
            Ok(QueryResult {
                status: "error".to_string(),
                results: "Workflow feature not enabled".to_string(),
                query,
            })
        }
    }
}
