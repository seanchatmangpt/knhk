//! Provenance tracking with lockchain integration

// Unused imports removed - will be used when implementing provenance tracking
use crate::error::WorkflowResult;
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Provenance event
#[derive(Debug, Clone)]
pub struct ProvenanceEvent {
    /// Event ID
    pub id: Uuid,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: String,
    /// Resource ID (case ID, spec ID, etc.)
    pub resource_id: String,
    /// Resource type
    pub resource_type: String,
    /// Principal who performed the action
    pub principal: Option<String>,
    /// Event data
    pub data: serde_json::Value,
    /// Lockchain receipt hash (if recorded) - stored as string since Receipt doesn't implement Serialize
    pub receipt_hash: Option<String>,
}

/// Provenance tracker
pub struct ProvenanceTracker {
    events: Arc<Mutex<Vec<ProvenanceEvent>>>,
    lockchain_enabled: bool,
}

impl ProvenanceTracker {
    /// Create a new provenance tracker
    pub fn new(lockchain_enabled: bool) -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            lockchain_enabled,
        }
    }

    /// Record a provenance event
    pub fn record_event(
        &self,
        event_type: String,
        resource_id: String,
        resource_type: String,
        principal: Option<String>,
        data: serde_json::Value,
    ) -> WorkflowResult<Uuid> {
        let event = ProvenanceEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            resource_id,
            resource_type,
            principal,
            data,
            receipt_hash: None,
        };

        let id = event.id;

        // Record to lockchain if enabled
        if self.lockchain_enabled {
            // Note: Lockchain integration is handled via LockchainIntegration
            // This provenance tracker maintains in-memory events for querying
            // Actual lockchain recording happens in executor via LockchainIntegration
        }

        let mut events = self.events.lock().map_err(|e| {
            crate::error::WorkflowError::Internal(format!(
                "Failed to acquire provenance lock: {}",
                e
            ))
        })?;

        events.push(event);

        Ok(id)
    }

    /// Get provenance events for a resource
    pub fn get_events_for_resource(
        &self,
        resource_id: &str,
    ) -> WorkflowResult<Vec<ProvenanceEvent>> {
        let events = self.events.lock().map_err(|e| {
            crate::error::WorkflowError::Internal(format!(
                "Failed to acquire provenance lock: {}",
                e
            ))
        })?;

        Ok(events
            .iter()
            .filter(|e| e.resource_id == resource_id)
            .cloned()
            .collect())
    }

    /// Get all events
    pub fn get_all_events(&self) -> WorkflowResult<Vec<ProvenanceEvent>> {
        let events = self.events.lock().map_err(|e| {
            crate::error::WorkflowError::Internal(format!(
                "Failed to acquire provenance lock: {}",
                e
            ))
        })?;

        Ok(events.clone())
    }
}

impl Default for ProvenanceTracker {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provenance_tracker() {
        let tracker = ProvenanceTracker::default();
        let id = tracker
            .record_event(
                "create_case".to_string(),
                "case-1".to_string(),
                "Case".to_string(),
                Some("user-1".to_string()),
                serde_json::json!({}),
            )
            .expect("record_event should succeed");

        let events = tracker
            .get_events_for_resource("case-1")
            .expect("get_events_for_resource should succeed");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, id);
    }
}
