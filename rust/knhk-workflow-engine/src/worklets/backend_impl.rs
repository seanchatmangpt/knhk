//! WorkletExecutionBackend implementation for WorkflowEngine
//!
//! This module implements the WorkletExecutionBackend trait for WorkflowEngine,
//! breaking the circular dependency between worklets and the engine.

use crate::case::{Case, CaseId};
use crate::error::WorkflowResult;
use crate::parser::WorkflowSpecId;
use crate::worklets::WorkletExecutionBackend;
use async_trait::async_trait;

/// Implementation of WorkletExecutionBackend for WorkflowEngine
///
/// This is a newtype wrapper that allows us to implement the trait
/// without creating a circular dependency.
#[async_trait]
impl WorkletExecutionBackend for crate::executor::WorkflowEngine {
    async fn create_case(
        &self,
        spec_id: WorkflowSpecId,
        data: serde_json::Value,
    ) -> WorkflowResult<CaseId> {
        // Delegate to WorkflowEngine::create_case
        crate::executor::WorkflowEngine::create_case(self, spec_id, data).await
    }

    async fn execute_case(&self, case_id: CaseId) -> WorkflowResult<()> {
        // Delegate to WorkflowEngine::execute_case
        crate::executor::WorkflowEngine::execute_case(self, case_id).await
    }

    async fn get_case(&self, case_id: CaseId) -> WorkflowResult<Case> {
        // Delegate to WorkflowEngine::get_case
        crate::executor::WorkflowEngine::get_case(self, case_id).await
    }
}

