//! Example gRPC client for KNHK workflow engine
//!
//! This example demonstrates how to interact with the workflow engine via gRPC.
//!
//! Prerequisites:
//!   Start the server first: cargo run --example grpc_server --features grpc
//!
//! Usage:
//!   cargo run --example grpc_client --features grpc

use knhk_workflow_engine::api::grpc::proto::{
    workflow_engine_service_client::WorkflowEngineServiceClient, CreateCaseRequest,
    GetCaseRequest, RegisterWorkflowRequest, WorkflowSpec,
};
use tonic::Request;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("Connecting to gRPC server at http://127.0.0.1:50051");

    // Connect to server
    let mut client = WorkflowEngineServiceClient::connect("http://127.0.0.1:50051").await?;

    info!("Connected to gRPC server");

    // Register a workflow
    info!("Registering workflow...");
    let workflow_spec = WorkflowSpec {
        id: "test-workflow-001".to_string(),
        name: "Test Workflow".to_string(),
        spec_data: r#"{
            "id": "test-workflow-001",
            "name": "Test Workflow",
            "tasks": {},
            "conditions": {},
            "flows": []
        }"#
        .to_string(),
    };

    let request = Request::new(RegisterWorkflowRequest {
        spec: Some(workflow_spec),
    });

    let response = client.register_workflow(request).await?;
    let spec_id = response.into_inner().spec_id;

    info!("Workflow registered with ID: {}", spec_id);

    // Create a case
    info!("Creating case for workflow {}...", spec_id);
    let request = Request::new(CreateCaseRequest {
        spec_id: spec_id.clone(),
        data: r#"{"test": "data"}"#.to_string(),
    });

    let response = client.create_case(request).await?;
    let case_id = response.into_inner().case_id;

    info!("Case created with ID: {}", case_id);

    // Get case status
    info!("Fetching case status...");
    let request = Request::new(GetCaseRequest {
        case_id: case_id.clone(),
    });

    let response = client.get_case(request).await?;
    let case = response.into_inner().case.expect("Case should exist");

    info!("Case status:");
    info!("  ID: {}", case.id);
    info!("  Spec ID: {}", case.spec_id);
    info!("  State: {}", case.state);
    info!("  Data: {}", case.data);

    info!("gRPC client demo completed successfully");

    Ok(())
}
