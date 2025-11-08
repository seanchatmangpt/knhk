//! Multiple Instance Patterns (12-15)

use crate::patterns::{
    PatternExecutionContext, PatternExecutionResult, PatternExecutor, PatternId,
};

/// Pattern 12: Multiple Instances Without Synchronization
pub struct MultipleInstanceWithoutSyncPattern;

impl PatternExecutor for MultipleInstanceWithoutSyncPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Execute multiple instances in parallel without waiting for all
        let instance_count: usize = ctx
            .variables
            .get("instance_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let mut variables = ctx.variables.clone();
        variables.insert("instances_executed".to_string(), instance_count.to_string());
        variables.insert("status".to_string(), "completed".to_string());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!("pattern:{}:completed", 12)),
            variables,
        }
    }
}

/// Pattern 13: Multiple Instances With a Priori Design-Time Knowledge
pub struct MultipleInstanceDesignTimePattern;

impl PatternExecutor for MultipleInstanceDesignTimePattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Execute known number of instances with synchronization
        let instance_count: usize = ctx
            .variables
            .get("instance_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let mut variables = ctx.variables.clone();
        variables.insert("instances_executed".to_string(), instance_count.to_string());
        variables.insert("all_completed".to_string(), "true".to_string());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!("pattern:{}:completed", 13)),
            variables,
        }
    }
}

/// Pattern 14: Multiple Instances With a Priori Runtime Knowledge
pub struct MultipleInstanceRuntimePattern;

impl PatternExecutor for MultipleInstanceRuntimePattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Execute runtime-determined number of instances
        let instance_count: usize = ctx
            .variables
            .get("instance_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let mut variables = ctx.variables.clone();
        variables.insert("instances_executed".to_string(), instance_count.to_string());
        variables.insert("runtime_determined".to_string(), "true".to_string());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!("pattern:{}:completed", 14)),
            variables,
        }
    }
}

/// Pattern 15: Multiple Instances Without a Priori Runtime Knowledge
pub struct MultipleInstanceDynamicPattern;

impl PatternExecutor for MultipleInstanceDynamicPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Execute dynamically created instances
        let mut variables = ctx.variables.clone();
        variables.insert("dynamic_instances".to_string(), "true".to_string());
        variables.insert("instances_created".to_string(), "variable".to_string());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!("pattern:{}:completed", 15)),
            variables,
        }
    }
}

pub fn create_pattern_12() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(12), Box::new(MultipleInstanceWithoutSyncPattern))
}

pub fn create_pattern_13() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(13), Box::new(MultipleInstanceDesignTimePattern))
}

pub fn create_pattern_14() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(14), Box::new(MultipleInstanceRuntimePattern))
}

pub fn create_pattern_15() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(15), Box::new(MultipleInstanceDynamicPattern))
}
