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

    // Execute task based on task type
    match task.task_type {
        crate::parser::TaskType::Atomic => {
            // Atomic task: Execute via work item service (human task) or connector (automated)
            // For now, create a work item for human tasks
            // FUTURE: Add connector integration for automated atomic tasks
            let case = engine.get_case(case_id).await?;
            let work_item_id = engine
                .work_item_service
                .create_work_item(
                    case_id.to_string(),
                    spec_id,
                    task.id.clone(),
                    case.data.clone(),
                )
                .await?;

            // Wait for work item completion (in production, would poll or use async notification)
            // For now, mark as completed immediately (simplified - real implementation would wait)
            // This is a placeholder - actual implementation would:
            // 1. Create work item
            // 2. Wait for human completion or connector execution
            // 3. Get result and update case variables
            engine
                .work_item_service
                .complete(work_item_id.as_str(), case.data.clone())
                .await?;
        }
        crate::parser::TaskType::Composite => {
            // Composite task: Execute as sub-workflow
            // NOTE: This creates a new case for the sub-workflow to avoid infinite recursion
            // In production, sub-workflow spec would be stored in task metadata or loaded separately
            // For now, use the same spec (simplified - real implementation would load sub-workflow)
            let sub_case_id = engine.create_case(spec_id, serde_json::json!({})).await?;
            engine.start_case(sub_case_id).await?;

            // Execute sub-workflow directly without going through execute_case to avoid recursion
            // Get workflow specification
            let specs = engine.specs.read().await;
            let sub_spec = specs.get(&spec_id).ok_or_else(|| {
                WorkflowError::InvalidSpecification(format!("Workflow {} not found", spec_id))
            })?;
            let sub_spec_clone = sub_spec.clone();
            drop(specs);

            // Execute sub-workflow tasks directly
            super::task::execute_workflow_tasks(engine, sub_case_id, &sub_spec_clone).await?;

            // Get result from sub-case and merge into parent case variables
            let sub_case = engine.get_case(sub_case_id).await?;
            // FUTURE: Merge sub_case.data into parent case variables
        }
        crate::parser::TaskType::MultipleInstance => {
            // Multiple instance task: Execute multiple instances
            // Determine instance count (from task properties or variables)
            let instance_count = 1; // FUTURE: Extract from task properties or case variables

            // Execute instances in parallel (or sequentially based on task configuration)
            for i in 0..instance_count {
                let instance_case_id = engine
                    .create_case(spec_id, serde_json::json!({ "instance": i }))
                    .await?;
                engine.start_case(instance_case_id).await?;

                // Execute instance workflow directly to avoid recursion
                let specs = engine.specs.read().await;
                let instance_spec = specs.get(&spec_id).ok_or_else(|| {
                    WorkflowError::InvalidSpecification(format!("Workflow {} not found", spec_id))
                })?;
                let instance_spec_clone = instance_spec.clone();
                drop(specs);

                super::task::execute_workflow_tasks(engine, instance_case_id, &instance_spec_clone)
                    .await?;

                // Collect results from instances
                let instance_case = engine.get_case(instance_case_id).await?;
                // FUTURE: Aggregate instance results
            }
        }
    }

    // Check max_ticks constraint after execution
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
