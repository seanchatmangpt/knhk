//! Unified API request models
//!
//! Request models used by all transport layers (REST, gRPC, CLI).

use crate::case::CaseId;
use crate::parser::{WorkflowSpec, WorkflowSpecId};
use crate::patterns::PatternId;
use serde::{Deserialize, Serialize};

/// Register workflow request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterWorkflowRequest {
    /// Workflow specification
    pub spec: WorkflowSpec,
}

/// Get workflow request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWorkflowRequest {
    /// Workflow specification ID
    pub spec_id: WorkflowSpecId,
}

/// List workflows request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWorkflowsRequest {
    // Empty for now, can add filters later
}

/// Delete workflow request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteWorkflowRequest {
    /// Workflow specification ID
    pub spec_id: WorkflowSpecId,
}

/// Create case request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCaseRequest {
    /// Workflow specification ID
    pub spec_id: WorkflowSpecId,
    /// Case data (input variables)
    pub data: serde_json::Value,
}

/// Get case request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCaseRequest {
    /// Case ID
    pub case_id: CaseId,
}

/// List cases request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCasesRequest {
    /// Optional workflow specification ID filter
    pub spec_id: Option<WorkflowSpecId>,
}

/// Start case request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartCaseRequest {
    /// Case ID
    pub case_id: CaseId,
}

/// Execute case request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteCaseRequest {
    /// Case ID
    pub case_id: CaseId,
}

/// Cancel case request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelCaseRequest {
    /// Case ID
    pub case_id: CaseId,
}

/// Get case history request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCaseHistoryRequest {
    /// Case ID
    pub case_id: CaseId,
}

/// List patterns request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPatternsRequest {
    // Empty for now, can add filters later
}

/// Get pattern request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPatternRequest {
    /// Pattern ID
    pub pattern_id: PatternId,
}

/// Execute pattern request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutePatternRequest {
    /// Pattern ID
    pub pattern_id: PatternId,
    /// Execution context variables
    pub variables: std::collections::HashMap<String, String>,
}
