//! Advanced Control Flow Pattern: Control (36-39)
use crate::patterns::{PatternExecutionContext, PatternExecutionResult, PatternExecutor};

pub struct DisableActivityPattern;
impl PatternExecutor for DisableActivityPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Pattern 36: Disable Activity
        // Disable specific activity(ies)
        // Uses "activity_id" or "activity_ids" from variables

        let activity_ids: Vec<String> = if let Some(ids_str) = ctx.variables.get("activity_ids") {
            ids_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else if let Some(activity_id) = ctx.variables.get("activity_id") {
            vec![activity_id.clone()]
        } else {
            vec!["activity".to_string()]
        };

        let mut variables = ctx.variables.clone();
        variables.insert("activity_disabled".to_string(), "true".to_string());
        variables.insert("disabled_count".to_string(), activity_ids.len().to_string());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!(
                "pattern:36:disable-activity:disabled:{}",
                activity_ids.join(",")
            )),
            next_activities: Vec::new(),
            variables,
            updates: Some(serde_json::json!({
                "disabled_activities": activity_ids,
                "scope_id": ctx.scope_id
            })),
            cancel_activities: Vec::new(), // Disable doesn't cancel, just disables
            terminates: false,
        }
    }
}

pub struct SkipActivityPattern;
impl PatternExecutor for SkipActivityPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Pattern 37: Skip Activity
        // Skip specific activity(ies)
        // Uses "activity_id" or "activity_ids" from variables

        let activity_ids: Vec<String> = if let Some(ids_str) = ctx.variables.get("activity_ids") {
            ids_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else if let Some(activity_id) = ctx.variables.get("activity_id") {
            vec![activity_id.clone()]
        } else {
            vec!["activity".to_string()]
        };

        let mut variables = ctx.variables.clone();
        variables.insert("activity_skipped".to_string(), "true".to_string());
        variables.insert("skipped_count".to_string(), activity_ids.len().to_string());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!(
                "pattern:37:skip-activity:skipped:{}",
                activity_ids.join(",")
            )),
            next_activities: Vec::new(), // Skip doesn't schedule activities
            variables,
            updates: Some(serde_json::json!({
                "skipped_activities": activity_ids,
                "scope_id": ctx.scope_id
            })),
            cancel_activities: Vec::new(), // Skip doesn't cancel
            terminates: false,
        }
    }
}

pub struct ActivityInstanceMultipleThreadsPattern;
impl PatternExecutor for ActivityInstanceMultipleThreadsPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Pattern 38: Activity Instance in Multiple Threads
        // Execute activity in multiple threads
        // Uses "thread_count" from variables to determine number of threads

        // Get thread count from variables
        let thread_count: usize = ctx
            .variables
            .get("thread_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(2); // Default to 2 threads

        // Validate thread count (max 1000 to prevent resource exhaustion)
        let thread_count = thread_count.min(1000).max(1);

        // Generate thread activity IDs
        let thread_activities: Vec<String> = (1..=thread_count)
            .map(|i| format!("thread_{}", i))
            .collect();

        let mut variables = ctx.variables.clone();
        variables.insert("threads_used".to_string(), thread_count.to_string());
        variables.insert("parallel_execution".to_string(), "true".to_string());

        PatternExecutionResult {
            success: true,
            next_state: Some(format!(
                "pattern:38:activity-instance-multiple-threads:executing:{}",
                thread_count
            )),
            next_activities: thread_activities,
            variables,
            updates: Some(serde_json::json!({
                "thread_count": thread_count,
                "threads": thread_activities
            })),
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }
}

pub struct ThreadMergePattern;
impl PatternExecutor for ThreadMergePattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionContext {
        // Pattern 39: Thread Merge
        // Merge multiple threads (AND-join semantics)
        // Uses arrived_from to check if all threads have completed

        // Get expected thread count from variables
        let expected_threads: usize = ctx
            .variables
            .get("expected_threads")
            .or_else(|| ctx.variables.get("thread_count"))
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| ctx.arrived_from.len().max(1));

        // Check if all threads have arrived (AND-join)
        let all_threads_arrived = ctx.arrived_from.len() >= expected_threads;

        let mut variables = ctx.variables.clone();
        variables.insert("threads_merged".to_string(), "true".to_string());
        variables.insert(
            "merge_status".to_string(),
            if all_threads_arrived {
                "success".to_string()
            } else {
                "waiting".to_string()
            },
        );
        variables.insert(
            "arrived_threads".to_string(),
            ctx.arrived_from.len().to_string(),
        );
        variables.insert("expected_threads".to_string(), expected_threads.to_string());

        PatternExecutionResult {
            success: true,
            next_state: if all_threads_arrived {
                Some("pattern:39:thread-merge:merged".to_string())
            } else {
                Some("pattern:39:thread-merge:waiting".to_string())
            },
            next_activities: if all_threads_arrived {
                vec!["continue".to_string()]
            } else {
                Vec::new() // Wait for more threads
            },
            variables,
            updates: Some(serde_json::json!({
                "arrived_from": ctx.arrived_from.iter().collect::<Vec<_>>(),
                "expected_threads": expected_threads,
                "all_threads_arrived": all_threads_arrived
            })),
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }
}
