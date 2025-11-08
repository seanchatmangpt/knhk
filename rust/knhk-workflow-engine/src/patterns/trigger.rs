//! Trigger Patterns (40-43)

use crate::patterns::{
    PatternExecutionContext, PatternExecutionResult, PatternExecutor, PatternId,
};

/// Pattern 40: External Trigger
pub struct ExternalTriggerPattern;

impl PatternExecutor for ExternalTriggerPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Handle external trigger event
        let trigger_source = ctx
            .variables
            .get("trigger_source")
            .cloned()
            .unwrap_or_else(|| "external".to_string());

        let mut variables = ctx.variables.clone();
        variables.insert("trigger_received".to_string(), "true".to_string());
        variables.insert("trigger_source".to_string(), trigger_source.clone());
        variables.insert("trigger_type".to_string(), "external".to_string());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!(
                "pattern:40:external-trigger:{}:received",
                trigger_source
            )),
            variables,
        }
    }
}

/// Pattern 41: Event-Based Trigger
pub struct EventBasedTriggerPattern;

impl PatternExecutor for EventBasedTriggerPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Handle event-based trigger
        let event_type = ctx
            .variables
            .get("event_type")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let mut variables = ctx.variables.clone();
        variables.insert("event_triggered".to_string(), "true".to_string());
        variables.insert("event_type".to_string(), event_type.clone());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!(
                "pattern:41:event-based-trigger:{}:triggered",
                event_type
            )),
            variables,
        }
    }
}

/// Pattern 42: Multiple Trigger
pub struct MultipleTriggerPattern;

impl PatternExecutor for MultipleTriggerPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Handle multiple triggers (wait for all)
        let trigger_count: usize = ctx
            .variables
            .get("trigger_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let mut variables = ctx.variables.clone();
        variables.insert("all_triggers_received".to_string(), "true".to_string());
        variables.insert("trigger_count".to_string(), trigger_count.to_string());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!("pattern:{}:completed", 42)),
            variables,
        }
    }
}

/// Pattern 43: Cancel Trigger
pub struct CancelTriggerPattern;

impl PatternExecutor for CancelTriggerPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Cancel trigger-based activity
        let trigger_id = ctx
            .variables
            .get("trigger_id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let mut variables = ctx.variables.clone();
        variables.insert("trigger_cancelled".to_string(), "true".to_string());
        variables.insert("trigger_id".to_string(), trigger_id.clone());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!(
                "pattern:43:cancel-trigger:{}:cancelled",
                trigger_id
            )),
            variables,
        }
    }
}

pub fn create_pattern_40() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(40), Box::new(ExternalTriggerPattern))
}

pub fn create_pattern_41() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(41), Box::new(EventBasedTriggerPattern))
}

pub fn create_pattern_42() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(42), Box::new(MultipleTriggerPattern))
}

pub fn create_pattern_43() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(43), Box::new(CancelTriggerPattern))
}
