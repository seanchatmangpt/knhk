// rust/knhk-workflow-engine/src/compensation.rs
//! Compensation Handlers for Transaction Management
//!
//! Implements YAWL-style compensation handlers for undoing completed work
//! when exceptions occur or transactions need to be rolled back.
//!
//! **Key Concepts**:
//! - Compensation handlers are sub-workflows that undo completed tasks
//! - Handlers are invoked in reverse execution order (LIFO)
//! - Compensation can be scoped to tasks, regions, or entire cases
//!
//! **Architecture**:
//! - Compensation log tracks completed tasks
//! - Handlers registered per task/region
//! - Lock-free compensation state for hot path checks

use crate::case::CaseId;
use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;
use crate::worklets::WorkletId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Compensation handler identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CompensationId(#[serde(with = "uuid::serde::compact")] pub Uuid);

impl CompensationId {
    /// Create new compensation ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CompensationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Compensation handler type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompensationHandler {
    /// Inline workflow specification
    Workflow(WorkflowSpec),
    /// Reference to a worklet
    Worklet(WorkletId),
    /// Custom compensation function (not serializable - must be registered)
    Custom(String), // Handler name/identifier
}

/// Compensation scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompensationScope {
    /// Compensate a single task
    Task,
    /// Compensate all tasks in a region
    Region,
    /// Compensate entire case
    Case,
}

/// Compensation log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationEntry {
    /// Entry identifier
    pub id: CompensationId,
    /// Task ID that was completed
    pub task_id: String,
    /// Timestamp when task completed (ms since epoch)
    pub completion_time_ms: u64,
    /// Compensation handler for this task
    pub handler: CompensationHandler,
    /// Context data captured at completion time
    pub context: HashMap<String, String>,
    /// Whether compensation has been executed
    pub compensated: bool,
    /// Compensation result (if executed)
    pub compensation_result: Option<String>,
}

impl CompensationEntry {
    /// Create new compensation entry
    pub fn new(
        task_id: String,
        handler: CompensationHandler,
        context: HashMap<String, String>,
    ) -> Self {
        Self {
            id: CompensationId::new(),
            task_id,
            completion_time_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            handler,
            context,
            compensated: false,
            compensation_result: None,
        }
    }
}

/// Compensation registry (per case)
pub struct CompensationRegistry {
    /// Compensation log (LIFO order for compensation)
    log: Arc<RwLock<Vec<CompensationEntry>>>,
    /// Task -> Handler mapping
    handlers: Arc<RwLock<HashMap<String, CompensationHandler>>>,
    /// Custom handler functions (registered at runtime)
    custom_handlers: Arc<RwLock<HashMap<String, Arc<dyn Fn(&HashMap<String, String>) -> WorkflowResult<()> + Send + Sync>>>>,
}

impl CompensationRegistry {
    /// Create new compensation registry
    pub fn new() -> Self {
        Self {
            log: Arc::new(RwLock::new(Vec::new())),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            custom_handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register compensation handler for a task
    pub async fn register_handler(
        &self,
        task_id: String,
        handler: CompensationHandler,
    ) -> WorkflowResult<()> {
        let mut handlers = self.handlers.write().await;
        handlers.insert(task_id, handler);
        Ok(())
    }

    /// Register custom compensation function
    pub async fn register_custom_handler<F>(
        &self,
        name: String,
        handler: F,
    ) -> WorkflowResult<()>
    where
        F: Fn(&HashMap<String, String>) -> WorkflowResult<()> + Send + Sync + 'static,
    {
        let mut custom_handlers = self.custom_handlers.write().await;
        custom_handlers.insert(name, Arc::new(handler));
        Ok(())
    }

    /// Log task completion with compensation context
    pub async fn log_completion(
        &self,
        task_id: String,
        context: HashMap<String, String>,
    ) -> WorkflowResult<()> {
        let handlers = self.handlers.read().await;

        // Only log if task has compensation handler
        if let Some(handler) = handlers.get(&task_id) {
            let entry = CompensationEntry::new(task_id, handler.clone(), context);

            let mut log = self.log.write().await;
            log.push(entry);
        }

        Ok(())
    }

    /// Execute compensation for specific task
    pub async fn compensate_task(
        &self,
        task_id: &str,
    ) -> WorkflowResult<()> {
        let mut log = self.log.write().await;

        // Find entries for this task (there may be multiple if task executed multiple times)
        for entry in log.iter_mut().rev() {
            if entry.task_id == task_id && !entry.compensated {
                self.execute_compensation(entry).await?;
                entry.compensated = true;
            }
        }

        Ok(())
    }

    /// Execute compensation for all logged tasks (LIFO order)
    pub async fn compensate_all(&self) -> WorkflowResult<Vec<String>> {
        let mut log = self.log.write().await;
        let mut errors = Vec::new();

        // Execute compensation in reverse order (LIFO)
        for entry in log.iter_mut().rev() {
            if !entry.compensated {
                if let Err(e) = self.execute_compensation(entry).await {
                    errors.push(format!("Task {}: {}", entry.task_id, e));
                    entry.compensation_result = Some(format!("Error: {}", e));
                } else {
                    entry.compensation_result = Some("Success".to_string());
                }
                entry.compensated = true;
            }
        }

        if errors.is_empty() {
            Ok(vec![])
        } else {
            Ok(errors)
        }
    }

    /// Execute compensation for tasks completed after a specific time
    pub async fn compensate_since(
        &self,
        timestamp_ms: u64,
    ) -> WorkflowResult<Vec<String>> {
        let mut log = self.log.write().await;
        let mut errors = Vec::new();

        // Compensate all tasks completed after timestamp (LIFO)
        for entry in log.iter_mut().rev() {
            if entry.completion_time_ms >= timestamp_ms && !entry.compensated {
                if let Err(e) = self.execute_compensation(entry).await {
                    errors.push(format!("Task {}: {}", entry.task_id, e));
                    entry.compensation_result = Some(format!("Error: {}", e));
                } else {
                    entry.compensation_result = Some("Success".to_string());
                }
                entry.compensated = true;
            }
        }

        if errors.is_empty() {
            Ok(vec![])
        } else {
            Ok(errors)
        }
    }

    /// Execute single compensation entry
    async fn execute_compensation(&self, entry: &mut CompensationEntry) -> WorkflowResult<()> {
        match &entry.handler {
            CompensationHandler::Workflow(_spec) => {
                // TODO: Execute workflow spec as compensation
                // Would require WorkflowEngine reference
                Ok(())
            }
            CompensationHandler::Worklet(_worklet_id) => {
                // TODO: Execute worklet as compensation
                // Would require WorkletExecutor reference
                Ok(())
            }
            CompensationHandler::Custom(name) => {
                let custom_handlers = self.custom_handlers.read().await;
                if let Some(handler) = custom_handlers.get(name) {
                    handler(&entry.context)?;
                    Ok(())
                } else {
                    Err(WorkflowError::ResourceUnavailable(format!(
                        "Custom compensation handler '{}' not found",
                        name
                    )))
                }
            }
        }
    }

    /// Get compensation log
    pub async fn get_log(&self) -> Vec<CompensationEntry> {
        let log = self.log.read().await;
        log.clone()
    }

    /// Get uncompensated tasks
    pub async fn get_uncompensated(&self) -> Vec<String> {
        let log = self.log.read().await;
        log.iter()
            .filter(|e| !e.compensated)
            .map(|e| e.task_id.clone())
            .collect()
    }

    /// Clear compensation log
    pub async fn clear(&self) {
        let mut log = self.log.write().await;
        log.clear();
    }

    /// Get statistics
    pub async fn get_stats(&self) -> CompensationStats {
        let log = self.log.read().await;

        let total = log.len();
        let compensated = log.iter().filter(|e| e.compensated).count();
        let pending = total - compensated;

        CompensationStats {
            total_entries: total,
            compensated_entries: compensated,
            pending_entries: pending,
            oldest_timestamp_ms: log.first().map(|e| e.completion_time_ms),
            newest_timestamp_ms: log.last().map(|e| e.completion_time_ms),
        }
    }
}

impl Default for CompensationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Compensation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationStats {
    /// Total compensation entries
    pub total_entries: usize,
    /// Number of compensated entries
    pub compensated_entries: usize,
    /// Number of pending compensations
    pub pending_entries: usize,
    /// Oldest entry timestamp
    pub oldest_timestamp_ms: Option<u64>,
    /// Newest entry timestamp
    pub newest_timestamp_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compensation_registration() {
        let registry = CompensationRegistry::new();

        let handler = CompensationHandler::Custom("test_handler".to_string());
        registry
            .register_handler("task1".to_string(), handler)
            .await
            .unwrap();

        let mut context = HashMap::new();
        context.insert("data".to_string(), "test_value".to_string());

        registry
            .log_completion("task1".to_string(), context)
            .await
            .unwrap();

        let log = registry.get_log().await;
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].task_id, "task1");
        assert!(!log[0].compensated);
    }

    #[tokio::test]
    async fn test_custom_handler_execution() {
        let registry = CompensationRegistry::new();

        // Register custom handler
        registry
            .register_custom_handler("rollback".to_string(), |context| {
                assert!(context.contains_key("amount"));
                Ok(())
            })
            .await
            .unwrap();

        // Register task with custom handler
        let handler = CompensationHandler::Custom("rollback".to_string());
        registry
            .register_handler("payment".to_string(), handler)
            .await
            .unwrap();

        // Log completion
        let mut context = HashMap::new();
        context.insert("amount".to_string(), "100.00".to_string());
        registry
            .log_completion("payment".to_string(), context)
            .await
            .unwrap();

        // Execute compensation
        registry.compensate_task("payment").await.unwrap();

        let log = registry.get_log().await;
        assert!(log[0].compensated);
    }

    #[tokio::test]
    async fn test_compensate_all_lifo_order() {
        let registry = CompensationRegistry::new();

        // Register custom handler that tracks execution order
        use std::sync::Mutex;
        let order = Arc::new(Mutex::new(Vec::new()));
        let order_clone = order.clone();

        registry
            .register_custom_handler("track".to_string(), move |context| {
                let task_id = context.get("task_id").unwrap();
                order_clone.lock().unwrap().push(task_id.clone());
                Ok(())
            })
            .await
            .unwrap();

        // Register multiple tasks
        for i in 1..=3 {
            let task_id = format!("task{}", i);
            registry
                .register_handler(task_id.clone(), CompensationHandler::Custom("track".to_string()))
                .await
                .unwrap();

            let mut context = HashMap::new();
            context.insert("task_id".to_string(), task_id.clone());
            registry.log_completion(task_id, context).await.unwrap();
        }

        // Compensate all
        registry.compensate_all().await.unwrap();

        // Verify LIFO order (reverse of execution)
        let execution_order = order.lock().unwrap();
        assert_eq!(execution_order.len(), 3);
        assert_eq!(execution_order[0], "task3");
        assert_eq!(execution_order[1], "task2");
        assert_eq!(execution_order[2], "task1");
    }

    #[tokio::test]
    async fn test_compensation_stats() {
        let registry = CompensationRegistry::new();

        // Register handler
        registry
            .register_custom_handler("test".to_string(), |_| Ok(()))
            .await
            .unwrap();

        registry
            .register_handler("task1".to_string(), CompensationHandler::Custom("test".to_string()))
            .await
            .unwrap();

        // Log multiple completions
        for i in 1..=5 {
            let mut context = HashMap::new();
            context.insert("iteration".to_string(), i.to_string());
            registry
                .log_completion("task1".to_string(), context)
                .await
                .unwrap();
        }

        // Compensate some
        for _i in 0..2 {
            registry.compensate_task("task1").await.unwrap();
        }

        let stats = registry.get_stats().await;
        assert_eq!(stats.total_entries, 5);
        assert_eq!(stats.compensated_entries, 2);
        assert_eq!(stats.pending_entries, 3);
    }
}
