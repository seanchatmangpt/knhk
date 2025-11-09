//! Pattern service
//!
//! Service layer for pattern operations.

use crate::api::models::{
    errors::ApiError,
    requests::{ExecutePatternRequest, GetPatternRequest, ListPatternsRequest},
    responses::{ExecutePatternResponse, GetPatternResponse, ListPatternsResponse},
    ApiResult,
};
use crate::executor::WorkflowEngine;
use crate::patterns::PatternExecutionContext;
use std::collections::HashSet;
use std::sync::Arc;

/// Pattern service for pattern operations
pub struct PatternService {
    engine: Arc<WorkflowEngine>,
}

impl PatternService {
    /// Create a new pattern service
    pub fn new(engine: Arc<WorkflowEngine>) -> Self {
        Self { engine }
    }

    /// List all registered patterns
    pub async fn list_patterns(
        &self,
        _request: ListPatternsRequest,
    ) -> ApiResult<ListPatternsResponse> {
        let registry = self.engine.pattern_registry();
        let patterns = registry.list();

        Ok(ListPatternsResponse { patterns })
    }

    /// Get pattern information
    pub async fn get_pattern(&self, request: GetPatternRequest) -> ApiResult<GetPatternResponse> {
        let registry = self.engine.pattern_registry();
        let _executor = registry.get(&request.pattern_id).ok_or_else(|| {
            ApiError::new(
                "NOT_FOUND",
                format!("Pattern {} not found", request.pattern_id),
            )
        })?;

        Ok(GetPatternResponse {
            pattern_id: request.pattern_id,
            name: format!("{}", request.pattern_id),
        })
    }

    /// Execute a pattern
    pub async fn execute_pattern(
        &self,
        request: ExecutePatternRequest,
    ) -> ApiResult<ExecutePatternResponse> {
        let registry = self.engine.pattern_registry();
        let executor = registry.get(&request.pattern_id).ok_or_else(|| {
            ApiError::new(
                "NOT_FOUND",
                format!("Pattern {} not found", request.pattern_id),
            )
        })?;

        // Create execution context
        let context = PatternExecutionContext {
            case_id: crate::case::CaseId::new(),
            workflow_id: crate::parser::WorkflowSpecId::new(),
            variables: request.variables,
            arrived_from: HashSet::new(),
            scope_id: String::new(),
        };

        // Execute pattern
        let result = executor.execute(&context);

        Ok(ExecutePatternResponse {
            pattern_id: request.pattern_id,
            success: result.success,
            variables: result.variables,
        })
    }
}
