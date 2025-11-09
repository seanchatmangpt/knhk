//! Unified API response models
//!
//! Response models used by all transport layers (REST, gRPC, CLI).

use crate::api::models::CaseHistoryEntry;
use crate::case::{Case, CaseId};
use crate::parser::{WorkflowSpec, WorkflowSpecId};
use crate::patterns::PatternId;
use serde::{Deserialize, Serialize};

/// Register workflow response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterWorkflowResponse {
    /// Workflow specification ID
    pub spec_id: WorkflowSpecId,
}

/// Get workflow response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWorkflowResponse {
    /// Workflow specification
    pub spec: WorkflowSpec,
}

/// List workflows response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWorkflowsResponse {
    /// Workflow specification IDs
    pub workflows: Vec<WorkflowSpecId>,
}

/// Delete workflow response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteWorkflowResponse {
    /// Success indicator
    pub success: bool,
}

/// Create case response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCaseResponse {
    /// Case ID
    pub case_id: CaseId,
}

/// Get case response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCaseResponse {
    /// Case
    pub case: Case,
}

/// List cases response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCasesResponse {
    /// Cases
    pub cases: Vec<CaseId>,
}

/// Start case response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartCaseResponse {
    /// Success indicator
    pub success: bool,
}

/// Execute case response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteCaseResponse {
    /// Success indicator
    pub success: bool,
}

/// Cancel case response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelCaseResponse {
    /// Success indicator
    pub success: bool,
}

/// Get case history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCaseHistoryResponse {
    /// History entries
    pub entries: Vec<CaseHistoryEntry>,
}

/// List patterns response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPatternsResponse {
    /// Pattern IDs
    pub patterns: Vec<PatternId>,
}

/// Get pattern response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPatternResponse {
    /// Pattern ID
    pub pattern_id: PatternId,
    /// Pattern name
    pub name: String,
}

/// Execute pattern response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutePatternResponse {
    /// Pattern ID
    pub pattern_id: PatternId,
    /// Success indicator
    pub success: bool,
    /// Result variables
    pub variables: std::collections::HashMap<String, String>,
}
