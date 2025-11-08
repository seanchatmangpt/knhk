//! Cancellation Patterns (19-25)

use crate::patterns::adapter::PatternAdapter;
use crate::patterns::{
    PatternExecutionContext, PatternExecutionResult, PatternExecutor, PatternId,
};
use knhk_patterns::{CancellationPattern, TimeoutPattern};
use serde_json::Value;
use std::sync::Arc;

/// Pattern 19: Cancel Activity
pub struct CancelActivityPattern;

impl PatternExecutor for CancelActivityPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Cancel a specific activity
        let activity_id = ctx
            .variables
            .get("activity_id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let mut variables = ctx.variables.clone();
        variables.insert("activity_cancelled".to_string(), activity_id.clone());
        variables.insert("cancellation_status".to_string(), "success".to_string());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!(
                "pattern:19:cancel-activity:{}:cancelled",
                activity_id
            )),
            variables,
        }
    }
}

/// Pattern 20: Cancel Case
pub fn create_timeout_pattern() -> (PatternId, Box<dyn PatternExecutor>) {
    let timeout_ms = 5000;
    let branch: Arc<dyn Fn(Value) -> Result<Value, knhk_patterns::PatternError> + Send + Sync> =
        Arc::new(|v: Value| Ok(v));

    let pattern = TimeoutPattern::new(branch, timeout_ms)
        .map(|p| Arc::new(p) as Arc<dyn knhk_patterns::Pattern<Value>>)
        .unwrap_or_else(|_| {
            Arc::new(
                #[allow(clippy::expect_used)] // Fallback pattern creation should never fail
                TimeoutPattern::new(Arc::new(|v: Value| Ok(v)), 5000)
                    .expect("TimeoutPattern with valid duration should never fail"),
            )
        });

    let adapter = PatternAdapter::new(pattern, PatternId(20));
    (PatternId(20), Box::new(adapter))
}

/// Pattern 21: Cancel Case
pub fn create_cancellation_pattern() -> (PatternId, Box<dyn PatternExecutor>) {
    let branch = Arc::new(|v: Value| Ok(v.clone()));
    let should_cancel = Arc::new(|| false);
    let pattern = CancellationPattern::new(branch, should_cancel)
        .map(|p| Arc::new(p) as Arc<dyn knhk_patterns::Pattern<Value>>)
        .unwrap_or_else(|_| {
            #[allow(clippy::expect_used)] // Fallback pattern creation should never fail
            Arc::new(
                CancellationPattern::new(Arc::new(|v: Value| Ok(v.clone())), Arc::new(|| false))
                    .expect("CancellationPattern with valid parameters should never fail"),
            )
        });

    let adapter = PatternAdapter::new(pattern, PatternId(21));
    (PatternId(21), Box::new(adapter))
}

/// Pattern 22: Cancel Case
pub struct CancelCasePattern;

impl PatternExecutor for CancelCasePattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Cancel entire workflow case
        let mut variables = ctx.variables.clone();
        variables.insert("case_cancelled".to_string(), "true".to_string());
        variables.insert(
            "cancellation_reason".to_string(),
            ctx.variables
                .get("reason")
                .cloned()
                .unwrap_or_else(|| "user_request".to_string()),
        );

        PatternExecutionResult {
            success: true,
            next_state: Some(format!("pattern:{}:completed", 22)),
            variables,
        }
    }
}

/// Pattern 23: Cancel Region
pub struct CancelRegionPattern;

impl PatternExecutor for CancelRegionPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Cancel a region of activities
        let region_id = ctx
            .variables
            .get("region_id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let mut variables = ctx.variables.clone();
        variables.insert("region_cancelled".to_string(), region_id.clone());
        variables.insert("cancellation_status".to_string(), "success".to_string());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!("pattern:23:cancel-region:{}:cancelled", region_id)),
            variables,
        }
    }
}

/// Pattern 24: Cancel Multiple Instance Activity
pub struct CancelMultipleInstanceActivityPattern;

impl PatternExecutor for CancelMultipleInstanceActivityPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Cancel multiple instance activity
        let activity_id = ctx
            .variables
            .get("activity_id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let mut variables = ctx.variables.clone();
        variables.insert(
            "multiple_instance_cancelled".to_string(),
            activity_id.clone(),
        );
        variables.insert(
            "instances_cancelled".to_string(),
            ctx.variables
                .get("instance_count")
                .cloned()
                .unwrap_or_else(|| "all".to_string()),
        );

        PatternExecutionResult {
            success: true,
            next_state: Some(format!(
                "pattern:24:cancel-multiple-instance-activity:{}:cancelled",
                activity_id
            )),
            variables,
        }
    }
}

/// Pattern 25: Complete Multiple Instance Activity
pub struct CompleteMultipleInstanceActivityPattern;

impl PatternExecutor for CompleteMultipleInstanceActivityPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Complete multiple instance activity
        let activity_id = ctx
            .variables
            .get("activity_id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let mut variables = ctx.variables.clone();
        variables.insert(
            "multiple_instance_completed".to_string(),
            activity_id.clone(),
        );
        variables.insert("completion_status".to_string(), "success".to_string());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!(
                "pattern:25:complete-multiple-instance-activity:{}:completed",
                activity_id
            )),
            variables,
        }
    }
}

pub fn create_pattern_19() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(19), Box::new(CancelActivityPattern))
}

pub fn create_pattern_22() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(22), Box::new(CancelCasePattern))
}

pub fn create_pattern_23() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(23), Box::new(CancelRegionPattern))
}

pub fn create_pattern_24() -> (PatternId, Box<dyn PatternExecutor>) {
    (
        PatternId(24),
        Box::new(CancelMultipleInstanceActivityPattern),
    )
}

pub fn create_pattern_25() -> (PatternId, Box<dyn PatternExecutor>) {
    (
        PatternId(25),
        Box::new(CompleteMultipleInstanceActivityPattern),
    )
}
