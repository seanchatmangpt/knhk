//! Cancellation Patterns (32-35)

use crate::patterns::{
    PatternExecutionContext, PatternExecutionResult, PatternExecutor, PatternId,
};

/// Pattern 32: Cancel Activity Instance
pub struct CancelActivityInstancePattern;

impl PatternExecutor for CancelActivityInstancePattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Cancel specific activity instance
        let instance_id = ctx
            .variables
            .get("instance_id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let mut variables = ctx.variables.clone();
        variables.insert("instance_cancelled".to_string(), instance_id.clone());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!(
                "pattern:32:cancel-activity-instance:{}:cancelled",
                instance_id
            )),
            next_activities: Vec::new(),
            variables,
            updates: None,
            cancel_activities: vec![instance_id],
            terminates: false,
        }
    }
}

/// Pattern 33: Cancel Process Instance
pub struct CancelProcessInstancePattern;

impl PatternExecutor for CancelProcessInstancePattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Cancel process instance
        let process_id = ctx
            .variables
            .get("process_id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let mut variables = ctx.variables.clone();
        variables.insert("process_cancelled".to_string(), process_id.clone());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!(
                "pattern:33:cancel-process-instance:{}:cancelled",
                process_id
            )),
            next_activities: Vec::new(),
            variables,
            updates: None,
            cancel_activities: vec![process_id],
            terminates: true,
        }
    }
}

/// Pattern 34: Stop Process Instance
pub struct StopProcessInstancePattern;

impl PatternExecutor for StopProcessInstancePattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Stop process instance
        let mut variables = ctx.variables.clone();
        variables.insert("process_stopped".to_string(), "true".to_string());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!("pattern:{}:completed", 34)),
            next_activities: Vec::new(),
            variables,
            updates: None,
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }
}

/// Pattern 35: Abort Process Instance
pub struct AbortProcessInstancePattern;

impl PatternExecutor for AbortProcessInstancePattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Abort process instance
        let mut variables = ctx.variables.clone();
        variables.insert("process_aborted".to_string(), "true".to_string());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!("pattern:{}:completed", 35)),
            next_activities: Vec::new(),
            variables,
            updates: None,
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }
}
