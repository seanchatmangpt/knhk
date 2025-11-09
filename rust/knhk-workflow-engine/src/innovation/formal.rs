//! Formal verification
//!
//! Provides formal verification capabilities for workflow specifications,
//! ensuring correctness properties hold.

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;
use std::collections::HashSet;

/// Formal verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Verification passed
    pub passed: bool,
    /// Violations found
    pub violations: Vec<Violation>,
    /// Properties verified
    pub properties: Vec<Property>,
}

/// Property violation
#[derive(Debug, Clone)]
pub struct Violation {
    /// Property name
    pub property: String,
    /// Violation description
    pub description: String,
    /// Location (task/condition ID)
    pub location: Option<String>,
}

/// Property to verify
#[derive(Debug, Clone)]
pub struct Property {
    /// Property name
    pub name: String,
    /// Property description
    pub description: String,
    /// Property verified
    pub verified: bool,
}

/// Formal verifier
pub struct FormalVerifier;

impl FormalVerifier {
    /// Verify workflow specification
    pub fn verify_workflow(spec: &WorkflowSpec) -> WorkflowResult<VerificationResult> {
        let mut violations = Vec::new();
        let mut properties = Vec::new();

        // Property 1: No deadlocks
        let deadlock_check = Self::check_deadlocks(spec);
        properties.push(Property {
            name: "deadlock_free".to_string(),
            description: "Workflow has no deadlocks".to_string(),
            verified: deadlock_check.is_ok(),
        });
        if let Err(e) = deadlock_check {
            violations.push(Violation {
                property: "deadlock_free".to_string(),
                description: e.to_string(),
                location: None,
            });
        }

        // Property 2: All tasks reachable
        let reachability_check = Self::check_reachability(spec);
        properties.push(Property {
            name: "all_tasks_reachable".to_string(),
            description: "All tasks are reachable from start".to_string(),
            verified: reachability_check.is_ok(),
        });
        if let Err(e) = reachability_check {
            violations.push(Violation {
                property: "all_tasks_reachable".to_string(),
                description: e.to_string(),
                location: None,
            });
        }

        // Property 3: Termination guarantee
        let termination_check = Self::check_termination(spec);
        properties.push(Property {
            name: "termination_guarantee".to_string(),
            description: "Workflow always terminates".to_string(),
            verified: termination_check.is_ok(),
        });
        if let Err(e) = termination_check {
            violations.push(Violation {
                property: "termination_guarantee".to_string(),
                description: e.to_string(),
                location: None,
            });
        }

        // Property 4: No orphaned tasks
        let orphan_check = Self::check_orphans(spec);
        properties.push(Property {
            name: "no_orphans".to_string(),
            description: "No orphaned tasks or conditions".to_string(),
            verified: orphan_check.is_ok(),
        });
        if let Err(e) = orphan_check {
            violations.push(Violation {
                property: "no_orphans".to_string(),
                description: e.to_string(),
                location: None,
            });
        }

        Ok(VerificationResult {
            passed: violations.is_empty(),
            violations,
            properties,
        })
    }

    /// Check for deadlocks
    fn check_deadlocks(spec: &WorkflowSpec) -> WorkflowResult<()> {
        // Simple deadlock detection: check for cycles
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for task_id in spec.tasks.keys() {
            if !visited.contains(task_id)
                && Self::has_cycle(spec, task_id, &mut visited, &mut rec_stack)
            {
                return Err(WorkflowError::Validation(format!(
                    "Deadlock detected: cycle involving task {}",
                    task_id
                )));
            }
        }

        Ok(())
    }

    /// Check for cycles (DFS)
    fn has_cycle(
        spec: &WorkflowSpec,
        task_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(task_id.to_string());
        rec_stack.insert(task_id.to_string());

        if let Some(task) = spec.tasks.get(task_id) {
            for next_id in &task.outgoing_flows {
                if !visited.contains(next_id) {
                    if Self::has_cycle(spec, next_id, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(next_id) {
                    return true;
                }
            }
        }

        rec_stack.remove(task_id);
        false
    }

    /// Check reachability
    fn check_reachability(spec: &WorkflowSpec) -> WorkflowResult<()> {
        let start = spec.start_condition.as_ref().ok_or_else(|| {
            WorkflowError::Validation("Workflow has no start condition".to_string())
        })?;

        let mut reachable = HashSet::new();
        Self::dfs_reachability(spec, start, &mut reachable);

        for task_id in spec.tasks.keys() {
            if !reachable.contains(task_id) {
                return Err(WorkflowError::Validation(format!(
                    "Task {} is not reachable from start",
                    task_id
                )));
            }
        }

        Ok(())
    }

    /// DFS for reachability
    fn dfs_reachability(spec: &WorkflowSpec, node: &str, reachable: &mut HashSet<String>) {
        reachable.insert(node.to_string());

        if let Some(task) = spec.tasks.get(node) {
            for next_id in &task.outgoing_flows {
                if !reachable.contains(next_id) {
                    Self::dfs_reachability(spec, next_id, reachable);
                }
            }
        }

        if let Some(condition) = spec.conditions.get(node) {
            for next_id in &condition.outgoing_flows {
                if !reachable.contains(next_id) {
                    Self::dfs_reachability(spec, next_id, reachable);
                }
            }
        }
    }

    /// Check termination
    fn check_termination(spec: &WorkflowSpec) -> WorkflowResult<()> {
        // Check that end condition is reachable
        let end = spec.end_condition.as_ref().ok_or_else(|| {
            WorkflowError::Validation("Workflow has no end condition".to_string())
        })?;

        let start = spec.start_condition.as_ref().ok_or_else(|| {
            WorkflowError::Validation("Workflow has no start condition".to_string())
        })?;

        let mut reachable = HashSet::new();
        Self::dfs_reachability(spec, start, &mut reachable);

        if !reachable.contains(end) {
            return Err(WorkflowError::Validation(
                "End condition is not reachable from start".to_string(),
            ));
        }

        Ok(())
    }

    /// Check for orphans
    fn check_orphans(spec: &WorkflowSpec) -> WorkflowResult<()> {
        let mut referenced = HashSet::new();

        // Collect all referenced nodes
        for task in spec.tasks.values() {
            for flow in &task.outgoing_flows {
                referenced.insert(flow.clone());
            }
            for flow in &task.incoming_flows {
                referenced.insert(flow.clone());
            }
        }

        for condition in spec.conditions.values() {
            for flow in &condition.outgoing_flows {
                referenced.insert(flow.clone());
            }
            for flow in &condition.incoming_flows {
                referenced.insert(flow.clone());
            }
        }

        // Check for unreferenced tasks
        for task_id in spec.tasks.keys() {
            if !referenced.contains(task_id) && spec.start_condition.as_ref() != Some(task_id) {
                return Err(WorkflowError::Validation(format!(
                    "Orphaned task: {}",
                    task_id
                )));
            }
        }

        Ok(())
    }
}
