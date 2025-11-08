//! Advanced Control Flow Pattern: Triggers (30-31)
use crate::patterns::{PatternExecutionContext, PatternExecutionResult, PatternExecutor};

pub struct TransientTriggerPattern;
impl PatternExecutor for TransientTriggerPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Pattern 30: Transient Trigger
        // Trigger that fires once and is consumed
        // Uses "trigger_fired" or timer event from variables

        // Check if trigger has fired
        let trigger_fired = ctx
            .variables
            .get("trigger_fired")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        // Get trigger source if provided
        let trigger_source = ctx
            .variables
            .get("trigger_source")
            .cloned()
            .unwrap_or_else(|| "transient".to_string());

        let mut variables = ctx.variables.clone();
        variables.insert(
            "transient_trigger_processed".to_string(),
            "true".to_string(),
        );
        variables.insert("trigger_source".to_string(), trigger_source.clone());

        PatternExecutionResult {
            success: true,
            next_state: if trigger_fired {
                Some(format!(
                    "pattern:30:transient-trigger:{}:fired",
                    trigger_source
                ))
            } else {
                Some(format!(
                    "pattern:30:transient-trigger:{}:waiting",
                    trigger_source
                ))
            },
            next_activities: if trigger_fired {
                vec!["continue".to_string()]
            } else {
                Vec::new()
            },
            variables,
            updates: Some(serde_json::json!({
                "trigger_fired": trigger_fired,
                "trigger_source": trigger_source
            })),
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }
}

pub struct PersistentTriggerPattern;
impl PatternExecutor for PersistentTriggerPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Pattern 31: Persistent Trigger
        // Trigger that persists and can fire multiple times
        // Uses "trigger_count" and "trigger_fired" from variables

        // Get trigger count (how many times it should fire)
        let trigger_count: usize = ctx
            .variables
            .get("trigger_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        // Get current fire count
        let fired_count: usize = ctx
            .variables
            .get("fired_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        // Check if trigger should fire now
        let should_fire = fired_count < trigger_count;

        let mut variables = ctx.variables.clone();
        let next_fired_count = if should_fire {
            fired_count + 1
        } else {
            fired_count
        };
        variables.insert("fired_count".to_string(), next_fired_count.to_string());
        variables.insert("persistent_trigger_active".to_string(), "true".to_string());

        PatternExecutionResult {
            success: true,
            next_state: if should_fire {
                Some(format!(
                    "pattern:31:persistent-trigger:fired:{}",
                    next_fired_count
                ))
            } else {
                Some(format!(
                    "pattern:31:persistent-trigger:completed:{}",
                    trigger_count
                ))
            },
            next_activities: if should_fire {
                vec!["trigger_action".to_string()]
            } else {
                Vec::new()
            },
            variables,
            updates: Some(serde_json::json!({
                "trigger_count": trigger_count,
                "fired_count": next_fired_count,
                "should_fire": should_fire
            })),
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }
}
