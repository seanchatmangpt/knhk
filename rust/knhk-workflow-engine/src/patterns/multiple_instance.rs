//! Multiple Instance Patterns (12-15)
//!
//! These patterns control how multiple instances of an activity are spawned and synchronized.
//! The actual parallel execution is handled by the task execution engine based on the
//! metadata set by these patterns.

use crate::patterns::{
    PatternExecutionContext, PatternExecutionResult, PatternExecutor, PatternId,
};
use serde_json::json;

/// Pattern 12: Multiple Instances Without Synchronization
///
/// Spawns multiple instances in parallel without waiting for completion.
/// The workflow continues immediately after spawning all instances.
pub struct MultipleInstanceWithoutSyncPattern;

impl PatternExecutor for MultipleInstanceWithoutSyncPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Extract instance count from context variables
        let instance_count: usize = ctx
            .variables
            .get("instance_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        // Clone variables and add MI execution metadata
        let mut variables = ctx.variables.clone();
        variables.insert("mi_mode".to_string(), "no_sync".to_string());
        variables.insert("instance_count".to_string(), instance_count.to_string());
        variables.insert("mi_wait_for_completion".to_string(), "false".to_string());
        variables.insert("instances_spawned".to_string(), instance_count.to_string());

        // Create updates JSON with instance tracking
        let updates = json!({
            "mi_mode": "no_sync",
            "instance_count": instance_count,
            "wait_for_completion": false,
            "spawned_instances": (0..instance_count).map(|i| {
                json!({
                    "instance_id": i,
                    "status": "spawned",
                    "created_at": chrono::Utc::now().to_rfc3339()
                })
            }).collect::<Vec<_>>()
        });

        PatternExecutionResult {
            success: true,
            next_state: Some(format!("pattern:{}:spawned", 12)),
            next_activities: Vec::new(),
            variables,
            updates: Some(updates),
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }
}

/// Pattern 13: Multiple Instances With a Priori Design-Time Knowledge
///
/// Spawns a known number of instances (determined at design time) and waits for all to complete.
pub struct MultipleInstanceDesignTimePattern;

impl PatternExecutor for MultipleInstanceDesignTimePattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Extract design-time known instance count
        let instance_count: usize = ctx
            .variables
            .get("instance_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        // Clone variables and add MI execution metadata
        let mut variables = ctx.variables.clone();
        variables.insert("mi_mode".to_string(), "design_time".to_string());
        variables.insert("instance_count".to_string(), instance_count.to_string());
        variables.insert("mi_wait_for_completion".to_string(), "true".to_string());
        variables.insert("instances_spawned".to_string(), instance_count.to_string());
        variables.insert("instances_completed".to_string(), "0".to_string());

        // Create updates JSON with instance tracking
        let updates = json!({
            "mi_mode": "design_time",
            "instance_count": instance_count,
            "wait_for_completion": true,
            "spawned_instances": (0..instance_count).map(|i| {
                json!({
                    "instance_id": i,
                    "status": "pending",
                    "created_at": chrono::Utc::now().to_rfc3339()
                })
            }).collect::<Vec<_>>(),
            "completed_instances": []
        });

        PatternExecutionResult {
            success: true,
            next_state: Some(format!("pattern:{}:executing", 13)),
            next_activities: Vec::new(),
            variables,
            updates: Some(updates),
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }
}

/// Pattern 14: Multiple Instances With a Priori Runtime Knowledge
///
/// Spawns a number of instances determined at runtime and waits for all to complete.
pub struct MultipleInstanceRuntimePattern;

impl PatternExecutor for MultipleInstanceRuntimePattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Extract runtime-determined instance count from case data or variables
        let instance_count: usize = ctx
            .variables
            .get("instance_count")
            .and_then(|v| v.parse().ok())
            .or_else(|| {
                // Try to get from runtime data source
                ctx.variables
                    .get("runtime_instance_data")
                    .and_then(|data| {
                        // Parse JSON array length as instance count
                        serde_json::from_str::<Vec<serde_json::Value>>(data)
                            .ok()
                            .map(|arr| arr.len())
                    })
            })
            .unwrap_or(1);

        // Clone variables and add MI execution metadata
        let mut variables = ctx.variables.clone();
        variables.insert("mi_mode".to_string(), "runtime".to_string());
        variables.insert("instance_count".to_string(), instance_count.to_string());
        variables.insert("mi_wait_for_completion".to_string(), "true".to_string());
        variables.insert("instances_spawned".to_string(), instance_count.to_string());
        variables.insert("instances_completed".to_string(), "0".to_string());
        variables.insert("runtime_determined".to_string(), "true".to_string());

        // Create updates JSON with instance tracking
        let updates = json!({
            "mi_mode": "runtime",
            "instance_count": instance_count,
            "wait_for_completion": true,
            "runtime_determined": true,
            "spawned_instances": (0..instance_count).map(|i| {
                json!({
                    "instance_id": i,
                    "status": "pending",
                    "created_at": chrono::Utc::now().to_rfc3339()
                })
            }).collect::<Vec<_>>(),
            "completed_instances": []
        });

        PatternExecutionResult {
            success: true,
            next_state: Some(format!("pattern:{}:executing", 14)),
            next_activities: Vec::new(),
            variables,
            updates: Some(updates),
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }
}

/// Pattern 15: Multiple Instances Without a Priori Runtime Knowledge
///
/// Dynamically spawns instances as needed (instance count not known in advance).
/// Waits for all instances to complete before continuing.
pub struct MultipleInstanceDynamicPattern;

impl PatternExecutor for MultipleInstanceDynamicPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // For dynamic MI, instance count may start at 0 and grow dynamically
        let initial_instance_count: usize = ctx
            .variables
            .get("initial_instance_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        // Check if instances can still be added
        let allow_dynamic_spawning = ctx
            .variables
            .get("allow_dynamic_spawning")
            .map(|v| v == "true")
            .unwrap_or(true);

        // Clone variables and add MI execution metadata
        let mut variables = ctx.variables.clone();
        variables.insert("mi_mode".to_string(), "dynamic".to_string());
        variables.insert("instance_count".to_string(), initial_instance_count.to_string());
        variables.insert("mi_wait_for_completion".to_string(), "true".to_string());
        variables.insert("allow_dynamic_spawning".to_string(), allow_dynamic_spawning.to_string());
        variables.insert("instances_spawned".to_string(), initial_instance_count.to_string());
        variables.insert("instances_completed".to_string(), "0".to_string());
        variables.insert("dynamic_instances".to_string(), "true".to_string());

        // Create updates JSON with instance tracking
        let updates = json!({
            "mi_mode": "dynamic",
            "instance_count": initial_instance_count,
            "wait_for_completion": true,
            "allow_dynamic_spawning": allow_dynamic_spawning,
            "spawned_instances": (0..initial_instance_count).map(|i| {
                json!({
                    "instance_id": i,
                    "status": "pending",
                    "created_at": chrono::Utc::now().to_rfc3339()
                })
            }).collect::<Vec<_>>(),
            "completed_instances": []
        });

        PatternExecutionResult {
            success: true,
            next_state: Some(format!("pattern:{}:executing", 15)),
            next_activities: Vec::new(),
            variables,
            updates: Some(updates),
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }
}

/// Pattern 12: Multiple Instance Without Synchronization
pub fn create_pattern_12() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(12), Box::new(MultipleInstanceWithoutSyncPattern))
}

/// Pattern 13: Multiple Instance With Design-Time Knowledge
pub fn create_pattern_13() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(13), Box::new(MultipleInstanceDesignTimePattern))
}

/// Pattern 14: Multiple Instance With Runtime Knowledge
pub fn create_pattern_14() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(14), Box::new(MultipleInstanceRuntimePattern))
}

/// Pattern 15: Multiple Instance Without Runtime Knowledge
pub fn create_pattern_15() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(15), Box::new(MultipleInstanceDynamicPattern))
}
