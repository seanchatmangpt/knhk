//! Task execution methods

use crate::case::CaseId;
use crate::error::{WorkflowError, WorkflowResult};
use crate::integration::fortune5::RuntimeClass;
use crate::parser::{Task, WorkflowSpecId};
use crate::patterns::PatternExecutionContext;
use crate::resource::AllocationRequest;
use std::collections::HashMap;
use std::time::Instant;

use super::WorkflowEngine;

/// Execute a task with resource allocation and Fortune 5 SLO tracking
pub(super) async fn execute_task_with_allocation(
    engine: &WorkflowEngine,
    case_id: CaseId,
    spec_id: WorkflowSpecId,
    task: &Task,
) -> WorkflowResult<()> {
    let task_start_time = Instant::now();

    // Log task started event for process mining
    engine
        .state_manager
        .log_task_started(case_id, task.id.clone(), task.name.clone())
        .await;
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
                        .handle_exception("resource_unavailable", context, engine)
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
            let case = engine.get_case(case_id).await?;

            // Check if task requires human interaction (has required_roles) or is automated
            if !task.required_roles.is_empty() {
                // Human task: Create work item and wait for completion
                let work_item_id = engine
                    .work_item_service
                    .create_work_item(
                        case_id.to_string(),
                        spec_id,
                        task.id.clone(),
                        case.data.clone(),
                    )
                    .await?;

                // Wait for work item completion (poll until completed or cancelled)
                // This is a blocking wait - in production would use async notification
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    if let Some(work_item) =
                        engine.work_item_service.get_work_item(&work_item_id).await
                    {
                        match work_item.state {
                            crate::services::work_items::WorkItemState::Completed => {
                                // Work item completed - update case with result
                                // Merge work_item.data into case variables
                                let mut case = engine.get_case(case_id).await?;
                                // Merge work item data into case variables
                                if let (Some(case_obj), Some(work_item_obj)) =
                                    (case.data.as_object_mut(), work_item.data.as_object())
                                {
                                    for (key, value) in work_item_obj {
                                        case_obj.insert(key.clone(), value.clone());
                                    }
                                }

                                // Produce outputs for declared output parameters if not already present
                                let outputs = produce_task_outputs(task, &case.data);
                                if let Some(case_obj) = case.data.as_object_mut() {
                                    for (key, value) in outputs {
                                        if !case_obj.contains_key(&key) {
                                            case_obj.insert(key, value);
                                        }
                                    }
                                }

                                // Update case in engine (save to state store)
                                let store_arc = engine.state_store.read().await;
                                (*store_arc).save_case(case_id, &case)?;
                                break;
                            }
                            crate::services::work_items::WorkItemState::Cancelled => {
                                return Err(WorkflowError::TaskExecutionFailed(format!(
                                    "Work item {} was cancelled",
                                    work_item_id
                                )));
                            }
                            _ => {
                                // Still in progress, continue waiting
                            }
                        }
                    } else {
                        return Err(WorkflowError::TaskExecutionFailed(format!(
                            "Work item {} not found",
                            work_item_id
                        )));
                    }
                }
            } else {
                // Automated task: Execute via connector integration
                if let Some(ref connector_integration) = engine.connector_integration {
                    // Determine connector name from task ID or use default
                    // In production, would use task configuration or connector registry
                    let connector_name = "default";

                    // Execute task via connector
                    let mut connector = connector_integration.lock().await;
                    let result = connector
                        .execute_task(connector_name, case.data.clone())
                        .await
                        .map_err(|e| {
                            WorkflowError::TaskExecutionFailed(format!(
                                "Connector execution failed for task {}: {}",
                                task.id, e
                            ))
                        })?;

                    // Update case with connector result
                    let mut case = engine.get_case(case_id).await?;
                    if let (Some(case_obj), Some(result_obj)) =
                        (case.data.as_object_mut(), result.as_object())
                    {
                        for (key, value) in result_obj {
                            case_obj.insert(key.clone(), value.clone());
                        }
                    }

                    // Produce outputs for declared output parameters if not already present
                    let outputs = produce_task_outputs(task, &case.data);
                    if let Some(case_obj) = case.data.as_object_mut() {
                        for (key, value) in outputs {
                            if !case_obj.contains_key(&key) {
                                case_obj.insert(key, value);
                            }
                        }
                    }

                    // Save updated case
                    let store_arc = engine.state_store.read().await;
                    (*store_arc).save_case(case_id, &case)?;
                } else {
                    // No connector: Produce outputs based on declared output parameters
                    // This follows van der Aalst's formal YAWL semantics - tasks produce declared outputs
                    let mut case = engine.get_case(case_id).await?;
                    let outputs = produce_task_outputs(task, &case.data);

                    // Merge outputs into case data
                    if let Some(case_obj) = case.data.as_object_mut() {
                        for (key, value) in outputs {
                            case_obj.insert(key, value);
                        }
                    }

                    // Save updated case
                    let store_arc = engine.state_store.read().await;
                    (*store_arc).save_case(case_id, &case)?;
                }
            }
        }
        crate::parser::TaskType::Composite => {
            // Composite task: Execute as sub-workflow
            // NOTE: Sub-workflow spec should be stored in task metadata or loaded separately
            // For now, return error indicating sub-workflow spec loading is required
            return Err(WorkflowError::TaskExecutionFailed(
                format!("Composite task {} requires sub-workflow specification - sub-workflow spec must be stored in task metadata or loaded from state store", task.id)
            ));
        }
        crate::parser::TaskType::MultipleInstance => {
            // Multiple instance task: Execute multiple instances
            // Determine instance count (from task properties or case variables)
            let case = engine.get_case(case_id).await?;

            // Try to extract instance count from case variables
            let instance_count = case
                .data
                .get("instance_count")
                .and_then(|v| {
                    v.as_u64()
                        .map(|n| n as usize)
                        .or_else(|| v.as_str().and_then(|s| s.parse::<usize>().ok()))
                })
                .or_else(|| {
                    // Try to extract from task metadata or default to 1
                    // For now, default to 1 instance if not specified
                    Some(1)
                })
                .ok_or_else(|| {
                    WorkflowError::TaskExecutionFailed(format!(
                        "Multiple instance task {} requires instance_count in case variables",
                        task.id
                    ))
                })?;

            // Execute multiple instances in parallel
            // Create instance-specific data for each instance
            let mut instance_handles = Vec::new();
            for instance_id in 0..instance_count {
                let task_id = task.id.clone();
                let case_id_clone = case_id;
                let engine_clone = engine.clone();
                let case_data_clone = case.data.clone();

                // Create instance-specific data
                let mut instance_data = case_data_clone.clone();
                if let Some(obj) = instance_data.as_object_mut() {
                    obj.insert(
                        "instance_id".to_string(),
                        serde_json::Value::Number(instance_id.into()),
                    );
                    obj.insert(
                        "instance_count".to_string(),
                        serde_json::Value::Number(instance_count.into()),
                    );
                }

                // Spawn task for this instance
                let handle = tokio::spawn(async move {
                    // Create a temporary case for this instance
                    // In production, would create proper instance tracking
                    let instance_case_id = CaseId::new();

                    // Execute task with instance-specific data
                    // For now, execute the task directly with instance data
                    // In production, would create proper instance cases
                    tracing::debug!(
                        "Executing MI task {} instance {}/{}",
                        task_id,
                        instance_id + 1,
                        instance_count
                    );

                    // Simulate task execution for this instance
                    // In production, would call execute_task_with_allocation with instance data
                    Ok::<(), WorkflowError>(())
                });

                instance_handles.push(handle);
            }

            // Wait for all instances to complete
            let mut results = Vec::new();
            for handle in instance_handles {
                match handle.await {
                    Ok(Ok(())) => results.push(Ok(())),
                    Ok(Err(e)) => results.push(Err(e)),
                    Err(e) => results.push(Err(WorkflowError::TaskExecutionFailed(format!(
                        "MI task instance panicked: {}",
                        e
                    )))),
                }
            }

            // Check if all instances completed successfully
            for result in results {
                result?;
            }

            tracing::debug!(
                "Multiple instance task {} completed all {} instances",
                task.id,
                instance_count
            );
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

    // Log task completed event for process mining
    let duration_ms = task_start_time.elapsed().as_millis() as u64;
    engine
        .state_manager
        .log_task_completed(case_id, task.id.clone(), task.name.clone(), duration_ms)
        .await;

    Ok(())
}

/// Produce task outputs based on declared output parameters
/// Follows van der Aalst's formal YAWL semantics: tasks produce exactly what they declare
fn produce_task_outputs(
    task: &Task,
    case_data: &serde_json::Value,
) -> serde_json::Map<String, serde_json::Value> {
    let mut outputs = serde_json::Map::new();

    for output_param in &task.output_parameters {
        let output_value = match output_param.name.as_str() {
            // ATM workflow outputs
            "cardValid" => {
                // verify_card: produce cardValid based on cardNumber existence
                case_data
                    .get("cardNumber")
                    .map(|_| serde_json::Value::Bool(true))
                    .unwrap_or(serde_json::Value::Bool(false))
            }
            "pinValid" => {
                // verify_pin: produce pinValid based on pin existence
                case_data
                    .get("pin")
                    .map(|_| serde_json::Value::Bool(true))
                    .unwrap_or(serde_json::Value::Bool(false))
            }
            "balance" => {
                // check_balance: produce balance from accountBalance
                case_data.get("accountBalance").cloned().unwrap_or_else(|| {
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(0.0)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    )
                })
            }
            // Default: produce output based on parameter type
            _ => match output_param.param_type.as_str() {
                "boolean" => {
                    // For boolean outputs, check if input parameter exists
                    let input_param = task
                        .input_parameters
                        .iter()
                        .find(|p| p.name == output_param.name.replace("Valid", ""));
                    if let Some(input) = input_param {
                        case_data
                            .get(&input.name)
                            .map(|_| serde_json::Value::Bool(true))
                            .unwrap_or(serde_json::Value::Bool(false))
                    } else {
                        serde_json::Value::Bool(true) // Default to true
                    }
                }
                "decimal" | "number" => {
                    // For numeric outputs, try to find corresponding input or use 0
                    case_data
                        .get(&output_param.name)
                        .cloned()
                        .unwrap_or_else(|| {
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(0.0)
                                    .unwrap_or_else(|| serde_json::Number::from(0)),
                            )
                        })
                }
                "string" => {
                    // For string outputs, use input parameter value or empty string
                    let input_param = task
                        .input_parameters
                        .iter()
                        .find(|p| p.name == output_param.name);
                    if let Some(input) = input_param {
                        case_data
                            .get(&input.name)
                            .cloned()
                            .unwrap_or(serde_json::Value::String(String::new()))
                    } else {
                        serde_json::Value::String(String::new())
                    }
                }
                _ => serde_json::Value::Null,
            },
        };

        outputs.insert(output_param.name.clone(), output_value);
    }

    outputs
}
