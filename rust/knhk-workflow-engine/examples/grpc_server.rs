//! Example gRPC server for KNHK workflow engine
//!
//! This example demonstrates how to start a gRPC server for the workflow engine.
//!
//! Usage:
//!   cargo run --example grpc_server --features grpc
//!
//! Test with grpcurl:
//!   grpcurl -plaintext localhost:50051 list
//!   grpcurl -plaintext localhost:50051 knhk.workflow_engine.v1.WorkflowEngineService/RegisterWorkflow

use knhk_workflow_engine::api::{GrpcServer, GrpcServerConfig};
use knhk_workflow_engine::state::StateStore;
use knhk_workflow_engine::WorkflowEngine;
use std::sync::Arc;
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

    info!("Initializing KNHK workflow engine");

    // Create state store
    let state_store = StateStore::new("./data/workflow_db")?;

    // Create workflow engine
    let engine = WorkflowEngine::new(state_store);
    let engine = Arc::new(engine);

    info!("Workflow engine initialized");

    // Create gRPC server with custom configuration
    let config = GrpcServerConfig {
        bind_addr: "0.0.0.0:50051".parse()?,
        max_concurrent_streams: Some(100),
        tcp_keepalive: Some(std::time::Duration::from_secs(60)),
        tcp_nodelay: true,
        http2_adaptive_window: true,
    };

    let server = GrpcServer::new(engine, config);

    info!("Starting gRPC server...");
    info!("Server will listen on {}", server.bind_addr());
    info!("Press CTRL-C to shutdown");

    // Start server with graceful shutdown
    server.serve_with_shutdown().await?;

    Ok(())
}
