//! gRPC API service for workflow engine
//!
//! Provides gRPC interface for workflow management operations.

use crate::api::models::{
    errors::ApiError,
    requests::{
        CancelCaseRequest, CreateCaseRequest, ExecuteCaseRequest, GetCaseHistoryRequest,
        GetCaseRequest, GetWorkflowRequest, RegisterWorkflowRequest, StartCaseRequest,
    },
};
use crate::api::service::{CaseService, WorkflowService};
use crate::api::transport::GrpcAdapter;
use crate::executor::WorkflowEngine;
use crate::{CaseId, WorkflowSpecId};
use std::sync::Arc;
use tonic::{Request, Response, Status};

// Include generated proto code
/// Generated gRPC protocol buffer definitions
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

    /// Register a workflow (gRPC method) - uses service layer
    pub async fn register_workflow(
        &self,
        spec: crate::parser::WorkflowSpec,
    ) -> Result<WorkflowSpecId, ApiError> {
        let service = WorkflowService::new(self.engine.clone());
        let request = RegisterWorkflowRequest { spec };
        let response = service.register_workflow(request).await?;
        Ok(response.spec_id)
    }

    /// Get workflow specification (gRPC method) - uses service layer
    pub async fn get_workflow(
        &self,
        spec_id: WorkflowSpecId,
    ) -> Result<crate::parser::WorkflowSpec, ApiError> {
        let service = WorkflowService::new(self.engine.clone());
        let request = GetWorkflowRequest { spec_id };
        let response = service.get_workflow(request).await?;
        Ok(response.spec)
    }

    /// Create a case (gRPC method) - uses service layer
    pub async fn create_case(
        &self,
        spec_id: WorkflowSpecId,
        data: serde_json::Value,
    ) -> Result<CaseId, ApiError> {
        let service = CaseService::new(self.engine.clone());
        let request = CreateCaseRequest { spec_id, data };
        let response = service.create_case(request).await?;
        Ok(response.case_id)
    }

    /// Get case status (gRPC method) - uses service layer
    pub async fn get_case(&self, case_id: CaseId) -> Result<crate::case::Case, ApiError> {
        let service = CaseService::new(self.engine.clone());
        let request = GetCaseRequest { case_id };
        let response = service.get_case(request).await?;
        Ok(response.case)
    }

    /// Cancel a case (gRPC method) - uses service layer
    pub async fn cancel_case(&self, case_id: CaseId) -> Result<(), ApiError> {
        let service = CaseService::new(self.engine.clone());
        let request = CancelCaseRequest { case_id };
        service.cancel_case(request).await?;
        Ok(())
    }

    /// Execute a case (gRPC method) - uses service layer
    pub async fn execute_case(&self, case_id: CaseId) -> Result<(), ApiError> {
        let service = CaseService::new(self.engine.clone());
        let request = ExecuteCaseRequest { case_id };
        service.execute_case(request).await?;
        Ok(())
    }

    /// Start a case (gRPC method) - uses service layer
    pub async fn start_case(&self, case_id: CaseId) -> Result<(), ApiError> {
        let service = CaseService::new(self.engine.clone());
        let request = StartCaseRequest { case_id };
        service.start_case(request).await?;
        Ok(())
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

        match self.register_workflow(workflow_spec).await {
            Ok(spec_id) => Ok(Response::new(proto::RegisterWorkflowResponse {
                spec_id: spec_id.to_string(),
            })),
            Err(e) => Err(GrpcAdapter::error_to_status(e)),
        }
    }

    async fn get_workflow(
        &self,
        request: Request<proto::GetWorkflowRequest>,
    ) -> Result<Response<proto::GetWorkflowResponse>, Status> {
        let req = request.into_inner();
        let spec_id = WorkflowSpecId::parse_str(&req.spec_id)
            .map_err(|_| Status::invalid_argument("Invalid spec_id format"))?;

        match self.get_workflow(spec_id).await {
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
            Err(e) => Err(GrpcAdapter::error_to_status(e)),
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

        match self.create_case(spec_id, data).await {
            Ok(case_id) => Ok(Response::new(proto::CreateCaseResponse {
                case_id: case_id.to_string(),
            })),
            Err(e) => Err(GrpcAdapter::error_to_status(e)),
        }
    }

    async fn get_case(
        &self,
        request: Request<proto::GetCaseRequest>,
    ) -> Result<Response<proto::GetCaseResponse>, Status> {
        let req = request.into_inner();
        let case_id = CaseId::parse_str(&req.case_id)
            .map_err(|_| Status::invalid_argument("Invalid case_id format"))?;

        match self.get_case(case_id).await {
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
            Err(e) => Err(GrpcAdapter::error_to_status(e)),
        }
    }

    async fn get_case_history(
        &self,
        request: Request<proto::GetCaseHistoryRequest>,
    ) -> Result<Response<proto::GetCaseHistoryResponse>, Status> {
        let req = request.into_inner();
        let case_id = CaseId::parse_str(&req.case_id)
            .map_err(|_| Status::invalid_argument("Invalid case_id format"))?;

        // Use service layer
        let service = CaseService::new(self.engine.clone());
        let request = GetCaseHistoryRequest { case_id };
        let response = match service.get_case_history(request).await {
            Ok(response) => response,
            Err(e) => return Err(GrpcAdapter::error_to_status(e)),
        };

        // Transform to proto CaseHistoryEntry
        let entries: Vec<proto::CaseHistoryEntry> = response
            .entries
            .into_iter()
            .map(|entry| proto::CaseHistoryEntry {
                timestamp: entry.timestamp.timestamp(),
                event_type: entry.event_type,
                data: entry.data.to_string(),
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

        match self.start_case(case_id).await {
            Ok(_) => Ok(Response::new(proto::StartCaseResponse { success: true })),
            Err(e) => Err(GrpcAdapter::error_to_status(e)),
        }
    }

    async fn execute_case(
        &self,
        request: Request<proto::ExecuteCaseRequest>,
    ) -> Result<Response<proto::ExecuteCaseResponse>, Status> {
        let req = request.into_inner();
        let case_id = CaseId::parse_str(&req.case_id)
            .map_err(|_| Status::invalid_argument("Invalid case_id format"))?;

        match self.execute_case(case_id).await {
            Ok(_) => Ok(Response::new(proto::ExecuteCaseResponse { success: true })),
            Err(e) => Err(GrpcAdapter::error_to_status(e)),
        }
    }

    async fn cancel_case(
        &self,
        request: Request<proto::CancelCaseRequest>,
    ) -> Result<Response<proto::CancelCaseResponse>, Status> {
        let req = request.into_inner();
        let case_id = CaseId::parse_str(&req.case_id)
            .map_err(|_| Status::invalid_argument("Invalid case_id format"))?;

        match self.cancel_case(case_id).await {
            Ok(_) => Ok(Response::new(proto::CancelCaseResponse { success: true })),
            Err(e) => Err(GrpcAdapter::error_to_status(e)),
        }
    }
}
