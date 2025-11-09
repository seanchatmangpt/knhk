//! Workflow service
//!
//! Service layer for workflow operations.

use crate::api::models::{
    errors::ApiError,
    requests::{
        DeleteWorkflowRequest, GetWorkflowRequest, ListWorkflowsRequest, RegisterWorkflowRequest,
    },
    responses::{
        DeleteWorkflowResponse, GetWorkflowResponse, ListWorkflowsResponse,
        RegisterWorkflowResponse,
    },
    ApiResult,
};
use crate::executor::WorkflowEngine;
use std::sync::Arc;

/// Workflow service for workflow management operations
pub struct WorkflowService {
    engine: Arc<WorkflowEngine>,
}

impl WorkflowService {
    /// Create a new workflow service
    pub fn new(engine: Arc<WorkflowEngine>) -> Self {
        Self { engine }
    }

    /// Register a workflow specification
    pub async fn register_workflow(
        &self,
        request: RegisterWorkflowRequest,
    ) -> ApiResult<RegisterWorkflowResponse> {
        let spec_id = request.spec.id;
        self.engine
            .register_workflow(request.spec)
            .await
            .map_err(ApiError::from)?;

        Ok(RegisterWorkflowResponse { spec_id })
    }

    /// Get workflow specification
    pub async fn get_workflow(
        &self,
        request: GetWorkflowRequest,
    ) -> ApiResult<GetWorkflowResponse> {
        let spec = self
            .engine
            .get_workflow(request.spec_id)
            .await
            .map_err(ApiError::from)?;

        Ok(GetWorkflowResponse { spec })
    }

    /// List all registered workflow specifications
    pub async fn list_workflows(
        &self,
        _request: ListWorkflowsRequest,
    ) -> ApiResult<ListWorkflowsResponse> {
        let workflows = self.engine.list_workflows().await.map_err(ApiError::from)?;

        Ok(ListWorkflowsResponse { workflows })
    }

    /// Delete a workflow specification
    pub async fn delete_workflow(
        &self,
        request: DeleteWorkflowRequest,
    ) -> ApiResult<DeleteWorkflowResponse> {
        // Verify workflow exists
        self.engine
            .get_workflow(request.spec_id)
            .await
            .map_err(ApiError::from)?;

        // Remove from in-memory specs
        self.engine.specs().remove(&request.spec_id);

        // Remove from state store
        let store = self.engine.state_store().read().await;
        store.delete_spec(&request.spec_id).map_err(|e| {
            ApiError::new(
                "INTERNAL_ERROR",
                format!("Failed to delete workflow: {}", e),
            )
        })?;

        Ok(DeleteWorkflowResponse { success: true })
    }
}
