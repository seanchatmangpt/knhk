//! Hook registry for workflow lifecycle events
//!
//! Provides hooks for:
//! - Task execution (before/after)
//! - Case lifecycle (create, start, complete, cancel)
//! - Workflow registration
//! - Pattern execution

use crate::case::CaseId;
use crate::error::WorkflowResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Hook type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookType {
    /// Before task execution
    BeforeTaskExecution,
    /// After task execution
    AfterTaskExecution,
    /// Before case creation
    BeforeCaseCreation,
    /// After case creation
    AfterCaseCreation,
    /// Before case start
    BeforeCaseStart,
    /// After case start
    AfterCaseStart,
    /// Before case completion
    BeforeCaseCompletion,
    /// After case completion
    AfterCaseCompletion,
    /// Before case cancellation
    BeforeCaseCancellation,
    /// After case cancellation
    AfterCaseCancellation,
    /// Before workflow registration
    BeforeWorkflowRegistration,
    /// After workflow registration
    AfterWorkflowRegistration,
    /// Before pattern execution
    BeforePatternExecution,
    /// After pattern execution
    AfterPatternExecution,
}

/// Hook function type
pub type HookFn = Arc<
    dyn Fn(&HookContext) -> std::pin::Pin<Box<dyn std::future::Future<Output = HookResult> + Send>>
        + Send
        + Sync,
>;

/// Hook context
#[derive(Debug, Clone)]
pub struct HookContext {
    /// Hook type
    pub hook_type: HookType,
    /// Case ID (if applicable)
    pub case_id: Option<CaseId>,
    /// Workflow spec ID (if applicable)
    pub workflow_spec_id: Option<String>,
    /// Task ID (if applicable)
    pub task_id: Option<String>,
    /// Pattern ID (if applicable)
    pub pattern_id: Option<u32>,
    /// Context data
    pub data: serde_json::Value,
}

/// Hook result
#[derive(Debug, Clone)]
pub struct HookResult {
    /// Whether to continue execution
    pub continue_execution: bool,
    /// Modified context data (if any)
    pub modified_data: Option<serde_json::Value>,
    /// Error message (if hook failed)
    pub error: Option<String>,
}

impl HookResult {
    /// Create success result
    pub fn success() -> Self {
        Self {
            continue_execution: true,
            modified_data: None,
            error: None,
        }
    }

    /// Create success result with modified data
    pub fn success_with_data(data: serde_json::Value) -> Self {
        Self {
            continue_execution: true,
            modified_data: Some(data),
            error: None,
        }
    }

    /// Create failure result
    pub fn failure(error: String) -> Self {
        Self {
            continue_execution: false,
            modified_data: None,
            error: Some(error),
        }
    }
}

/// Workflow hook metadata
#[derive(Clone)]
pub struct WorkflowHook {
    /// Hook ID
    pub id: String,
    /// Hook type
    pub hook_type: HookType,
    /// Hook name
    pub name: String,
    /// Hook description
    pub description: String,
    /// Hook function
    pub hook_fn: HookFn,
    /// Enabled flag
    pub enabled: bool,
    /// Priority (lower = higher priority)
    pub priority: u32,
}

/// Hook registry error
#[derive(Debug, Clone)]
pub enum HookError {
    /// Hook not found
    NotFound(String),
    /// Hook execution failed
    ExecutionFailed(String),
    /// Invalid hook configuration
    InvalidConfiguration(String),
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookError::NotFound(id) => write!(f, "Hook not found: {}", id),
            HookError::ExecutionFailed(msg) => write!(f, "Hook execution failed: {}", msg),
            HookError::InvalidConfiguration(msg) => {
                write!(f, "Invalid hook configuration: {}", msg)
            }
        }
    }
}

impl std::error::Error for HookError {}

/// Hook registry
pub struct HookRegistry {
    /// Hooks by type
    hooks: Arc<RwLock<HashMap<HookType, Vec<WorkflowHook>>>>,
    /// Hook metadata by ID
    hook_metadata: Arc<RwLock<HashMap<String, WorkflowHook>>>,
}

impl HookRegistry {
    /// Create new hook registry
    pub fn new() -> Self {
        Self {
            hooks: Arc::new(RwLock::new(HashMap::new())),
            hook_metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a hook
    pub async fn register(&self, hook: WorkflowHook) -> WorkflowResult<()> {
        if hook.id.is_empty() {
            return Err(WorkflowError::Validation(
                "Hook ID cannot be empty".to_string(),
            ));
        }

        let mut hooks = self.hooks.write().await;
        let mut metadata = self.hook_metadata.write().await;

        // Check for duplicate ID
        if metadata.contains_key(&hook.id) {
            return Err(WorkflowError::Validation(format!(
                "Hook with ID {} already exists",
                hook.id
            )));
        }

        // Add to hooks by type
        hooks
            .entry(hook.hook_type)
            .or_insert_with(Vec::new)
            .push(hook.clone());

        // Sort by priority
        if let Some(hooks_for_type) = hooks.get_mut(&hook.hook_type) {
            hooks_for_type.sort_by_key(|h| h.priority);
        }

        // Add to metadata
        metadata.insert(hook.id.clone(), hook);

        Ok(())
    }

    /// Execute hooks for a given type
    pub async fn execute_hooks(
        &self,
        hook_type: HookType,
        context: HookContext,
    ) -> WorkflowResult<HookResult> {
        let hooks = self.hooks.read().await;
        let hooks_for_type = hooks.get(&hook_type).cloned().unwrap_or_default();
        drop(hooks);

        // Execute hooks in priority order
        let mut final_result = HookResult::success();
        let mut modified_data = context.data.clone();

        for hook in hooks_for_type {
            if !hook.enabled {
                continue;
            }

            let hook_context = HookContext {
                hook_type: context.hook_type,
                case_id: context.case_id.clone(),
                workflow_spec_id: context.workflow_spec_id.clone(),
                task_id: context.task_id.clone(),
                pattern_id: context.pattern_id,
                data: modified_data.clone(),
            };

            let result = (hook.hook_fn)(&hook_context).await;

            if !result.continue_execution {
                return Ok(result);
            }

            if let Some(data) = result.modified_data {
                modified_data = data;
            }

            if let Some(error) = result.error {
                final_result = HookResult::failure(error);
                break;
            }
        }

        final_result.modified_data = Some(modified_data);
        Ok(final_result)
    }

    /// Get hook by ID
    pub async fn get_hook(&self, id: &str) -> Option<WorkflowHook> {
        let metadata = self.hook_metadata.read().await;
        metadata.get(id).cloned()
    }

    /// List all hooks
    pub async fn list_hooks(&self) -> Vec<WorkflowHook> {
        let metadata = self.hook_metadata.read().await;
        metadata.values().cloned().collect()
    }

    /// List hooks by type
    pub async fn list_hooks_by_type(&self, hook_type: HookType) -> Vec<WorkflowHook> {
        let hooks = self.hooks.read().await;
        hooks.get(&hook_type).cloned().unwrap_or_default()
    }

    /// Enable/disable a hook
    pub async fn set_hook_enabled(&self, id: &str, enabled: bool) -> WorkflowResult<()> {
        let mut hooks = self.hooks.write().await;
        let mut metadata = self.hook_metadata.write().await;

        if let Some(hook) = metadata.get_mut(id) {
            hook.enabled = enabled;

            // Update in hooks map
            if let Some(hooks_for_type) = hooks.get_mut(&hook.hook_type) {
                if let Some(h) = hooks_for_type.iter_mut().find(|h| h.id == id) {
                    h.enabled = enabled;
                }
            }
            Ok(())
        } else {
            Err(WorkflowError::Validation(format!("Hook {} not found", id)))
        }
    }

    /// Remove a hook
    pub async fn remove_hook(&self, id: &str) -> WorkflowResult<()> {
        let mut hooks = self.hooks.write().await;
        let mut metadata = self.hook_metadata.write().await;

        if let Some(hook) = metadata.remove(id) {
            if let Some(hooks_for_type) = hooks.get_mut(&hook.hook_type) {
                hooks_for_type.retain(|h| h.id != id);
            }
            Ok(())
        } else {
            Err(WorkflowError::Validation(format!("Hook {} not found", id)))
        }
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hook_registry() {
        let registry = HookRegistry::new();

        let hook = WorkflowHook {
            id: "test-hook".to_string(),
            hook_type: HookType::BeforeTaskExecution,
            name: "Test Hook".to_string(),
            description: "Test hook".to_string(),
            hook_fn: Arc::new(|_ctx| Box::pin(async move { HookResult::success() })),
            enabled: true,
            priority: 0,
        };

        registry
            .register(hook)
            .await
            .expect("Failed to register hook");

        let hooks = registry.list_hooks().await;
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].id, "test-hook");
    }

    #[tokio::test]
    async fn test_hook_execution() {
        let registry = HookRegistry::new();

        let hook = WorkflowHook {
            id: "test-hook".to_string(),
            hook_type: HookType::BeforeTaskExecution,
            name: "Test Hook".to_string(),
            description: "Test hook".to_string(),
            hook_fn: Arc::new(|ctx| {
                let data = ctx.data.clone();
                Box::pin(async move {
                    let mut modified = data;
                    modified["executed"] = serde_json::Value::Bool(true);
                    HookResult::success_with_data(modified)
                })
            }),
            enabled: true,
            priority: 0,
        };

        registry
            .register(hook)
            .await
            .expect("Failed to register hook");

        let context = HookContext {
            hook_type: HookType::BeforeTaskExecution,
            case_id: None,
            workflow_spec_id: None,
            task_id: Some("task-1".to_string()),
            pattern_id: None,
            data: serde_json::json!({}),
        };

        let result = registry
            .execute_hooks(HookType::BeforeTaskExecution, context)
            .await
            .expect("Failed to execute hooks");

        assert!(result.continue_execution);
        assert_eq!(
            result.modified_data.expect("Expected modified_data")["executed"],
            serde_json::Value::Bool(true)
        );
    }
}
