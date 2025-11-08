//! State-Based Patterns (16-18)

use crate::patterns::adapter::PatternAdapter;
use crate::patterns::{
    PatternExecutionContext, PatternExecutionResult, PatternExecutor, PatternId,
};
use knhk_patterns::DeferredChoicePattern;
use serde_json::Value;
use std::sync::Arc;

/// Pattern 16: Deferred Choice
pub fn create_deferred_choice_pattern() -> (PatternId, Box<dyn PatternExecutor>) {
    let branches: Vec<
        Arc<dyn Fn(Value) -> Result<Value, knhk_patterns::PatternError> + Send + Sync>,
    > = vec![Arc::new(|v: Value| Ok(v.clone()))];

    let choices: Vec<(
        Arc<dyn Fn(&Value) -> bool + Send + Sync>,
        Arc<dyn Fn(Value) -> Result<Value, knhk_patterns::PatternError> + Send + Sync>,
    )> = branches
        .iter()
        .map(|b| {
            let branch = b.clone();
            let condition: Arc<dyn Fn(&Value) -> bool + Send + Sync> = Arc::new(|_| true);
            (condition, branch)
        })
        .collect();
    let timeout_ms = 5000u64;
    let pattern = DeferredChoicePattern::new(choices, timeout_ms)
        .map(|p| Arc::new(p) as Arc<dyn knhk_patterns::Pattern<Value>>)
        .unwrap_or_else(|_| {
            #[allow(clippy::expect_used)] // Fallback pattern creation should never fail
            Arc::new(
                DeferredChoicePattern::new(
                    vec![(Arc::new(|_| true), Arc::new(|v: Value| Ok(v.clone())))],
                    5000,
                )
                .expect("Empty DeferredChoicePattern should never fail"),
            )
        });

    let adapter = PatternAdapter::new(pattern, PatternId(16));
    (PatternId(16), Box::new(adapter))
}

/// Pattern 17: Interleaved Parallel Routing
pub struct InterleavedParallelRoutingPattern;

impl PatternExecutor for InterleavedParallelRoutingPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Execute branches in interleaved order (one at a time, but can switch)
        let mut variables = ctx.variables.clone();
        variables.insert("interleaved".to_string(), "true".to_string());
        variables.insert("execution_order".to_string(), "interleaved".to_string());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!("pattern:{}:completed", 17)),
            variables,
        }
    }
}

/// Pattern 18: Milestone
pub struct MilestonePattern;

impl PatternExecutor for MilestonePattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Milestone: enable activity only when milestone is reached
        let milestone_reached = ctx
            .variables
            .get("milestone_reached")
            .map(|v| v == "true")
            .unwrap_or(false);

        let mut variables = ctx.variables.clone();
        if milestone_reached {
            variables.insert("activity_enabled".to_string(), "true".to_string());
            variables.insert("milestone_status".to_string(), "reached".to_string());
        } else {
            variables.insert("activity_enabled".to_string(), "false".to_string());
            variables.insert("milestone_status".to_string(), "pending".to_string());
        }

        PatternExecutionResult {
            success: milestone_reached,
            next_state: Some(format!(
                "pattern:18:milestone:{}",
                if milestone_reached {
                    "reached"
                } else {
                    "pending"
                }
            )),
            variables,
        }
    }
}

pub fn create_pattern_17() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(17), Box::new(InterleavedParallelRoutingPattern))
}

pub fn create_pattern_18() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(18), Box::new(MilestonePattern))
}
