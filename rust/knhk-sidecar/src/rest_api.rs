//! REST API server for workflow engine
//!
//! Provides REST API endpoints for workflow management when `rest-api` feature is enabled.
//! This allows the sidecar to serve both gRPC and REST APIs.

#[cfg(feature = "rest-api")]
use knhk_workflow_engine::{
    api::rest::RestApiServer as WorkflowRestApiServer, state::StateStore, WorkflowEngine,
};
#[cfg(feature = "rest-api")]
use std::sync::Arc;
#[cfg(feature = "rest-api")]
use tokio::net::TcpListener;

/// REST API server configuration
#[cfg(feature = "rest-api")]
#[derive(Debug, Clone)]
pub struct RestApiConfig {
    /// Bind address for REST API
    pub bind_address: String,
    /// State store path for workflow engine
    pub state_store_path: String,
}

#[cfg(feature = "rest-api")]
impl Default for RestApiConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:8080".to_string(),
            state_store_path: "./workflow_db".to_string(),
        }
    }
}

/// REST API server wrapper
#[cfg(feature = "rest-api")]
pub struct SidecarRestApiServer {
    config: RestApiConfig,
    engine: Arc<WorkflowEngine>,
}

#[cfg(feature = "rest-api")]
impl SidecarRestApiServer {
    /// Create a new REST API server
    pub async fn new(config: RestApiConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let state_store = StateStore::new(&config.state_store_path)
            .map_err(|e| format!("Failed to create state store: {}", e))?;
        let engine = Arc::new(WorkflowEngine::new(state_store));

        Ok(Self { config, engine })
    }

    /// Start the REST API server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        use tracing::info;

        let app = WorkflowRestApiServer::new(self.engine.clone()).router();

        let listener = TcpListener::bind(&self.config.bind_address)
            .await
            .map_err(|e| format!("Failed to bind to {}: {}", self.config.bind_address, e))?;

        info!(
            "REST API server listening on http://{}",
            self.config.bind_address
        );
        info!("API endpoints:");
        info!("  GET  /health - Health check");
        info!("  POST /workflows - Register workflow");
        info!("  GET  /workflows/:id - Get workflow");
        info!("  POST /cases - Create case");
        info!("  GET  /cases/:id - Get case status");
        info!("  POST /cases/:id/execute - Execute case");
        info!("  GET  /cases/:id/history - Get case history");

        axum::serve(listener, app)
            .await
            .map_err(|e| format!("REST API server error: {}", e))?;

        Ok(())
    }
}

/// Start REST API server (feature-gated)
#[cfg(feature = "rest-api")]
pub async fn start_rest_api(config: RestApiConfig) -> Result<(), Box<dyn std::error::Error>> {
    let server = SidecarRestApiServer::new(config).await?;
    server.start().await
}

/// Start REST API server (no-op when feature disabled)
#[cfg(not(feature = "rest-api"))]
pub async fn start_rest_api(_config: ()) -> Result<(), Box<dyn std::error::Error>> {
    Err("REST API feature not enabled. Enable with --features rest-api".into())
}
