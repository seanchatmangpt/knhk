#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Audit logging for compliance and security

use crate::case::CaseId;
use crate::error::WorkflowResult;
use crate::parser::WorkflowSpecId;
use crate::security::auth::Principal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Audit event level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditLevel {
    /// Informational event
    Info,
    /// Warning event
    Warning,
    /// Error event
    Error,
    /// Critical security event
    Critical,
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event ID
    pub id: Uuid,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event level
    pub level: AuditLevel,
    /// Principal who performed the action
    pub principal: Option<Principal>,
    /// Action performed
    pub action: String,
    /// Resource affected (workflow ID, case ID, etc.)
    pub resource: Option<String>,
    /// Resource type
    pub resource_type: Option<String>,
    /// Success or failure
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Audit logger for compliance
pub struct AuditLogger {
    events: Arc<Mutex<Vec<AuditEvent>>>,
    max_events: usize,
    /// Lockchain integration for immutable audit trail
    lockchain_enabled: bool,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(max_events: usize, lockchain_enabled: bool) -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            max_events,
            lockchain_enabled,
        }
    }

    /// Log an audit event
    pub fn log(&self, event: AuditEvent) -> WorkflowResult<()> {
        let mut events = self.events.lock().map_err(|e| {
            crate::error::WorkflowError::Internal(format!("Failed to acquire audit lock: {}", e))
        })?;

        // Remove oldest events if at capacity
        while events.len() >= self.max_events {
            events.remove(0);
        }

        events.push(event.clone());

        // Lockchain integration for immutable audit trail
        if self.lockchain_enabled {
            // FUTURE: Integrate with knhk-lockchain to create a receipt
            // When lockchain_enabled is true but integration is not implemented,
            // this is a false positive - we claim to integrate but don't
            // For now, return unimplemented to indicate incomplete implementation
            unimplemented!("log: needs knhk-lockchain integration to create immutable audit receipt - event_id={}, action={}, resource={:?}", event.id, event.action, event.resource)
        }

        Ok(())
    }

    /// Log workflow registration
    pub fn log_workflow_registration(
        &self,
        principal: Option<&Principal>,
        spec_id: WorkflowSpecId,
        success: bool,
        error: Option<String>,
    ) -> WorkflowResult<()> {
        self.log(AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level: if success {
                AuditLevel::Info
            } else {
                AuditLevel::Error
            },
            principal: principal.cloned(),
            action: "register_workflow".to_string(),
            resource: Some(spec_id.to_string()),
            resource_type: Some("WorkflowSpec".to_string()),
            success,
            error,
            metadata: serde_json::json!({}),
        })
    }

    /// Log case creation
    pub fn log_case_creation(
        &self,
        principal: Option<&Principal>,
        case_id: CaseId,
        spec_id: WorkflowSpecId,
        success: bool,
        error: Option<String>,
    ) -> WorkflowResult<()> {
        self.log(AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level: if success {
                AuditLevel::Info
            } else {
                AuditLevel::Error
            },
            principal: principal.cloned(),
            action: "create_case".to_string(),
            resource: Some(case_id.to_string()),
            resource_type: Some("Case".to_string()),
            success,
            error,
            metadata: serde_json::json!({
                "spec_id": spec_id.to_string(),
            }),
        })
    }

    /// Log case execution
    pub fn log_case_execution(
        &self,
        principal: Option<&Principal>,
        case_id: CaseId,
        success: bool,
        error: Option<String>,
    ) -> WorkflowResult<()> {
        self.log(AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level: if success {
                AuditLevel::Info
            } else {
                AuditLevel::Error
            },
            principal: principal.cloned(),
            action: "execute_case".to_string(),
            resource: Some(case_id.to_string()),
            resource_type: Some("Case".to_string()),
            success,
            error,
            metadata: serde_json::json!({}),
        })
    }

    /// Log authorization failure
    pub fn log_authorization_failure(
        &self,
        principal: &Principal,
        action: &str,
        resource: &str,
    ) -> WorkflowResult<()> {
        self.log(AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level: AuditLevel::Warning,
            principal: Some(principal.clone()),
            action: format!("authorize:{}", action),
            resource: Some(resource.to_string()),
            resource_type: None,
            success: false,
            error: Some("Authorization denied".to_string()),
            metadata: serde_json::json!({}),
        })
    }

    /// Get audit events
    pub fn get_events(&self, limit: Option<usize>) -> WorkflowResult<Vec<AuditEvent>> {
        let events = self.events.lock().map_err(|e| {
            crate::error::WorkflowError::Internal(format!("Failed to acquire audit lock: {}", e))
        })?;

        let mut result: Vec<AuditEvent> = events.iter().cloned().collect();
        result.reverse(); // Most recent first

        if let Some(limit) = limit {
            result.truncate(limit);
        }

        Ok(result)
    }

    /// Get events for a resource
    pub fn get_events_for_resource(&self, resource: &str) -> WorkflowResult<Vec<AuditEvent>> {
        let events = self.events.lock().map_err(|e| {
            crate::error::WorkflowError::Internal(format!("Failed to acquire audit lock: {}", e))
        })?;

        Ok(events
            .iter()
            .filter(|e| e.resource.as_ref().map(|r| r == resource).unwrap_or(false))
            .cloned()
            .collect())
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(10000, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_logger() {
        let logger = AuditLogger::default();

        logger
            .log_workflow_registration(None, crate::parser::WorkflowSpecId::new(), true, None)
            .unwrap();

        let events = logger.get_events(Some(10)).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].action, "register_workflow");
    }
}
