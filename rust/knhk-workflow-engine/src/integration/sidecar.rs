//! Sidecar Integration for Workflow Engine
//!
//! NOTE: This is a stub implementation to avoid circular dependency.
//! The sidecar now depends on the workflow engine, not the other way around.
//! This stub allows the workflow engine to compile without the sidecar dependency.

use crate::error::{WorkflowError, WorkflowResult};

/// Sidecar integration for workflow engine
/// NOTE: Disabled to avoid circular dependency - sidecar now depends on workflow engine
/// This is a stub implementation that allows the workflow engine to compile
/// without the sidecar dependency. The sidecar now integrates with the workflow engine.
#[derive(Default)]
pub struct SidecarIntegration {
    /// Whether sidecar integration is enabled (always false in this stub)
    _enabled: bool,
}

impl SidecarIntegration {
    /// Create new sidecar integration (stub implementation)
    pub fn new(_enabled: bool) -> Self {
        Self { _enabled: false }
    }

    /// Initialize sidecar integration (stub - no-op)
    pub async fn initialize(&self) -> WorkflowResult<()> {
        Ok(())
    }

    /// Refresh sidecar certificates (stub - no-op)
    pub async fn refresh_certificates(&self) -> WorkflowResult<()> {
        Ok(())
    }
}
