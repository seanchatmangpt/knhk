//! Advanced Control Flow Pattern: Discriminators (26-27)
use crate::patterns::{PatternExecutionContext, PatternExecutionResult, PatternExecutor};

pub struct BlockingDiscriminatorPattern;
impl PatternExecutor for BlockingDiscriminatorPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Pattern 26: Blocking Discriminator
        // Wait for first branch to complete, then block others until all arrive
        // Uses arrived_from to track which branches have completed

        // Get expected branch count from variables (if provided)
        let expected_branches: usize = ctx
            .variables
            .get("expected_branches")
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| ctx.arrived_from.len().max(1));

        // Check if all branches have arrived
        let all_arrived = ctx.arrived_from.len() >= expected_branches;

        let mut variables = ctx.variables.clone();
        variables.insert("blocking_discriminator".to_string(), "true".to_string());
        variables.insert("all_branches_arrived".to_string(), all_arrived.to_string());
        variables.insert(
            "arrived_count".to_string(),
            ctx.arrived_from.len().to_string(),
        );

        PatternExecutionResult {
            success: true,
            next_state: if all_arrived {
                Some("pattern:26:blocking-discriminator:all-arrived".to_string())
            } else {
                Some("pattern:26:blocking-discriminator:waiting".to_string())
            },
            next_activities: if all_arrived {
                vec!["continue".to_string()]
            } else {
                Vec::new()
            },
            variables,
            updates: Some(serde_json::json!({
                "arrived_from": ctx.arrived_from.iter().collect::<Vec<_>>(),
                "expected_branches": expected_branches,
                "all_arrived": all_arrived
            })),
            cancel_activities: Vec::new(), // Blocking discriminator doesn't cancel, just blocks
            terminates: false,
        }
    }
}

pub struct CancellingDiscriminatorPattern;
impl PatternExecutor for CancellingDiscriminatorPattern {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Pattern 27: Cancelling Discriminator
        // Wait for first branch to complete, then cancel others
        // Uses arrived_from to determine which branches completed and which to cancel

        // Get all expected branches from variables
        let all_branches: Vec<String> = ctx
            .variables
            .get("all_branches")
            .map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_else(|| {
                // If not provided, infer from arrived_from + expected
                let expected: usize = ctx
                    .variables
                    .get("expected_branches")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(2);
                (0..expected).map(|i| format!("branch_{}", i)).collect()
            });

        // First branch that arrived (if any)
        let first_arrived = ctx.arrived_from.iter().next().cloned();

        // Cancel all branches except the first one that arrived
        let cancel_activities: Vec<String> = if let Some(ref first) = first_arrived {
            all_branches
                .iter()
                .filter(|branch| *branch != first)
                .cloned()
                .collect()
        } else {
            // If no branch has arrived yet, cancel all except first
            all_branches.into_iter().skip(1).collect()
        };

        let mut variables = ctx.variables.clone();
        variables.insert("cancelling_discriminator".to_string(), "true".to_string());
        if let Some(ref first) = first_arrived {
            variables.insert("first_branch".to_string(), first.clone());
        }
        variables.insert(
            "cancelled_count".to_string(),
            cancel_activities.len().to_string(),
        );

        PatternExecutionResult {
            success: true,
            next_state: if first_arrived.is_some() {
                Some("pattern:27:cancelling-discriminator:first-completed".to_string())
            } else {
                Some("pattern:27:cancelling-discriminator:waiting".to_string())
            },
            next_activities: if first_arrived.is_some() {
                vec!["continue".to_string()]
            } else {
                Vec::new()
            },
            variables,
            updates: Some(serde_json::json!({
                "arrived_from": ctx.arrived_from.iter().collect::<Vec<_>>(),
                "first_arrived": first_arrived,
                "cancel_activities": cancel_activities
            })),
            cancel_activities,
            terminates: false,
        }
    }
}
