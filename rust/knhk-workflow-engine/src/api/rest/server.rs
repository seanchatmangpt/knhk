//! REST API server implementation

use crate::executor::WorkflowEngine;
use axum::Router;
use std::sync::Arc;

/// REST API server
pub struct RestApiServer {
    engine: Arc<WorkflowEngine>,
    /// Enable Fortune 5 features
    fortune5_enabled: bool,
}

impl RestApiServer {
    /// Create a new REST API server
    pub fn new(engine: Arc<WorkflowEngine>) -> Self {
        Self {
            engine,
            fortune5_enabled: false,
        }
    }

    /// Create a new REST API server with Fortune 5 features
    pub fn with_fortune5(engine: Arc<WorkflowEngine>) -> Self {
        Self {
            engine,
            fortune5_enabled: true,
        }
    }

    /// Create the router
    ///
    /// Provides REST API routes for workflow management:
    /// - Health check route (GET /health)
    /// - Workflow registration (POST /workflows)
    /// - Case creation (POST /cases)
    /// - Case execution (POST /cases/:id/execute)
    /// - Case status (GET /cases/:id)
    pub fn router(&self) -> Router {
        use crate::api::rest::handlers;
        use axum::routing::{get, post};

        Router::new()
            .route("/health", get(handlers::health))
            .route("/workflows", post(handlers::register_workflow))
            .route("/cases", post(handlers::create_case))
            .route("/cases/:id/execute", post(handlers::execute_case))
            .route("/cases/:id/history", get(handlers::get_case_history))
            .route("/cases/:id", get(handlers::get_case))
            .with_state(self.engine.clone())
    }

    /// Get engine reference
    pub fn engine(&self) -> &Arc<WorkflowEngine> {
        &self.engine
    }

    /// Check if Fortune 5 is enabled
    pub fn fortune5_enabled(&self) -> bool {
        self.fortune5_enabled
    }
}
