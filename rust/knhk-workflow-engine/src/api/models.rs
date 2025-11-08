//! API models

use serde::{Deserialize, Serialize};

use crate::case::{Case, CaseId};
use crate::parser::{WorkflowSpec, WorkflowSpecId};

/// Request to register a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterWorkflowRequest {
    /// Workflow specification
    pub spec: WorkflowSpec,
}

/// Response from workflow registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterWorkflowResponse {
    /// Workflow specification ID
    pub spec_id: WorkflowSpecId,
}

/// Request to create a case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCaseRequest {
    /// Workflow specification ID
    pub spec_id: WorkflowSpecId,
    /// Case data (input variables)
    pub data: serde_json::Value,
}

/// Response from case creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCaseResponse {
    /// Case ID
    pub case_id: CaseId,
}

/// Case status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseStatusResponse {
    /// Case
    pub case: Case,
}

/// Case history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseHistoryEntry {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event type
    pub event_type: String,
    /// Event data
    pub data: serde_json::Value,
}

/// Case history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseHistoryResponse {
    /// History entries
    pub entries: Vec<CaseHistoryEntry>,
}
