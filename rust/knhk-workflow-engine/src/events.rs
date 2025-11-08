//! Event sidecar for workflow engine
//!
//! Provides event bus integration for external events and timer-fired events.

use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Event sidecar for publishing workflow events
#[derive(Clone)]
pub struct EventSidecar {
    /// Event channel sender
    tx: mpsc::Sender<Value>,
}

impl EventSidecar {
    /// Create a new event sidecar
    pub fn new(tx: mpsc::Sender<Value>) -> Self {
        Self { tx }
    }

    /// Publish an event
    pub async fn publish(&self, event: Value) -> Result<(), EventError> {
        self.tx
            .send(event)
            .await
            .map_err(|_| EventError::ChannelClosed)
    }

    /// Publish a case event
    pub async fn publish_case_event(
        &self,
        case_id: &str,
        workflow_id: &str,
        event_type: &str,
        data: Value,
    ) -> Result<(), EventError> {
        let event = serde_json::json!({
            "case_id": case_id,
            "workflow_id": workflow_id,
            "event_type": event_type,
            "data": data,
        });
        self.publish(event).await
    }
}

/// Event error
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Event channel closed")]
    ChannelClosed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_sidecar() {
        let (tx, mut rx) = mpsc::channel(10);
        let sidecar = EventSidecar::new(tx);

        let event = serde_json::json!({"test": "value"});
        sidecar.publish(event.clone()).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received, event);
    }
}

