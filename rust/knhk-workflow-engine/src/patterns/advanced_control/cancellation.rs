//! Advanced Control Flow Pattern: Cancellation (32-35)
use crate::patterns::{PatternExecutionContext, PatternExecutionResult, PatternExecutor};

pub struct CancelActivityInstancePattern;
impl PatternExecutor for CancelActivityInstancePattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Pattern 32: Cancel Activity Instance
        // Cancel specific activity instance(s)
        // Uses "activity_ids" or "instance_id" from variables to determine what to cancel

        // Get activity IDs to cancel from variables
        let activity_ids: Vec<String> = if let Some(ids_str) = ctx.variables.get("activity_ids") {
            ids_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else if let Some(instance_id) = ctx.variables.get("instance_id") {
            vec![instance_id.clone()]
        } else {
            // Default: cancel activity in scope
            if ctx.scope_id.is_empty() {
                vec!["activity_instance".to_string()]
            } else {
                vec![format!("{}:activity", ctx.scope_id)]
            }
        };

        let mut variables = ctx.variables.clone();
        variables.insert("instance_cancelled".to_string(), "true".to_string());
        variables.insert(
            "cancelled_count".to_string(),
            activity_ids.len().to_string(),
        );

        PatternExecutionResult {
            success: true,
            next_state: Some(format!(
                "pattern:32:cancel-activity-instance:cancelled:{}",
                activity_ids.join(",")
            )),
            next_activities: Vec::new(),
            variables,
            updates: Some(serde_json::json!({
                "cancelled_activities": activity_ids,
                "scope_id": ctx.scope_id
            })),
            cancel_activities: activity_ids,
            terminates: false, // Cancel activity doesn't terminate workflow
        }
    }
}

pub struct CancelProcessInstancePattern;
impl PatternExecutor for CancelProcessInstancePattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Pattern 33: Cancel Process Instance
        // Cancel entire process instance - CRITICAL: MUST terminate workflow
        // Uses scope_id or case_id to determine what to cancel

        // Determine what to cancel based on scope
        let cancel_targets = if !ctx.scope_id.is_empty() {
            vec![format!("process:{}", ctx.scope_id)]
        } else {
            vec![format!("process:{}", ctx.case_id)]
        };

        let mut variables = ctx.variables.clone();
        variables.insert("process_cancelled".to_string(), "true".to_string());
        variables.insert("cancelled_at".to_string(), chrono::Utc::now().to_rfc3339());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!(
                "pattern:33:cancel-process-instance:cancelled:{}",
                ctx.case_id
            )),
            next_activities: Vec::new(),
            variables,
            updates: Some(serde_json::json!({
                "cancelled_process": cancel_targets,
                "case_id": ctx.case_id.to_string(),
                "scope_id": ctx.scope_id
            })),
            cancel_activities: cancel_targets,
            terminates: true, // CRITICAL: Cancel process MUST terminate
        }
    }
}

pub struct StopProcessInstancePattern;
impl PatternExecutor for StopProcessInstancePattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Pattern 34: Stop Process Instance
        // Stop process instance gracefully - CRITICAL: MUST terminate

        let mut variables = ctx.variables.clone();
        variables.insert("process_stopped".to_string(), "true".to_string());
        variables.insert("stopped_at".to_string(), chrono::Utc::now().to_rfc3339());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!(
                "pattern:34:stop-process-instance:stopped:{}",
                ctx.case_id
            )),
            next_activities: Vec::new(),
            variables,
            updates: Some(serde_json::json!({
                "stopped_process": ctx.case_id.to_string(),
                "scope_id": ctx.scope_id
            })),
            cancel_activities: Vec::new(), // Stop doesn't cancel, just stops
            terminates: true,              // CRITICAL: Stop MUST terminate
        }
    }
}

pub struct AbortProcessInstancePattern;
impl PatternExecutor for AbortProcessInstancePattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Pattern 35: Abort Process Instance
        // Abort process instance immediately - CRITICAL: MUST terminate

        // Get abort reason if provided
        let abort_reason = ctx
            .variables
            .get("abort_reason")
            .cloned()
            .unwrap_or_else(|| "Aborted by pattern 35".to_string());

        let mut variables = ctx.variables.clone();
        variables.insert("process_aborted".to_string(), "true".to_string());
        variables.insert("abort_reason".to_string(), abort_reason.clone());
        variables.insert("aborted_at".to_string(), chrono::Utc::now().to_rfc3339());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!(
                "pattern:35:abort-process-instance:aborted:{}",
                ctx.case_id
            )),
            next_activities: Vec::new(),
            variables,
            updates: Some(serde_json::json!({
                "aborted_process": ctx.case_id.to_string(),
                "abort_reason": abort_reason,
                "scope_id": ctx.scope_id
            })),
            cancel_activities: Vec::new(), // Abort doesn't cancel, just aborts
            terminates: true,              // CRITICAL: Abort MUST terminate
        }
    }
}
