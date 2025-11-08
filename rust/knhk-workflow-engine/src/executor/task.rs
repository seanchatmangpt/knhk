//! Task execution methods

use crate::case::CaseId;
use crate::error::{WorkflowError, WorkflowResult};
use crate::integration::fortune5::RuntimeClass;
use crate::parser::{Task, WorkflowSpec, WorkflowSpecId};
use crate::patterns::PatternExecutionContext;
use crate::resource::AllocationRequest;
use std::collections::HashMap;
use std::time::Instant;

use super::WorkflowEngine;

/// Execute workflow tasks with resource allocation
pub(super) async fn execute_workflow_tasks(
    engine: &WorkflowEngine,
    case_id: CaseId,
    spec: &WorkflowSpec,
) -> WorkflowResult<()> {
    // Start from start condition
    if let Some(ref start_condition_id) = spec.start_condition {
        if let Some(start_condition) = spec.conditions.get(start_condition_id) {
            // Execute tasks from start condition
            for task_id in &start_condition.outgoing_flows {
                if let Some(task) = spec.tasks.get(task_id) {
                    execute_task_with_allocation(engine, case_id, spec.id, task).await?;
                }
            }
        }
    }

    Ok(())
}

/// Execute a task with resource allocation and Fortune 5 SLO tracking
pub(super) async fn execute_task_with_allocation(
    engine: &WorkflowEngine,
    case_id: CaseId,
    spec_id: WorkflowSpecId,
    task: &Task,
) -> WorkflowResult<()> {
    let task_start_time = Instant::now();
    // Allocate resources if allocation policy is specified
    if let Some(ref policy) = task.allocation_policy {
        let request = AllocationRequest {
            task_id: task.id.clone(),
            spec_id,
            required_roles: task.required_roles.clone(),
            required_capabilities: task.required_capabilities.clone(),
            policy: *policy,
            priority: task.priority.unwrap_or(100) as u8,
        };

        match engine.resource_allocator.allocate(request).await {
            Ok(allocation) => {
                // Resources allocated - proceed with task execution
                // In production, would track allocation and release after execution
                for resource_id in &allocation.resource_ids {
                    engine
                        .resource_allocator
                        .update_workload(*resource_id, 1)
                        .await?;
                }
            }
            Err(e) => {
                // Resource allocation failed - try worklet exception handling
                if let Some(_worklet_id) = task.exception_worklet {
                    let context = PatternExecutionContext {
                        case_id,
                        workflow_id: spec_id,
                        variables: HashMap::new(),
                        arrived_from: std::collections::HashSet::new(),
                        scope_id: String::new(),
                    };
                    if let Some(result) = engine
                        .worklet_executor
                        .handle_exception("resource_unavailable", context)
                        .await?
                    {
                        if !result.success {
                            return Err(e);
                        }
                    } else {
                        return Err(e);
                    }
                } else {
                    return Err(e);
                }
            }
        }
    }

    // Execute task - must actually execute task logic, not just validate constraints
    // Check max_ticks constraint
    if let Some(max_ticks) = task.max_ticks {
        let elapsed_ns = task_start_time.elapsed().as_nanos() as u64;
        let elapsed_ticks = elapsed_ns / 2; // 2ns per tick
        if elapsed_ticks > max_ticks as u64 {
            return Err(WorkflowError::TaskExecutionFailed(format!(
                "Task {} exceeded tick budget: {} ticks > {} ticks",
                task.id, elapsed_ticks, max_ticks
            )));
        }
    }

    // FUTURE: Execute actual task logic based on task type:
    // - Atomic: Execute task handler/connector
    // - Composite: Execute sub-workflow
    // - MultipleInstance: Execute multiple instances
    // For now, this is a placeholder that doesn't actually execute the task
    unimplemented!("execute_task_with_allocation: needs actual task execution implementation for task_id={}, task_type={:?} - must execute task logic (atomic handler, composite sub-workflow, or multiple instance execution) not just validate constraints", task.id, task.task_type);

    // Record SLO metrics if Fortune 5 is enabled
    if let Some(ref fortune5) = engine.fortune5_integration {
        let elapsed_ns = task_start_time.elapsed().as_nanos() as u64;
        // Determine runtime class based on task max_ticks or execution time
        let runtime_class = if task.max_ticks.map_or(false, |ticks| ticks <= 8) {
            RuntimeClass::R1 // Hot path task
        } else if elapsed_ns <= 1_000_000 {
            RuntimeClass::W1 // Warm path
        } else {
            RuntimeClass::C1 // Cold path
        };
        fortune5.record_slo_metric(runtime_class, elapsed_ns).await;
    }

    Ok(())
}
