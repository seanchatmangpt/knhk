//! Schema validation
//!
//! Validates workflow schema structure and constraints.

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;

/// Validate workflow specification schema
pub fn validate_workflow_spec(spec: &WorkflowSpec) -> WorkflowResult<()> {
    // Validate workflow has at least one task
    if spec.tasks.is_empty() {
        return Err(WorkflowError::Validation(
            "Workflow must have at least one task".into(),
        ));
    }

    // Validate start condition exists if specified
    if let Some(ref start_cond) = spec.start_condition {
        if !spec.conditions.contains_key(start_cond) {
            return Err(WorkflowError::Validation(format!(
                "Start condition {} not found in workflow",
                start_cond
            )));
        }
    }

    // Validate end condition exists if specified
    if let Some(ref end_cond) = spec.end_condition {
        if !spec.conditions.contains_key(end_cond) {
            return Err(WorkflowError::Validation(format!(
                "End condition {} not found in workflow",
                end_cond
            )));
        }
    }

    // Validate task flows reference valid tasks/conditions
    for (task_id, task) in &spec.tasks {
        for flow_id in &task.outgoing_flows {
            if !spec.tasks.contains_key(flow_id) && !spec.conditions.contains_key(flow_id) {
                return Err(WorkflowError::Validation(format!(
                    "Task {} has outgoing flow to non-existent target {}",
                    task_id, flow_id
                )));
            }
        }
        for flow_id in &task.incoming_flows {
            if !spec.tasks.contains_key(flow_id) && !spec.conditions.contains_key(flow_id) {
                return Err(WorkflowError::Validation(format!(
                    "Task {} has incoming flow from non-existent source {}",
                    task_id, flow_id
                )));
            }
        }
    }

    Ok(())
}

