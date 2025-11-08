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
    /// NOTE: Currently returns empty router due to LockchainStorage Sync issue.
    /// LockchainStorage contains git2::Repository which is not Sync, preventing
    /// WorkflowEngine from being used in axum Router state.
    ///
    /// Routes are disabled until LockchainStorage is made thread-safe.
    /// In production, would implement:
    /// - Health check route (GET /health)
    /// - Workflow registration (POST /workflows)
    /// - Case creation (POST /cases)
    /// - Case execution (POST /cases/:id/execute)
    /// - Case status (GET /cases/:id)
    pub fn router(&self) -> Router {
        // Return empty router - routes disabled due to LockchainStorage Sync issue
        // FUTURE: Re-enable when LockchainStorage is thread-safe
        // Example implementation:
        // Router::new()
        //     .route("/health", get(health_check))
        //     .route("/workflows", post(register_workflow))
        //     .route("/cases", post(create_case))
        //     .route("/cases/:id/execute", post(execute_case))
        //     .route("/cases/:id", get(get_case))
        //     .with_state(self.engine.clone())
        Router::new()
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
