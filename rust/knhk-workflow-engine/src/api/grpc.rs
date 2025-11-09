//! gRPC API service for workflow engine
//!
//! Provides gRPC interface for workflow management operations.

use crate::error::WorkflowResult;
use crate::executor::WorkflowEngine;
use crate::{CaseId, WorkflowSpecId};
use std::sync::Arc;
use tonic::{Request, Response, Status};

// Include generated proto code
pub mod proto {
    tonic::include_proto!("knhk.workflow_engine.v1");
}

pub use proto::workflow_engine_service_server::{
    WorkflowEngineService, WorkflowEngineServiceServer,
};

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

#[tonic::async_trait]
impl WorkflowEngineService for GrpcService {
    async fn register_workflow(
        &self,
        request: Request<proto::RegisterWorkflowRequest>,
    ) -> Result<Response<proto::RegisterWorkflowResponse>, Status> {
        let req = request.into_inner();
        let spec = req
            .spec
            .ok_or_else(|| Status::invalid_argument("WorkflowSpec is required"))?;

        // Parse spec from proto (simplified - in production would fully deserialize)
        let spec_id_str = spec.id.clone();
        let spec_name = spec.name.clone();
        let spec_data = spec.spec_data.clone();
        let spec_id = WorkflowSpecId::parse_str(&spec_id_str)
            .map_err(|_| Status::invalid_argument("Invalid spec_id format"))?;

        // Try to deserialize WorkflowSpec from spec_data JSON, or create minimal spec
        let workflow_spec: crate::parser::WorkflowSpec = if !spec_data.is_empty() {
            serde_json::from_str(&spec_data)
                .map_err(|e| Status::invalid_argument(format!("Invalid spec_data JSON: {}", e)))?
        } else {
            // Create minimal spec if spec_data is empty
            crate::parser::WorkflowSpec {
                id: spec_id,
                name: spec_name,
                start_condition: None,
                end_condition: None,
                tasks: std::collections::HashMap::new(),
                conditions: std::collections::HashMap::new(),
                flows: Vec::new(),
                source_turtle: None,
            }
        };

        match self.engine.register_workflow(workflow_spec).await {
            Ok(_) => Ok(Response::new(proto::RegisterWorkflowResponse {
                spec_id: spec_id_str,
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_workflow(
        &self,
        request: Request<proto::GetWorkflowRequest>,
    ) -> Result<Response<proto::GetWorkflowResponse>, Status> {
        let req = request.into_inner();
        let spec_id = WorkflowSpecId::parse_str(&req.spec_id)
            .map_err(|_| Status::invalid_argument("Invalid spec_id format"))?;

        match self.engine.get_workflow(spec_id).await {
            Ok(spec) => {
                let spec_id = spec.id.to_string();
                let spec_name = spec.name.clone();
                let spec_data = serde_json::to_string(&spec).unwrap_or_default();
                Ok(Response::new(proto::GetWorkflowResponse {
                    spec: Some(proto::WorkflowSpec {
                        id: spec_id,
                        name: spec_name,
                        spec_data,
                    }),
                }))
            }
            Err(e) => Err(Status::not_found(e.to_string())),
        }
    }

    async fn create_case(
        &self,
        request: Request<proto::CreateCaseRequest>,
    ) -> Result<Response<proto::CreateCaseResponse>, Status> {
        let req = request.into_inner();
        let spec_id = WorkflowSpecId::parse_str(&req.spec_id)
            .map_err(|_| Status::invalid_argument("Invalid spec_id format"))?;

        let data: serde_json::Value = serde_json::from_str(&req.data)
            .map_err(|e| Status::invalid_argument(format!("Invalid JSON data: {}", e)))?;

        match self.engine.create_case(spec_id, data).await {
            Ok(case_id) => Ok(Response::new(proto::CreateCaseResponse {
                case_id: case_id.to_string(),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_case(
        &self,
        request: Request<proto::GetCaseRequest>,
    ) -> Result<Response<proto::GetCaseResponse>, Status> {
        let req = request.into_inner();
        let case_id = CaseId::parse_str(&req.case_id)
            .map_err(|_| Status::invalid_argument("Invalid case_id format"))?;

        match self.engine.get_case(case_id).await {
            Ok(case) => Ok(Response::new(proto::GetCaseResponse {
                case: Some(proto::Case {
                    id: case.id.to_string(),
                    spec_id: case.spec_id.to_string(),
                    state: format!("{:?}", case.state),
                    data: serde_json::to_string(&case.data).unwrap_or_default(),
                    created_at: case.created_at.timestamp(),
                    updated_at: case.created_at.timestamp(), // Use created_at as updated_at fallback
                }),
            })),
            Err(e) => Err(Status::not_found(e.to_string())),
        }
    }

    async fn get_case_history(
        &self,
        request: Request<proto::GetCaseHistoryRequest>,
    ) -> Result<Response<proto::GetCaseHistoryResponse>, Status> {
        let req = request.into_inner();
        let case_id = CaseId::parse_str(&req.case_id)
            .map_err(|_| Status::invalid_argument("Invalid case_id format"))?;

        // Get case history from StateManager
        let events = self.engine.state_manager.get_case_history(case_id).await;

        // Transform StateEvent to proto CaseHistoryEntry
        use crate::state::manager::StateEvent;
        let entries: Vec<proto::CaseHistoryEntry> = events
            .into_iter()
            .map(|event| match event {
                StateEvent::CaseCreated {
                    case_id,
                    spec_id,
                    timestamp,
                } => proto::CaseHistoryEntry {
                    timestamp: timestamp.timestamp(),
                    event_type: "case_created".to_string(),
                    data: serde_json::json!({
                        "case_id": case_id.to_string(),
                        "spec_id": spec_id.to_string(),
                    })
                    .to_string(),
                },
                StateEvent::CaseStateChanged {
                    case_id,
                    old_state,
                    new_state,
                    timestamp,
                } => proto::CaseHistoryEntry {
                    timestamp: timestamp.timestamp(),
                    event_type: "state_changed".to_string(),
                    data: serde_json::json!({
                        "case_id": case_id.to_string(),
                        "from": old_state,
                        "to": new_state,
                    })
                    .to_string(),
                },
                StateEvent::SpecRegistered { .. } => proto::CaseHistoryEntry {
                    timestamp: chrono::Utc::now().timestamp(),
                    event_type: "unknown".to_string(),
                    data: "{}".to_string(),
                },
            })
            .collect();

        Ok(Response::new(proto::GetCaseHistoryResponse { entries }))
    }

    async fn start_case(
        &self,
        request: Request<proto::StartCaseRequest>,
    ) -> Result<Response<proto::StartCaseResponse>, Status> {
        let req = request.into_inner();
        let case_id = CaseId::parse_str(&req.case_id)
            .map_err(|_| Status::invalid_argument("Invalid case_id format"))?;

        match self.engine.start_case(case_id).await {
            Ok(_) => Ok(Response::new(proto::StartCaseResponse { success: true })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn execute_case(
        &self,
        request: Request<proto::ExecuteCaseRequest>,
    ) -> Result<Response<proto::ExecuteCaseResponse>, Status> {
        let req = request.into_inner();
        let case_id = CaseId::parse_str(&req.case_id)
            .map_err(|_| Status::invalid_argument("Invalid case_id format"))?;

        match self.engine.execute_case(case_id).await {
            Ok(_) => Ok(Response::new(proto::ExecuteCaseResponse { success: true })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn cancel_case(
        &self,
        request: Request<proto::CancelCaseRequest>,
    ) -> Result<Response<proto::CancelCaseResponse>, Status> {
        let req = request.into_inner();
        let case_id = CaseId::parse_str(&req.case_id)
            .map_err(|_| Status::invalid_argument("Invalid case_id format"))?;

        match self.engine.cancel_case(case_id).await {
            Ok(_) => Ok(Response::new(proto::CancelCaseResponse { success: true })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
