// rust/knhk-workflow-engine/src/autonomic/execute.rs
//! Execute Component for MAPE-K Framework
//!
//! Executes adaptation plans by applying actions to the running system.

use super::plan::{Action, ActionType, AdaptationPlan};
use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Action ID
    pub action_id: super::plan::ActionId,
    /// Whether execution succeeded
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Actual impact (if successful)
    pub actual_impact: Option<f64>,
    /// Execution duration (ms)
    pub duration_ms: u64,
}

/// Executor component
pub struct Executor {
    /// Workflow engine reference (for applying adaptations)
    #[allow(dead_code)]
    engine: Option<Arc<RwLock<crate::executor::WorkflowEngine>>>,
    /// Execution history
    history: Arc<RwLock<Vec<ExecutionResult>>>,
}

impl Executor {
    /// Create new executor
    pub fn new() -> Self {
        Self {
            engine: None,
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create executor with workflow engine
    #[allow(dead_code)]
    pub fn with_engine(engine: Arc<RwLock<crate::executor::WorkflowEngine>>) -> Self {
        Self {
            engine: Some(engine),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Execute adaptation plan
    pub async fn execute(&self, plan: &AdaptationPlan) -> WorkflowResult<Vec<ExecutionResult>> {
        let mut results = Vec::new();

        for action in &plan.actions {
            let result = self.execute_action(action).await?;
            results.push(result);
        }

        // Save to history
        let mut history = self.history.write().await;
        history.extend(results.clone());

        Ok(results)
    }

    /// Execute single action
    async fn execute_action(&self, action: &Action) -> WorkflowResult<ExecutionResult> {
        let start = std::time::Instant::now();

        let result = match &action.action_type {
            ActionType::ScaleInstances { delta } => self.scale_instances(*delta).await,
            ActionType::AdjustResources { resource, amount } => {
                self.adjust_resources(resource, *amount).await
            }
            ActionType::Cancel { target } => self.cancel_task(target).await,
            ActionType::Compensate { task_id } => self.compensate_task(task_id).await,
            ActionType::MigrateRuntime { from, to } => self.migrate_runtime(from, to).await,
            ActionType::OptimizePattern { pattern_id } => self.optimize_pattern(*pattern_id).await,
            ActionType::Custom { name, params } => self.execute_custom(name, params).await,
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(impact) => Ok(ExecutionResult {
                action_id: action.id,
                success: true,
                error: None,
                actual_impact: Some(impact),
                duration_ms,
            }),
            Err(e) => Ok(ExecutionResult {
                action_id: action.id,
                success: false,
                error: Some(e.to_string()),
                actual_impact: None,
                duration_ms,
            }),
        }
    }

    /// Scale multi-instance count
    async fn scale_instances(&self, delta: i32) -> WorkflowResult<f64> {
        // TODO: Integrate with multi-instance tracker
        // For now, simulate execution
        tracing::info!("Scaling instances by {}", delta);
        Ok(0.5) // Estimated impact
    }

    /// Adjust resource allocation
    async fn adjust_resources(&self, resource: &str, amount: f64) -> WorkflowResult<f64> {
        tracing::info!("Adjusting {} by {}", resource, amount);
        Ok(0.3) // Estimated impact
    }

    /// Cancel task
    async fn cancel_task(&self, target: &str) -> WorkflowResult<f64> {
        // TODO: Integrate with cancellation registry
        tracing::info!("Cancelling task {}", target);
        Ok(0.8) // High impact
    }

    /// Compensate task
    async fn compensate_task(&self, task_id: &str) -> WorkflowResult<f64> {
        // TODO: Integrate with compensation registry
        tracing::info!("Compensating task {}", task_id);
        Ok(0.7) // High impact
    }

    /// Migrate runtime class
    async fn migrate_runtime(&self, from: &str, to: &str) -> WorkflowResult<f64> {
        tracing::info!("Migrating from {} to {}", from, to);
        Ok(0.6) // Moderate-high impact
    }

    /// Optimize pattern
    async fn optimize_pattern(&self, pattern_id: u8) -> WorkflowResult<f64> {
        tracing::info!("Optimizing pattern {}", pattern_id);
        Ok(0.4) // Moderate impact
    }

    /// Execute custom action
    async fn execute_custom(&self, name: &str, params: &str) -> WorkflowResult<f64> {
        tracing::info!("Executing custom action '{}' with params '{}'", name, params);
        Ok(0.5) // Estimated impact
    }

    /// Get execution history
    pub async fn get_history(&self) -> Vec<ExecutionResult> {
        let history = self.history.read().await;
        history.clone()
    }

    /// Get success rate
    pub async fn success_rate(&self) -> f64 {
        let history = self.history.read().await;
        if history.is_empty() {
            return 1.0;
        }

        let successes = history.iter().filter(|r| r.success).count();
        successes as f64 / history.len() as f64
    }

    /// Clear history
    pub async fn clear_history(&self) {
        let mut history = self.history.write().await;
        history.clear();
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autonomic::plan::AdaptationPlan;

    #[tokio::test]
    async fn test_executor() {
        let executor = Executor::new();

        let mut plan = AdaptationPlan::new();
        plan.actions.push(Action::new(ActionType::ScaleInstances { delta: 2 }));
        plan.actions.push(Action::new(ActionType::OptimizePattern { pattern_id: 12 }));

        let results = executor.execute(&plan).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.success));
    }

    #[tokio::test]
    async fn test_success_rate() {
        let executor = Executor::new();

        let mut plan = AdaptationPlan::new();
        plan.actions.push(Action::new(ActionType::ScaleInstances { delta: 1 }));

        executor.execute(&plan).await.unwrap();

        let rate = executor.success_rate().await;
        assert!((rate - 1.0).abs() < 0.01);
    }
}
