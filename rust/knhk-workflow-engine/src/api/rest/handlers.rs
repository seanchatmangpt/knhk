//! REST API route handlers

use crate::api::models::requests::{
    CancelCaseRequest, CreateCaseRequest, ExecuteCaseRequest, ExecutePatternRequest,
    GetCaseHistoryRequest, GetCaseRequest, GetPatternRequest, GetWorkflowRequest, ListCasesRequest,
    ListPatternsRequest, ListWorkflowsRequest, RegisterWorkflowRequest, StartCaseRequest,
};
use crate::api::service::{CaseService, PatternService, WorkflowService};
use crate::api::transport::RestAdapter;
use crate::case::CaseId;
use crate::executor::WorkflowEngine;
use crate::observability::HealthStatus;
use crate::parser::WorkflowSpecId;
use crate::patterns::PatternId;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::collections::HashMap;
use std::sync::Arc;

/// Register a workflow
pub async fn register_workflow(
    State(engine): State<Arc<WorkflowEngine>>,
    Json(request): Json<RegisterWorkflowRequest>,
) -> axum::response::Response {
    let service = WorkflowService::new(engine);
    let result = service.register_workflow(request).await;
    RestAdapter::result_to_response(result)
}

/// Get workflow specification
pub async fn get_workflow(
    State(engine): State<Arc<WorkflowEngine>>,
    Path(id): Path<String>,
) -> axum::response::Response {
    let spec_id = match WorkflowSpecId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return RestAdapter::error_to_response(crate::api::models::errors::ApiError::new(
                "BAD_REQUEST",
                "Invalid spec_id format",
            ));
        }
    };

    let service = WorkflowService::new(engine);
    let request = GetWorkflowRequest { spec_id };
    let result = service.get_workflow(request).await;
    RestAdapter::result_to_response(result)
}

/// Create a case
pub async fn create_case(
    State(engine): State<Arc<WorkflowEngine>>,
    Json(request): Json<CreateCaseRequest>,
) -> axum::response::Response {
    let service = CaseService::new(engine);
    let result = service.create_case(request).await;
    RestAdapter::result_to_response(result)
}

/// Get case status
pub async fn get_case(
    State(engine): State<Arc<WorkflowEngine>>,
    Path(id): Path<String>,
) -> axum::response::Response {
    let case_id = match CaseId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return RestAdapter::error_to_response(crate::api::models::errors::ApiError::new(
                "BAD_REQUEST",
                "Invalid case_id format",
            ));
        }
    };

    let service = CaseService::new(engine);
    let request = GetCaseRequest { case_id };
    let result = service.get_case(request).await;
    RestAdapter::result_to_response(result)
}

/// Cancel a case
pub async fn cancel_case(
    State(engine): State<Arc<WorkflowEngine>>,
    Path(id): Path<String>,
) -> axum::response::Response {
    let case_id = match CaseId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return RestAdapter::error_to_response(crate::api::models::errors::ApiError::new(
                "BAD_REQUEST",
                "Invalid case_id format",
            ));
        }
    };

    let service = CaseService::new(engine);
    let request = CancelCaseRequest { case_id };
    let result = service.cancel_case(request).await;
    match result {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response(),
        Err(error) => RestAdapter::error_to_response(error),
    }
}

/// Get case history
pub async fn get_case_history(
    State(engine): State<Arc<WorkflowEngine>>,
    Path(id): Path<String>,
) -> axum::response::Response {
    let case_id = match CaseId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return RestAdapter::error_to_response(crate::api::models::errors::ApiError::new(
                "BAD_REQUEST",
                "Invalid case_id format",
            ));
        }
    };

    let service = CaseService::new(engine);
    let request = GetCaseHistoryRequest { case_id };
    let result = service.get_case_history(request).await;
    RestAdapter::result_to_response(result)
}

/// Health check endpoint
pub async fn health(State(engine): State<Arc<WorkflowEngine>>) -> impl IntoResponse {
    // Check engine health
    let health_status = if engine.pattern_registry().list().is_empty() {
        HealthStatus::Degraded
    } else {
        // Check SLO compliance if Fortune 5 is enabled
        let slo_compliant = if let Some(fortune5) = engine.fortune5_integration() {
            fortune5.check_slo_compliance().await.unwrap_or(false)
        } else {
            true
        };

        if slo_compliant {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded
        }
    };

    let status_code = match health_status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    let response = serde_json::json!({
        "status": format!("{:?}", health_status),
        "service": "knhk-workflow-engine",
        "version": env!("CARGO_PKG_VERSION"),
    });

    (status_code, Json(response))
}

/// Readiness probe endpoint
pub async fn ready(State(engine): State<Arc<WorkflowEngine>>) -> impl IntoResponse {
    // Check if engine is ready to accept requests
    let ready = !engine.pattern_registry().list().is_empty();

    if ready {
        (StatusCode::OK, Json(serde_json::json!({ "ready": true })))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(
                serde_json::json!({ "ready": false, "reason": "Pattern registry not initialized" }),
            ),
        )
    }
}

/// Liveness probe endpoint
pub async fn live() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "alive": true })))
}

/// OpenAPI specification endpoint
pub async fn openapi() -> impl IntoResponse {
    // Generate OpenAPI spec dynamically from route handlers
    let openapi = serde_json::json!({
        "openapi": "3.0.0",
        "info": {
            "title": "KNHK Workflow Engine API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Enterprise workflow engine with full 43-pattern YAWL support"
        },
        "paths": {
            "/api/v1/workflows": {
                "post": {
                    "summary": "Register a workflow",
                    "operationId": "registerWorkflow"
                },
                "get": {
                    "summary": "List workflows",
                    "operationId": "listWorkflows"
                }
            },
            "/api/v1/workflows/{id}": {
                "get": {
                    "summary": "Get workflow",
                    "operationId": "getWorkflow"
                },
                "delete": {
                    "summary": "Delete workflow",
                    "operationId": "deleteWorkflow"
                }
            },
            "/api/v1/cases": {
                "post": {
                    "summary": "Create case",
                    "operationId": "createCase"
                },
                "get": {
                    "summary": "List cases",
                    "operationId": "listCases"
                }
            },
            "/api/v1/cases/{id}": {
                "get": {
                    "summary": "Get case",
                    "operationId": "getCase"
                }
            },
            "/api/v1/patterns": {
                "get": {
                    "summary": "List patterns",
                    "operationId": "listPatterns"
                }
            }
        }
    });

    Json(openapi)
}

/// Swagger UI endpoint
pub async fn swagger() -> impl IntoResponse {
    // Return Swagger UI HTML with dynamic OpenAPI spec URL
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>KNHK Workflow Engine API - Swagger UI</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@4.5.0/swagger-ui.css" />
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@4.5.0/swagger-ui-bundle.js"></script>
    <script>
        SwaggerUIBundle({
            url: '/openapi.json',
            dom_id: '#swagger-ui',
        });
    </script>
</body>
</html>
    "#;

    (
        StatusCode::OK,
        [("Content-Type", "text/html")],
        html.to_string(),
    )
}

/// Delete workflow
pub async fn delete_workflow(
    State(engine): State<Arc<WorkflowEngine>>,
    Path(id): Path<String>,
) -> axum::response::Response {
    let spec_id = match WorkflowSpecId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return RestAdapter::error_to_response(crate::api::models::errors::ApiError::new(
                "BAD_REQUEST",
                "Invalid spec_id format",
            ));
        }
    };

    let service = WorkflowService::new(engine);
    let request = crate::api::models::requests::DeleteWorkflowRequest { spec_id };
    let result = service.delete_workflow(request).await;
    match result {
        Ok(_) => (
            StatusCode::NO_CONTENT,
            Json(serde_json::json!({ "success": true })),
        )
            .into_response(),
        Err(error) => RestAdapter::error_to_response(error),
    }
}

/// List workflows
pub async fn list_workflows(State(engine): State<Arc<WorkflowEngine>>) -> axum::response::Response {
    let service = WorkflowService::new(engine);
    let request = ListWorkflowsRequest {};
    let result = service.list_workflows(request).await;
    RestAdapter::result_to_response(result)
}

/// Start case
pub async fn start_case(
    State(engine): State<Arc<WorkflowEngine>>,
    Path(id): Path<String>,
) -> axum::response::Response {
    let case_id = match CaseId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return RestAdapter::error_to_response(crate::api::models::errors::ApiError::new(
                "BAD_REQUEST",
                "Invalid case_id format",
            ));
        }
    };

    let service = CaseService::new(engine);
    let request = StartCaseRequest { case_id };
    let result = service.start_case(request).await;
    match result {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response(),
        Err(error) => RestAdapter::error_to_response(error),
    }
}

/// Execute case
pub async fn execute_case(
    State(engine): State<Arc<WorkflowEngine>>,
    Path(id): Path<String>,
) -> axum::response::Response {
    let case_id = match CaseId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return RestAdapter::error_to_response(crate::api::models::errors::ApiError::new(
                "BAD_REQUEST",
                "Invalid case_id format",
            ));
        }
    };

    let service = CaseService::new(engine);
    let request = ExecuteCaseRequest { case_id };
    let result = service.execute_case(request).await;
    match result {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response(),
        Err(error) => RestAdapter::error_to_response(error),
    }
}

/// List cases
pub async fn list_cases(
    State(engine): State<Arc<WorkflowEngine>>,
    Query(params): Query<HashMap<String, String>>,
) -> axum::response::Response {
    let spec_id = params
        .get("workflow_id")
        .and_then(|s| WorkflowSpecId::parse_str(s).ok());

    let service = CaseService::new(engine);
    let request = ListCasesRequest { spec_id };
    let result = service.list_cases(request).await;
    RestAdapter::result_to_response(result)
}

/// List patterns
pub async fn list_patterns(State(engine): State<Arc<WorkflowEngine>>) -> axum::response::Response {
    let service = PatternService::new(engine);
    let request = ListPatternsRequest {};
    let result = service.list_patterns(request).await;
    RestAdapter::result_to_response(result)
}

/// Get pattern
pub async fn get_pattern(
    State(engine): State<Arc<WorkflowEngine>>,
    Path(id): Path<u32>,
) -> axum::response::Response {
    let pattern_id = match PatternId::new(id) {
        Ok(id) => id,
        Err(_) => {
            return RestAdapter::error_to_response(crate::api::models::errors::ApiError::new(
                "BAD_REQUEST",
                "Invalid pattern_id format",
            ));
        }
    };

    let service = PatternService::new(engine);
    let request = GetPatternRequest { pattern_id };
    let result = service.get_pattern(request).await;
    RestAdapter::result_to_response(result)
}

/// Execute pattern
pub async fn execute_pattern(
    State(engine): State<Arc<WorkflowEngine>>,
    Path(id): Path<u32>,
    Json(request): Json<serde_json::Value>,
) -> axum::response::Response {
    let pattern_id = match PatternId::new(id) {
        Ok(id) => id,
        Err(_) => {
            return RestAdapter::error_to_response(crate::api::models::errors::ApiError::new(
                "BAD_REQUEST",
                "Invalid pattern_id format",
            ));
        }
    };

    // Extract variables from request
    let mut variables = std::collections::HashMap::new();
    if let Some(obj) = request.as_object() {
        for (key, value) in obj {
            variables.insert(key.clone(), value.to_string());
        }
    }

    let service = PatternService::new(engine);
    let request = ExecutePatternRequest {
        pattern_id,
        variables,
    };
    let result = service.execute_pattern(request).await;
    RestAdapter::result_to_response(result)
}
