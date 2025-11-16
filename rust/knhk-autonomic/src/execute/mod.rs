//! # Execute Component - Action Execution & Feedback
//!
//! **Covenant 3**: Feedback loops run at machine speed
//!
//! The Execute component executes planned actions and captures feedback.
//! It monitors action effects, records execution results, and feeds data
//! back to the knowledge base for learning.
//!
//! ## Responsibilities
//!
//! - Execute actions in sequence
//! - Monitor action effects (metric changes)
//! - Capture execution output and errors
//! - Record execution timing (must be ≤8 ticks for hot path)
//! - Feed results to knowledge base
//! - Generate impact analysis
//!
//! ## Example
//!
//! ```rust,no_run
//! use knhk_autonomic::execute::ExecutionComponent;
//! use knhk_autonomic::types::Plan;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let mut executor = ExecutionComponent::new();
//!
//! let plan = /* ... */;
//! let execution = executor.execute_plan(&plan).await?;
//!
//! println!("Executed {} actions", execution.len());
//! # Ok(())
//! # }
//! ```

use crate::types::{Plan, Action, ActionExecution, ExecutionStatus, Metric};
use crate::error::{AutonomicError, Result};
use crate::monitor::MonitoringComponent;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{instrument, debug, warn, error};
use uuid::Uuid;

/// Execution component for running actions
#[derive(Debug, Clone)]
pub struct ExecutionComponent {
    /// Execution history
    history: Arc<RwLock<Vec<ActionExecution>>>,

    /// Monitor for collecting post-execution metrics
    monitor: Arc<RwLock<MonitoringComponent>>,
}

impl Default for ExecutionComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionComponent {
    /// Create a new execution component
    pub fn new() -> Self {
        Self {
            history: Arc::new(RwLock::new(Vec::new())),
            monitor: Arc::new(RwLock::new(MonitoringComponent::new())),
        }
    }

    /// Set monitoring component
    pub async fn set_monitor(&mut self, monitor: MonitoringComponent) {
        let mut m = self.monitor.write().await;
        *m = monitor;
    }

    /// Execute a plan (sequence of actions)
    #[instrument(skip(self, plan, actions))]
    pub async fn execute_plan(
        &mut self,
        plan: &Plan,
        actions: &[Action],
    ) -> Result<Vec<ActionExecution>> {
        let mut executions = Vec::new();

        debug!("Executing plan {} with {} actions", plan.id, plan.actions.len());

        for action_id in &plan.actions {
            // Find action
            let action = actions.iter()
                .find(|a| &a.id == action_id)
                .ok_or_else(|| AutonomicError::Execute(format!("Action not found: {}", action_id)))?;

            // Execute action
            match self.execute_action(action).await {
                Ok(execution) => {
                    debug!("Action {} executed successfully", action_id);
                    executions.push(execution);
                }
                Err(e) => {
                    error!("Action {} failed: {}", action_id, e);

                    // Record failure
                    let execution = ActionExecution {
                        id: Uuid::new_v4(),
                        action_id: *action_id,
                        start_time: Utc::now(),
                        end_time: Utc::now(),
                        status: ExecutionStatus::Failed,
                        output: String::new(),
                        error: Some(e.to_string()),
                        metrics_after: Vec::new(),
                        impact_analysis: "Action failed, no impact".to_string(),
                    };

                    executions.push(execution);

                    // Continue to next action (don't fail entire plan)
                }
            }
        }

        // Store in history
        let mut history = self.history.write().await;
        history.extend(executions.clone());

        Ok(executions)
    }

    /// Execute a single action
    #[instrument(skip(self, action))]
    async fn execute_action(&mut self, action: &Action) -> Result<ActionExecution> {
        let start_time = Utc::now();

        debug!("Executing action: {} ({})", action.description, action.id);

        // Collect metrics before execution
        let metrics_before = {
            let monitor = self.monitor.read().await;
            monitor.collect_metrics().await?
        };

        // Execute action based on implementation
        // In production, this would call actual implementation handlers
        let (status, output, error) = self.invoke_action(action).await;

        // Collect metrics after execution
        let metrics_after = {
            let monitor = self.monitor.read().await;
            monitor.collect_metrics().await?
        };

        let end_time = Utc::now();

        // Analyze impact
        let impact_analysis = self.analyze_impact(&metrics_before, &metrics_after, action);

        let execution = ActionExecution {
            id: Uuid::new_v4(),
            action_id: action.id,
            start_time,
            end_time,
            status,
            output,
            error,
            metrics_after,
            impact_analysis,
        };

        Ok(execution)
    }

    /// Invoke action implementation
    async fn invoke_action(&self, action: &Action) -> (ExecutionStatus, String, Option<String>) {
        // Simplified action execution (placeholder)
        // In production, this would:
        // - Look up implementation handler
        // - Execute with timeout
        // - Capture output/errors
        // - Measure latency (must be ≤8 ticks for hot path)

        debug!("Invoking action implementation: {}", action.implementation);

        // Simulate execution
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let status = ExecutionStatus::Successful;
        let output = format!("Action {} executed successfully", action.description);
        let error = None;

        (status, output, error)
    }

    /// Analyze impact of action execution
    fn analyze_impact(&self, before: &[Metric], after: &[Metric], action: &Action) -> String {
        let mut improvements = Vec::new();
        let mut degradations = Vec::new();

        for metric_after in after {
            if let Some(metric_before) = before.iter().find(|m| m.name == metric_after.name) {
                let change = metric_after.current_value - metric_before.current_value;
                let change_pct = (change / metric_before.current_value) * 100.0;

                if change.abs() > 0.1 {
                    if change_pct < 0.0 && metric_after.metric_type == crate::types::MetricType::Performance {
                        improvements.push(format!("{}: improved by {:.1}%", metric_after.name, change_pct.abs()));
                    } else if change_pct > 0.0 && metric_after.metric_type == crate::types::MetricType::Performance {
                        degradations.push(format!("{}: degraded by {:.1}%", metric_after.name, change_pct));
                    }
                }
            }
        }

        if improvements.is_empty() && degradations.is_empty() {
            return format!("Action {} had minimal impact on metrics", action.description);
        }

        let mut analysis = format!("Action {} impact:", action.description);
        if !improvements.is_empty() {
            analysis.push_str(&format!(" Improvements: {}", improvements.join(", ")));
        }
        if !degradations.is_empty() {
            analysis.push_str(&format!(" Degradations: {}", degradations.join(", ")));
        }

        analysis
    }

    /// Get execution history
    pub async fn get_history(&self) -> Result<Vec<ActionExecution>> {
        let history = self.history.read().await;
        Ok(history.clone())
    }

    /// Get success rate for an action
    pub async fn get_success_rate(&self, action_id: &Uuid) -> Result<f64> {
        let history = self.history.read().await;

        let executions: Vec<_> = history.iter()
            .filter(|e| &e.action_id == action_id)
            .collect();

        if executions.is_empty() {
            return Ok(0.5); // Default for unknown actions
        }

        let successes = executions.iter()
            .filter(|e| e.status == ExecutionStatus::Successful)
            .count();

        Ok(successes as f64 / executions.len() as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ActionType, RiskLevel};

    #[tokio::test]
    async fn test_execute_action() {
        let mut executor = ExecutionComponent::new();

        let action = Action {
            id: Uuid::new_v4(),
            action_type: ActionType::Heal,
            description: "Test action".to_string(),
            target: "test_task".to_string(),
            implementation: "test_handler".to_string(),
            estimated_impact: "Test impact".to_string(),
            risk_level: RiskLevel::LowRisk,
        };

        let execution = executor.execute_action(&action).await.unwrap();

        assert_eq!(execution.action_id, action.id);
        assert_eq!(execution.status, ExecutionStatus::Successful);
    }

    #[tokio::test]
    async fn test_get_success_rate() {
        let executor = ExecutionComponent::new();

        let action_id = Uuid::new_v4();

        // No history = default 0.5
        let rate = executor.get_success_rate(&action_id).await.unwrap();
        assert_eq!(rate, 0.5);
    }
}
