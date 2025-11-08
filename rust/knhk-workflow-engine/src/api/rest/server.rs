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
    pub fn router(&self) -> Router {
        // FUTURE: Fix LockchainStorage Sync issue and axum handler signatures
        // For now, return empty router to allow compilation
        // LockchainStorage contains git2::Repository which is not Sync
        // This prevents WorkflowEngine from being used in axum Router
        // All routes are disabled until LockchainStorage is made thread-safe
        Router::new()
        // FUTURE: Re-enable when LockchainStorage is thread-safe
        // .with_state(self.engine.clone())
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
