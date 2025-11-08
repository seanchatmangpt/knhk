//! REST API server

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::api::models::*;
use crate::executor::WorkflowEngine;
use crate::error::WorkflowResult;

/// REST API server
pub struct RestApiServer {
    engine: Arc<WorkflowEngine>,
}

impl RestApiServer {
    /// Create a new REST API server
    pub fn new(engine: Arc<WorkflowEngine>) -> Self {
        Self { engine }
    }

    /// Create the router
    pub fn router(&self) -> Router {
        Router::new()
            .route("/workflows", post(Self::register_workflow))
            .route("/workflows/:id", get(Self::get_workflow))
            .route("/cases", post(Self::create_case))
            .route("/cases/:id", get(Self::get_case))
            .route("/cases/:id/cancel", post(Self::cancel_case))
            .route("/cases/:id/history", get(Self::get_case_history))
            .with_state(Arc::clone(&self.engine))
    }

    /// Register a workflow
    async fn register_workflow(
        State(engine): State<Arc<WorkflowEngine>>,
        Json(request): Json<RegisterWorkflowRequest>,
    ) -> Result<Json<RegisterWorkflowResponse>, StatusCode> {
        engine
            .register_workflow(request.spec)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        Ok(Json(RegisterWorkflowResponse {
            spec_id: request.spec.id,
        }))
    }

    /// Get workflow specification
    async fn get_workflow(
        State(engine): State<Arc<WorkflowEngine>>,
        Path(id): Path<String>,
    ) -> Result<Json<crate::parser::WorkflowSpec>, StatusCode> {
        let spec_id = crate::parser::WorkflowSpecId::parse_str(&id)
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        
        let spec = engine
            .get_workflow(spec_id)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;
        
        Ok(Json(spec))
    }

    /// Create a case
    async fn create_case(
        State(engine): State<Arc<WorkflowEngine>>,
        Json(request): Json<CreateCaseRequest>,
    ) -> Result<Json<CreateCaseResponse>, StatusCode> {
        let case_id = engine
            .create_case(request.spec_id, request.data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        Ok(Json(CreateCaseResponse { case_id }))
    }

    /// Get case status
    async fn get_case(
        State(engine): State<Arc<WorkflowEngine>>,
        Path(id): Path<String>,
    ) -> Result<Json<CaseStatusResponse>, StatusCode> {
        let case_id = crate::case::CaseId::parse_str(&id)
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        
        let case = engine
            .get_case(case_id)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;
        
        Ok(Json(CaseStatusResponse { case }))
    }

    /// Cancel a case
    async fn cancel_case(
        State(engine): State<Arc<WorkflowEngine>>,
        Path(id): Path<String>,
    ) -> Result<StatusCode, StatusCode> {
        let case_id = crate::case::CaseId::parse_str(&id)
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        
        engine
            .cancel_case(case_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        Ok(StatusCode::OK)
    }

    /// Get case history
    async fn get_case_history(
        State(_engine): State<Arc<WorkflowEngine>>,
        Path(_id): Path<String>,
    ) -> Result<Json<CaseHistoryResponse>, StatusCode> {
        // TODO: Implement case history
        Ok(Json(CaseHistoryResponse { entries: vec![] }))
    }
}

