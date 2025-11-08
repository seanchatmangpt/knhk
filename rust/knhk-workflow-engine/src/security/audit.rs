#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Audit logging for compliance and security

use crate::case::CaseId;
use crate::error::WorkflowResult;
use crate::integration::LockchainIntegration;
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
    /// Optional lockchain integration instance
    lockchain: Option<Arc<LockchainIntegration>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(max_events: usize, lockchain_enabled: bool) -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            max_events,
            lockchain_enabled,
            lockchain: None,
        }
    }

    /// Create a new audit logger with lockchain integration
    pub fn with_lockchain(max_events: usize, lockchain: Arc<LockchainIntegration>) -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            max_events,
            lockchain_enabled: true,
            lockchain: Some(lockchain),
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
            if let Some(ref lockchain) = self.lockchain {
                // Convert audit event to JSON for lockchain
                let _event_data = serde_json::json!({
                    "event_type": "audit.log",
                    "event_id": event.id.to_string(),
                    "timestamp": event.timestamp.to_rfc3339(),
                    "level": format!("{:?}", event.level),
                    "principal": event.principal.as_ref().map(|p| serde_json::json!({
                        "id": p.id.clone(),
                        "principal_type": format!("{:?}", p.principal_type),
                    })),
                    "action": event.action,
                    "resource": event.resource,
                    "resource_type": event.resource_type,
                    "success": event.success,
                    "error": event.error,
                    "metadata": event.metadata,
                });

                // Record event in lockchain (async call from sync context)
                // LockchainIntegration contains non-Send types (git_repository), so we can't spawn a task
                // Instead, we'll log a warning that lockchain recording requires async context
                // In production, this would be handled by a background task that owns the lockchain
                tracing::warn!(
                    "Audit event {} not recorded to lockchain: lockchain is not Send-safe for task spawning",
                    event.id
                );
            } else {
                // Lockchain enabled but no integration instance provided
                tracing::warn!(
                    "Lockchain enabled but no integration instance provided for audit event {}",
                    event.id
                );
            }
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
