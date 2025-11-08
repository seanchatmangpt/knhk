//! gRPC API service for workflow engine
//!
//! Provides gRPC interface for workflow management operations.
//! Note: Full implementation requires proto definitions and tonic-build integration.

use crate::error::WorkflowResult;
use crate::executor::WorkflowEngine;
use crate::{CaseId, WorkflowSpecId};
use std::sync::Arc;

/// gRPC service for workflow engine
pub struct GrpcService {
    engine: Arc<WorkflowEngine>,
}

impl GrpcService {
    /// Create a new gRPC service
    pub fn new(engine: Arc<WorkflowEngine>) -> Self {
        Self { engine }
    }

    /// Register a workflow (gRPC method)
    pub async fn register_workflow(
        &self,
        spec: crate::parser::WorkflowSpec,
    ) -> WorkflowResult<WorkflowSpecId> {
        let spec_id = spec.id;
        self.engine.register_workflow(spec).await?;
        Ok(spec_id)
    }

    /// Get workflow specification (gRPC method)
    pub async fn get_workflow(
        &self,
        spec_id: WorkflowSpecId,
    ) -> WorkflowResult<crate::parser::WorkflowSpec> {
        self.engine.get_workflow(spec_id).await
    }

    /// Create a case (gRPC method)
    pub async fn create_case(
        &self,
        spec_id: WorkflowSpecId,
        data: serde_json::Value,
    ) -> WorkflowResult<CaseId> {
        self.engine.create_case(spec_id, data).await
    }

    /// Get case status (gRPC method)
    pub async fn get_case(&self, case_id: CaseId) -> WorkflowResult<crate::case::Case> {
        self.engine.get_case(case_id).await
    }

    /// Cancel a case (gRPC method)
    pub async fn cancel_case(&self, case_id: CaseId) -> WorkflowResult<()> {
        self.engine.cancel_case(case_id).await
    }

    /// Execute a case (gRPC method)
    pub async fn execute_case(&self, case_id: CaseId) -> WorkflowResult<()> {
        self.engine.execute_case(case_id).await
    }

    /// Start a case (gRPC method)
    pub async fn start_case(&self, case_id: CaseId) -> WorkflowResult<()> {
        self.engine.start_case(case_id).await
    }
}

// FUTURE: Implement tonic service trait when proto definitions are available
// Example structure:
//
// #[tonic::async_trait]
// impl workflow_engine::WorkflowEngineService for GrpcService {
//     async fn register_workflow(
//         &self,
//         request: Request<RegisterWorkflowRequest>,
//     ) -> Result<Response<RegisterWorkflowResponse>, Status> {
//         let req = request.into_inner();
//         match self.register_workflow(req.spec).await {
//             Ok(spec_id) => Ok(Response::new(RegisterWorkflowResponse { spec_id: spec_id.to_string() })),
//             Err(e) => Err(Status::internal(e.to_string())),
//         }
//     }
//     // ... other methods
// }
