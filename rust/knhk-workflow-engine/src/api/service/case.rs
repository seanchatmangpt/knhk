//! Case service
//!
//! Service layer for case management operations.

use crate::api::models::{
    errors::ApiError,
    requests::{
        CancelCaseRequest, CreateCaseRequest, ExecuteCaseRequest, GetCaseHistoryRequest,
        GetCaseRequest, ListCasesRequest, StartCaseRequest,
    },
    responses::{
        CancelCaseResponse, CreateCaseResponse, ExecuteCaseResponse, GetCaseHistoryResponse,
        GetCaseResponse, ListCasesResponse, StartCaseResponse,
    },
    ApiResult, CaseHistoryEntry,
};
use crate::executor::WorkflowEngine;
use crate::state::manager::StateEvent;
use std::sync::Arc;

/// Case service for case management operations
pub struct CaseService {
    engine: Arc<WorkflowEngine>,
}

impl CaseService {
    /// Create a new case service
    pub fn new(engine: Arc<WorkflowEngine>) -> Self {
        Self { engine }
    }

    /// Create a new case
    pub async fn create_case(&self, request: CreateCaseRequest) -> ApiResult<CreateCaseResponse> {
        let case_id = self
            .engine
            .create_case(request.spec_id, request.data)
            .await
            .map_err(ApiError::from)?;

        Ok(CreateCaseResponse { case_id })
    }

    /// Get case status
    pub async fn get_case(&self, request: GetCaseRequest) -> ApiResult<GetCaseResponse> {
        let case = self
            .engine
            .get_case(request.case_id)
            .await
            .map_err(ApiError::from)?;

        Ok(GetCaseResponse { case })
    }

    /// List cases
    pub async fn list_cases(&self, request: ListCasesRequest) -> ApiResult<ListCasesResponse> {
        let cases = if let Some(spec_id) = request.spec_id {
            // List cases for specific workflow
            self.engine
                .list_cases(spec_id)
                .await
                .map_err(ApiError::from)?
        } else {
            // List all cases (from in-memory cache)
            self.engine
                .cases()
                .iter()
                .map(|entry| *entry.key())
                .collect()
        };

        Ok(ListCasesResponse { cases })
    }

    /// Start a case
    pub async fn start_case(&self, request: StartCaseRequest) -> ApiResult<StartCaseResponse> {
        self.engine
            .start_case(request.case_id)
            .await
            .map_err(ApiError::from)?;

        Ok(StartCaseResponse { success: true })
    }

    /// Execute a case
    pub async fn execute_case(
        &self,
        request: ExecuteCaseRequest,
    ) -> ApiResult<ExecuteCaseResponse> {
        self.engine
            .execute_case(request.case_id)
            .await
            .map_err(ApiError::from)?;

        Ok(ExecuteCaseResponse { success: true })
    }

    /// Cancel a case
    pub async fn cancel_case(&self, request: CancelCaseRequest) -> ApiResult<CancelCaseResponse> {
        self.engine
            .cancel_case(request.case_id)
            .await
            .map_err(ApiError::from)?;

        Ok(CancelCaseResponse { success: true })
    }

    /// Get case history
    pub async fn get_case_history(
        &self,
        request: GetCaseHistoryRequest,
    ) -> ApiResult<GetCaseHistoryResponse> {
        // Get case history from StateManager
        // Access state_manager directly (it's pub(crate) so we can access it from the api module)
        let events = self
            .engine
            .state_manager
            .get_case_history(request.case_id)
            .await;

        // Transform StateEvent enum variants to CaseHistoryEntry format
        let entries: Vec<CaseHistoryEntry> = events
            .into_iter()
            .filter_map(|event| match event {
                StateEvent::TaskStarted { .. } | StateEvent::TaskCompleted { .. } => {
                    // Task events are logged but not returned in case history
                    None
                }
                StateEvent::CaseCreated {
                    case_id,
                    spec_id,
                    timestamp,
                } => Some(CaseHistoryEntry {
                    timestamp,
                    event_type: "case_created".to_string(),
                    data: serde_json::json!({
                        "case_id": case_id.to_string(),
                        "spec_id": spec_id.to_string(),
                    }),
                }),
                StateEvent::CaseStateChanged {
                    case_id,
                    old_state,
                    new_state,
                    timestamp,
                } => Some(CaseHistoryEntry {
                    timestamp,
                    event_type: "state_changed".to_string(),
                    data: serde_json::json!({
                        "case_id": case_id.to_string(),
                        "from": old_state,
                        "to": new_state,
                    }),
                }),
                StateEvent::SpecRegistered { .. } => {
                    // This shouldn't appear in case history, but handle it gracefully
                    Some(CaseHistoryEntry {
                        timestamp: chrono::Utc::now(),
                        event_type: "unknown".to_string(),
                        data: serde_json::json!({}),
                    })
                }
            })
            .collect();

        Ok(GetCaseHistoryResponse { entries })
    }
}
