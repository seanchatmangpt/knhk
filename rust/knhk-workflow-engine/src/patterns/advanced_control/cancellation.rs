//! Advanced Control Flow Pattern: Cancellation (32-35)
use crate::patterns::{PatternExecutionContext, PatternExecutionResult, PatternExecutor};

pub struct CancelActivityInstancePattern;
impl PatternExecutor for CancelActivityInstancePattern {
    fn execute(&self, _ctx: &PatternExecutionContext) -> PatternExecutionResult {
        PatternExecutionResult {
            success: true,
            next_state: Some("pattern:32:cancel-activity-instance:cancelled".to_string()),
            next_activities: Vec::new(),
            variables: std::collections::HashMap::new(),
            updates: None,
            cancel_activities: vec!["activity_instance".to_string()],
            terminates: false,
        }
    }
}

pub struct CancelProcessInstancePattern;
impl PatternExecutor for CancelProcessInstancePattern {
    fn execute(&self, _ctx: &PatternExecutionContext) -> PatternExecutionResult {
        PatternExecutionResult {
            success: true,
            next_state: Some("pattern:33:cancel-process-instance:cancelled".to_string()),
            next_activities: Vec::new(),
            variables: std::collections::HashMap::new(),
            updates: None,
            cancel_activities: vec!["process_instance".to_string()],
            terminates: true,
        }
    }
}

pub struct StopProcessInstancePattern;
impl PatternExecutor for StopProcessInstancePattern {
    fn execute(&self, _ctx: &PatternExecutionContext) -> PatternExecutionResult {
        PatternExecutionResult {
            success: true,
            next_state: Some("pattern:34:stop-process-instance:stopped".to_string()),
            next_activities: Vec::new(),
            variables: std::collections::HashMap::new(),
            updates: None,
            cancel_activities: Vec::new(),
            terminates: true,
        }
    }
}

pub struct AbortProcessInstancePattern;
impl PatternExecutor for AbortProcessInstancePattern {
    fn execute(&self, _ctx: &PatternExecutionContext) -> PatternExecutionResult {
        PatternExecutionResult {
            success: true,
            next_state: Some("pattern:35:abort-process-instance:aborted".to_string()),
            next_activities: Vec::new(),
            variables: std::collections::HashMap::new(),
            updates: None,
            cancel_activities: Vec::new(),
            terminates: true,
        }
    }
}
