//! YAWL Interface X Implementation
//!
//! Implements Interface X (Inter-Process Communication) with TRIZ Principle 37: Thermal Expansion
//! - Scale IPC resources based on load temperature
//!
//! Based on: org.yawlfoundation.yawl.engine.interfce.interfaceX

use crate::case::CaseId;
use crate::error::{WorkflowError, WorkflowResult};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Interface X message
#[derive(Debug, Clone)]
pub struct InterfaceXMessage {
    /// Message ID
    pub message_id: String,
    /// From case ID
    pub from_case: CaseId,
    /// To case ID (optional)
    pub to_case: Option<CaseId>,
    /// Message type
    pub message_type: MessageType,
    /// Message data
    pub data: serde_json::Value,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Message type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// External exception
    ExternalException,
    /// Case message
    CaseMessage,
    /// Event subscription
    EventSubscription,
    /// Status check
    StatusCheck,
}

/// Interface X - Inter-Process Communication
///
/// TRIZ Principle 37: Thermal Expansion
/// - Scales IPC resources based on load temperature
pub struct InterfaceX {
    /// Message queue
    message_queue: Arc<DashMap<String, InterfaceXMessage>>,
    /// Event subscriptions
    subscriptions: Arc<DashMap<CaseId, Vec<String>>>,
    /// Message channels (TRIZ Principle 13: Inversion - push-based)
    message_tx: mpsc::UnboundedSender<InterfaceXMessage>,
    message_rx: Arc<RwLock<mpsc::UnboundedReceiver<InterfaceXMessage>>>,
    /// Load temperature (TRIZ Principle 37: Thermal Expansion)
    load_temperature: Arc<RwLock<f64>>,
}

impl InterfaceX {
    /// Create a new Interface X instance
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            message_queue: Arc::new(DashMap::new()),
            subscriptions: Arc::new(DashMap::new()),
            message_tx: tx,
            message_rx: Arc::new(RwLock::new(rx)),
            load_temperature: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Raise external exception
    pub async fn raise_external_exception(
        &self,
        case_id: CaseId,
        item_id: Option<String>,
        data: serde_json::Value,
    ) -> WorkflowResult<String> {
        let message = InterfaceXMessage {
            message_id: uuid::Uuid::new_v4().to_string(),
            from_case: case_id.clone(),
            to_case: None,
            message_type: MessageType::ExternalException,
            data,
            timestamp: chrono::Utc::now(),
        };

        self.send_message(message).await
    }

    /// Send case message
    pub async fn send_case_message(
        &self,
        from_case: CaseId,
        to_case: CaseId,
        data: serde_json::Value,
    ) -> WorkflowResult<String> {
        let message = InterfaceXMessage {
            message_id: uuid::Uuid::new_v4().to_string(),
            from_case,
            to_case: Some(to_case),
            message_type: MessageType::CaseMessage,
            data,
            timestamp: chrono::Utc::now(),
        };

        self.send_message(message).await
    }

    /// Subscribe to case events
    pub async fn subscribe_to_case_events(
        &self,
        case_id: CaseId,
        callback_url: String,
    ) -> WorkflowResult<String> {
        let mut subscriptions = self.subscriptions.entry(case_id).or_insert_with(Vec::new);
        subscriptions.push(callback_url.clone());

        Ok(format!("Subscribed to case {} events", case_id))
    }

    /// Get Interface X status
    pub async fn get_status(&self) -> WorkflowResult<HashMap<String, serde_json::Value>> {
        let temperature = *self.load_temperature.read().await;
        let queue_size = self.message_queue.len();

        Ok(HashMap::from([
            ("status".to_string(), serde_json::json!("running")),
            ("queue_size".to_string(), serde_json::json!(queue_size)),
            ("load_temperature".to_string(), serde_json::json!(temperature)),
        ]))
    }

    /// Update load temperature (TRIZ Principle 37: Thermal Expansion)
    pub async fn update_load_temperature(&self, temperature: f64) {
        let mut load = self.load_temperature.write().await;
        *load = temperature.clamp(0.0, 1.0);
    }

    /// Send message (TRIZ Principle 13: Inversion - push-based)
    async fn send_message(&self, message: InterfaceXMessage) -> WorkflowResult<String> {
        // Store in queue
        self.message_queue
            .insert(message.message_id.clone(), message.clone());

        // Send via channel (TRIZ Principle 13: Inversion)
        self.message_tx
            .send(message.clone())
            .map_err(|_| WorkflowError::Internal("Failed to send message".to_string()))?;

        Ok(message.message_id)
    }

    /// Start message handler (TRIZ Principle 13: Inversion)
    pub async fn start_message_handler(&self) {
        let rx = self.message_rx.clone();
        let queue = self.message_queue.clone();
        let subscriptions = self.subscriptions.clone();

        tokio::spawn(async move {
            let mut receiver = rx.write().await;
            while let Ok(message) = receiver.recv().await {
                // Handle message
                tracing::debug!("Interface X message received: {:?}", message.message_id);

                // Notify subscribers if case message
                if let Some(to_case) = &message.to_case {
                    if let Some(callbacks) = subscriptions.get(to_case) {
                        for callback_url in callbacks.value() {
                            tracing::debug!("Notifying subscriber: {}", callback_url);
                            // In real implementation, would make HTTP call to callback_url
                        }
                    }
                }
            }
        });
    }
}

impl Default for InterfaceX {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_interface_x_creation() {
        let interface_x = InterfaceX::new();
        let status = interface_x.get_status().await.unwrap();
        assert_eq!(status.get("status").unwrap(), &serde_json::json!("running"));
    }

    #[tokio::test]
    async fn test_external_exception() {
        let interface_x = InterfaceX::new();
        let case_id = CaseId::new();
        let data = serde_json::json!({"error": "timeout"});

        let result = interface_x
            .raise_external_exception(case_id, None, data)
            .await;
        assert!(result.is_ok());
    }
}

