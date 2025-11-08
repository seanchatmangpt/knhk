//! Lockchain integration for provenance

use crate::case::CaseId;
use crate::error::WorkflowResult;
use knhk_lockchain::LockchainStorage;

/// Lockchain integration for workflow provenance
pub struct LockchainIntegration {
    lockchain: LockchainStorage,
}

impl LockchainIntegration {
    /// Create new lockchain integration
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> WorkflowResult<Self> {
        let lockchain = LockchainStorage::new(path).map_err(|e| {
            crate::error::WorkflowError::Internal(format!("Failed to initialize lockchain: {}", e))
        })?;
        Ok(Self { lockchain })
    }

    /// Record workflow execution event
    pub fn record_event(
        &mut self,
        _case_id: CaseId,
        _event_type: &str,
        _data: serde_json::Value,
    ) -> WorkflowResult<()> {
        // FUTURE: Implement lockchain event recording
        // This requires understanding the lockchain API better
        // For now, this is a placeholder
        Ok(())
    }
}
