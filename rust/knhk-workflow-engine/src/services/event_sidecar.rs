//! Event sidecar for external event handling
//!
//! Handles:
//! - Pattern 16: Deferred Choice (event-driven choice)
//! - External event routing to workflow cases
//!
//! Events are sent via async channel to the engine for pattern execution.

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Event sidecar for external event handling
pub struct EventSidecar {
    /// Event sender
    event_tx: mpsc::Sender<serde_json::Value>,
}

/// External event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalEvent {
    /// Event type
    pub event_type: String,
    /// Case ID (if applicable)
    pub case_id: Option<String>,
    /// Workflow ID (if applicable)
    pub workflow_id: Option<String>,
    /// Event data
    pub data: serde_json::Value,
}

impl EventSidecar {
    /// Create a new event sidecar
    pub fn new(event_tx: mpsc::Sender<serde_json::Value>) -> Self {
        Self { event_tx }
    }

    /// Send an external event
    pub async fn send_event(&self, event: ExternalEvent) -> Result<(), String> {
        let json_event = serde_json::json!({
            "event_type": event.event_type,
            "case_id": event.case_id,
            "workflow_id": event.workflow_id,
            "data": event.data,
        });

        self.event_tx
            .send(json_event)
            .await
            .map_err(|e| format!("Failed to send event: {}", e))
    }

    /// Send a raw JSON event
    pub async fn send_json(&self, event: serde_json::Value) -> Result<(), String> {
        self.event_tx
            .send(event)
            .await
            .map_err(|e| format!("Failed to send event: {}", e))
    }
}

impl Clone for EventSidecar {
    fn clone(&self) -> Self {
        Self {
            event_tx: self.event_tx.clone(),
        }
    }
}
