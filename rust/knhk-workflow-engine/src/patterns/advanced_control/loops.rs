//! Advanced Control Flow Pattern: Loops (28-29)
use crate::patterns::{PatternExecutionContext, PatternExecutionResult, PatternExecutor};

pub struct StructuredLoopPattern;
impl PatternExecutor for StructuredLoopPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Pattern 28: Structured Loop
        // Execute loop with structured exit condition
        // Uses "continue" variable to determine if loop should continue

        // Check if loop should continue
        let should_continue = ctx
            .variables
            .get("continue")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true); // Default to continuing if not specified

        // Get current iteration count
        let iteration: usize = ctx
            .variables
            .get("iteration")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        // Get max iterations (if specified)
        let max_iterations: usize = ctx
            .variables
            .get("max_iterations")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1000);

        let mut variables = ctx.variables.clone();
        let next_iteration = iteration + 1;
        variables.insert("iteration".to_string(), next_iteration.to_string());

        // Check if loop should exit
        let should_exit = !should_continue || next_iteration >= max_iterations;

        PatternExecutionResult {
            success: true,
            next_state: if should_exit {
                Some("pattern:28:structured-loop:exited".to_string())
            } else {
                Some("pattern:28:structured-loop:iterating".to_string())
            },
            next_activities: if should_exit {
                vec!["loop_exit".to_string()]
            } else {
                vec!["loop_body".to_string()]
            },
            variables,
            updates: Some(serde_json::json!({
                "iteration": next_iteration,
                "should_continue": should_continue,
                "should_exit": should_exit
            })),
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }
}

pub struct RecursionPattern;
impl PatternExecutor for RecursionPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionContext {
        // Pattern 29: Recursion
        // Execute recursive pattern with depth tracking
        // Uses "depth" and "max_depth" variables to control recursion

        // Get current depth
        let depth: usize = ctx
            .variables
            .get("depth")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        // Get max depth (if specified)
        let max_depth: usize = ctx
            .variables
            .get("max_depth")
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);

        // Check if sub-case is done (from variables)
        let sub_case_done = ctx
            .variables
            .get("sub_case_done")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let mut variables = ctx.variables.clone();
        let next_depth = depth + 1;
        variables.insert("depth".to_string(), next_depth.to_string());

        // Check if recursion should continue
        let should_recurse = !sub_case_done && next_depth < max_depth;

        PatternExecutionResult {
            success: true,
            next_state: if should_recurse {
                Some("pattern:29:recursion:recursing".to_string())
            } else {
                Some("pattern:29:recursion:completed".to_string())
            },
            next_activities: if should_recurse {
                vec!["recurse".to_string()]
            } else {
                vec!["return".to_string()]
            },
            variables,
            updates: Some(serde_json::json!({
                "depth": next_depth,
                "max_depth": max_depth,
                "sub_case_done": sub_case_done,
                "should_recurse": should_recurse
            })),
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }
}
