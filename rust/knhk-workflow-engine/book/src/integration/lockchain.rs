//! Lockchain integration for workflow provenance
//!
//! Records workflow execution events to the lockchain for audit trails
//! and compliance requirements.

use crate::case::CaseId;
use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpecId;
use crate::patterns::PatternId;
use knhk_lockchain::LockchainStorage;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Lockchain integration for workflow provenance
pub struct LockchainIntegration {
    lockchain: Arc<RwLock<LockchainStorage>>,
}

impl LockchainIntegration {
    /// Create new lockchain integration
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> WorkflowResult<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let lockchain = LockchainStorage::new(&path_str).map_err(|e| {
            WorkflowError::Internal(format!("Failed to initialize lockchain storage: {:?}", e))
        })?;
        Ok(Self {
            lockchain: Arc::new(RwLock::new(lockchain)),
        })
    }

    /// Record workflow registration event
    pub async fn record_workflow_registration(
        &self,
        spec_id: &WorkflowSpecId,
        spec_name: &str,
    ) -> WorkflowResult<()> {
        let event_data = serde_json::json!({
            "event_type": "workflow.registered",
            "spec_id": spec_id.to_string(),
            "spec_name": spec_name,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.record_event_internal("workflow.registered", &event_data)
            .await
    }

    /// Record case creation event
    pub async fn record_case_created(
        &self,
        case_id: &CaseId,
        spec_id: &WorkflowSpecId,
    ) -> WorkflowResult<()> {
        let event_data = serde_json::json!({
            "event_type": "case.created",
            "case_id": case_id.to_string(),
            "spec_id": spec_id.to_string(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.record_event_internal("case.created", &event_data)
            .await
    }

    /// Record case execution event
    pub async fn record_case_executed(
        &self,
        case_id: &CaseId,
        success: bool,
    ) -> WorkflowResult<()> {
        let event_data = serde_json::json!({
            "event_type": "case.executed",
            "case_id": case_id.to_string(),
            "success": success,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.record_event_internal("case.executed", &event_data)
            .await
    }

    /// Record pattern execution event
    pub async fn record_pattern_executed(
        &self,
        case_id: &CaseId,
        pattern_id: &PatternId,
        success: bool,
        ticks: u32,
    ) -> WorkflowResult<()> {
        let event_data = serde_json::json!({
            "event_type": "pattern.executed",
            "case_id": case_id.to_string(),
            "pattern_id": pattern_id,
            "success": success,
            "ticks": ticks,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.record_event_internal("pattern.executed", &event_data)
            .await
    }

    /// Internal method to record events
    async fn record_event_internal(
        &self,
        _event_type: &str,
        data: &serde_json::Value,
    ) -> WorkflowResult<()> {
        let mut lockchain = self.lockchain.write().await;

        // Serialize event data
        let event_bytes = serde_json::to_vec(data).map_err(|e| {
            WorkflowError::Internal(format!("Failed to serialize event data: {:?}", e))
        })?;

        // Hash the event data for lockchain entry
        let mut hasher = Sha256::new();
        hasher.update(&event_bytes);
        let receipt_hash: [u8; 32] = hasher.finalize().into();

        // Use cycle number based on timestamp (simplified - in production would use actual cycle)
        let cycle = chrono::Utc::now().timestamp() as u64;

        // Append to Git repository for audit trail
        lockchain.append_to_git(&receipt_hash, cycle).map_err(|e| {
            WorkflowError::Internal(format!("Failed to append event to lockchain: {:?}", e))
        })?;

        Ok(())
    }
}
