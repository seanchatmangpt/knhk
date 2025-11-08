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
    ///
    /// NOTE: This is a stub implementation. The sidecar now depends on the workflow engine,
    /// not the other way around. This stub allows the workflow engine to compile without
    /// the sidecar dependency.
    ///
    /// In production, the sidecar would initialize itself and connect to the workflow engine.
    pub async fn initialize(&self) -> WorkflowResult<()> {
        Err(WorkflowError::ResourceUnavailable(
            "Sidecar integration is not available from workflow engine side. \
             The sidecar depends on the workflow engine, not the other way around. \
             To use sidecar functionality, initialize the sidecar separately and connect it to the workflow engine."
                .to_string(),
        ))
    }

    /// Refresh sidecar certificates (stub - no-op)
    ///
    /// NOTE: This is a stub implementation. Certificate refresh should be handled by
    /// the sidecar itself, not by the workflow engine.
    pub async fn refresh_certificates(&self) -> WorkflowResult<()> {
        Err(WorkflowError::ResourceUnavailable(
            "Sidecar certificate refresh is not available from workflow engine side. \
             The sidecar manages its own certificates. \
             To refresh certificates, use the sidecar's certificate management API."
                .to_string(),
        ))
    }
}
