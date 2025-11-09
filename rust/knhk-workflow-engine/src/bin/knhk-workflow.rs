//! KNHK Workflow Engine CLI
//!
//! Enterprise workflow engine CLI for managing YAWL workflows and cases.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use clap::{Parser, Subcommand};
use knhk_workflow_engine::{
    executor::WorkflowEngine, parser::WorkflowParser, state::StateStore, CaseId, WorkflowSpecId,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "knhk-workflow")]
#[command(about = "Enterprise workflow engine with full 43-pattern YAWL support")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// State store path
    #[arg(long, default_value = "./workflow_db")]
    state_store: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a workflow from Turtle file
    Parse {
        /// Turtle file path
        #[arg(short, long)]
        file: PathBuf,
        /// Output JSON file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Register a workflow specification
    Register {
        /// Workflow specification file (Turtle or JSON)
        #[arg(short, long)]
        file: PathBuf,
    },

    /// List all registered workflows
    ListWorkflows,

    /// Get workflow specification
    GetWorkflow {
        /// Workflow specification ID
        spec_id: String,
    },

    /// Create a new workflow case
    CreateCase {
        /// Workflow specification ID
        spec_id: String,
        /// Case data (JSON)
        #[arg(short, long)]
        data: Option<String>,
    },

    /// Start a workflow case
    StartCase {
        /// Case ID
        case_id: String,
    },

    /// Execute a workflow case
    ExecuteCase {
        /// Case ID
        case_id: String,
    },

    /// Get case status
    GetCase {
        /// Case ID
        case_id: String,
    },

    /// Cancel a workflow case
    CancelCase {
        /// Case ID
        case_id: String,
    },

    /// List all cases for a workflow
    ListCases {
        /// Workflow specification ID
        spec_id: String,
    },

    /// Start REST API server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Host to bind to
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
    },

    /// List all registered patterns
    ListPatterns,

    /// Export case to XES format for ProM process mining
    ExportXes {
        /// Case ID to export
        case_id: String,
        /// Output XES file path
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Export all cases for a workflow to XES format
    ExportWorkflowXes {
        /// Workflow specification ID
        spec_id: String,
        /// Output XES file path
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Export all cases (all workflows) to XES format
    ExportAllXes {
        /// Output XES file path
        #[arg(short, long)]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Create state store
    let state_store = StateStore::new(&cli.state_store)
        .map_err(|e| format!("Failed to create state store: {}", e))?;

    // Create workflow engine
    let engine = std::sync::Arc::new(WorkflowEngine::new(state_store));

    match cli.command {
        Commands::Parse { file, output } => {
            let mut parser =
                WorkflowParser::new().map_err(|e| format!("Failed to create parser: {}", e))?;
            let spec = parser
                .parse_file(&file)
                .map_err(|e| format!("Failed to parse workflow: {}", e))?;

            let json = serde_json::to_string_pretty(&spec)
                .map_err(|e| format!("Failed to serialize workflow: {}", e))?;

            if let Some(output_path) = output {
                std::fs::write(&output_path, json)
                    .map_err(|e| format!("Failed to write output file: {}", e))?;
                println!("Workflow parsed and saved to {}", output_path.display());
            } else {
                println!("{}", json);
            }
        }

        Commands::Register { file } => {
            let mut parser =
                WorkflowParser::new().map_err(|e| format!("Failed to create parser: {}", e))?;

            let spec = if file.extension().and_then(|s| s.to_str()) == Some("ttl") {
                parser
                    .parse_file(&file)
                    .map_err(|e| format!("Failed to parse Turtle file: {}", e))?
            } else {
                // Try JSON
                let contents = std::fs::read_to_string(&file)
                    .map_err(|e| format!("Failed to read file: {}", e))?;
                serde_json::from_str(&contents)
                    .map_err(|e| format!("Failed to parse JSON: {}", e))?
            };

            engine
                .register_workflow(spec.clone())
                .await
                .map_err(|e| format!("Failed to register workflow: {}", e))?;

            println!("Workflow registered: {} ({})", spec.name, spec.id);
        }

        Commands::ListWorkflows => {
            // Note: This requires adding a list_workflows method to WorkflowEngine
            println!("Listing workflows...");
            println!("(Feature not yet implemented - workflows are stored in memory)");
        }

        Commands::GetWorkflow { spec_id } => {
            let spec_id = WorkflowSpecId::parse_str(&spec_id)
                .map_err(|e| format!("Invalid spec ID: {}", e))?;
            let spec = engine
                .get_workflow(spec_id)
                .await
                .map_err(|e| format!("Failed to get workflow: {}", e))?;
            let json = serde_json::to_string_pretty(&spec)
                .map_err(|e| format!("Failed to serialize workflow: {}", e))?;
            println!("{}", json);
        }

        Commands::CreateCase { spec_id, data } => {
            let spec_id = WorkflowSpecId::parse_str(&spec_id)
                .map_err(|e| format!("Invalid spec ID: {}", e))?;
            let case_data = if let Some(data_str) = data {
                serde_json::from_str(&data_str).map_err(|e| format!("Invalid JSON data: {}", e))?
            } else {
                serde_json::json!({})
            };

            let case_id = engine
                .create_case(spec_id, case_data)
                .await
                .map_err(|e| format!("Failed to create case: {}", e))?;
            println!("Case created: {}", case_id);
        }

        Commands::StartCase { case_id } => {
            let case_id =
                CaseId::parse_str(&case_id).map_err(|e| format!("Invalid case ID: {}", e))?;
            engine
                .start_case(case_id)
                .await
                .map_err(|e| format!("Failed to start case: {}", e))?;
            println!("Case started: {}", case_id);
        }

        Commands::ExecuteCase { case_id } => {
            let case_id =
                CaseId::parse_str(&case_id).map_err(|e| format!("Invalid case ID: {}", e))?;
            engine
                .execute_case(case_id)
                .await
                .map_err(|e| format!("Failed to execute case: {}", e))?;
            println!("Case executed: {}", case_id);
        }

        Commands::GetCase { case_id } => {
            let case_id =
                CaseId::parse_str(&case_id).map_err(|e| format!("Invalid case ID: {}", e))?;
            let case = engine
                .get_case(case_id)
                .await
                .map_err(|e| format!("Failed to get case: {}", e))?;
            let json = serde_json::to_string_pretty(&case)
                .map_err(|e| format!("Failed to serialize case: {}", e))?;
            println!("{}", json);
        }

        Commands::CancelCase { case_id } => {
            let case_id =
                CaseId::parse_str(&case_id).map_err(|e| format!("Invalid case ID: {}", e))?;
            engine
                .cancel_case(case_id)
                .await
                .map_err(|e| format!("Failed to cancel case: {}", e))?;
            println!("Case cancelled: {}", case_id);
        }

        Commands::ListCases { spec_id } => {
            let spec_id = WorkflowSpecId::parse_str(&spec_id)
                .map_err(|e| format!("Invalid spec ID: {}", e))?;
            // Note: This requires adding a list_cases method to WorkflowEngine
            println!("Listing cases for workflow {}...", spec_id);
            println!("(Feature not yet implemented)");
        }

        Commands::Serve { port, host } => {
            println!("Starting REST API server on {}:{}", host, port);
            use knhk_workflow_engine::api::rest::RestApiServer;
            let app = RestApiServer::new(engine.clone()).router();
            use std::net::SocketAddr;
            let addr: SocketAddr = format!("{}:{}", host, port)
                .parse()
                .map_err(|e| format!("Invalid address {}:{}: {}", host, port, e))?;
            println!("Server listening on http://{}:{}", host, port);
            use tokio::net::TcpListener;
            let listener = TcpListener::bind(&addr)
                .await
                .map_err(|e| format!("Failed to bind to {}:{}: {}", host, port, e))?;
            axum::serve(listener, app)
                .await
                .map_err(|e| format!("Server error: {}", e))?;
        }

        Commands::ListPatterns => {
            let registry = engine.pattern_registry();
            let patterns = registry.list_patterns();
            println!("Registered patterns ({}):", patterns.len());
            for pattern_id in patterns {
                println!("  - Pattern {}", pattern_id.0);
            }
        }

        Commands::ExportXes { case_id, output } => {
            let case_id =
                CaseId::parse_str(&case_id).map_err(|e| format!("Invalid case ID: {}", e))?;
            let xes = engine
                .export_case_to_xes(case_id)
                .await
                .map_err(|e| format!("Failed to export case to XES: {}", e))?;

            std::fs::write(&output, xes).map_err(|e| format!("Failed to write XES file: {}", e))?;

            println!("Case {} exported to XES: {}", case_id, output.display());
            println!("Import into ProM: prom --import {}", output.display());
        }

        Commands::ExportWorkflowXes { spec_id, output } => {
            let spec_id = WorkflowSpecId::parse_str(&spec_id)
                .map_err(|e| format!("Invalid spec ID: {}", e))?;
            let xes = engine
                .export_workflow_to_xes(spec_id)
                .await
                .map_err(|e| format!("Failed to export workflow to XES: {}", e))?;

            std::fs::write(&output, xes).map_err(|e| format!("Failed to write XES file: {}", e))?;

            println!("Workflow {} exported to XES: {}", spec_id, output.display());
            println!(
                "Discover process model: prom --discover-model {} --output model.pnml",
                output.display()
            );
        }

        Commands::ExportAllXes { output } => {
            let xes = engine
                .export_all_cases_to_xes()
                .await
                .map_err(|e| format!("Failed to export all cases to XES: {}", e))?;

            std::fs::write(&output, xes).map_err(|e| format!("Failed to write XES file: {}", e))?;

            println!("All cases exported to XES: {}", output.display());
            println!(
                "Check conformance: prom --check-conformance model.pnml {}",
                output.display()
            );
        }
    }

    Ok(())
}
