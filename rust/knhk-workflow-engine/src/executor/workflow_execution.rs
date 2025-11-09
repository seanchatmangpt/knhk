//! Real workflow execution engine
//!
//! Follows workflow flows from start condition to end condition,
//! executing tasks and handling patterns (parallel split, synchronization, XOR choice, etc.)

use crate::case::{CaseId, CaseState};
use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::{Flow, WorkflowSpec};
use std::collections::{HashMap, HashSet, VecDeque};

use super::task::execute_task_with_allocation;
use super::WorkflowEngine;

/// Evaluate a predicate against case data
/// Supports: "variable == value", "variable >= value", "variable <= value" formats
fn evaluate_predicate(predicate: &str, case_data: &serde_json::Value) -> bool {
    let predicate = predicate.trim();

    // Handle "variable >= value" pattern (e.g., "balance >= withdrawalAmount")
    if let Some(ge_pos) = predicate.find(">=") {
        let left_var = predicate[..ge_pos].trim();
        let right_var = predicate[ge_pos + 2..].trim();

        let left_value = case_data.get(left_var).and_then(|v| v.as_f64());
        let right_value = case_data.get(right_var).and_then(|v| v.as_f64());

        if let (Some(left), Some(right)) = (left_value, right_value) {
            return left >= right;
        }
        return false;
    }

    // Handle "variable <= value" pattern
    if let Some(le_pos) = predicate.find("<=") {
        let left_var = predicate[..le_pos].trim();
        let right_var = predicate[le_pos + 2..].trim();

        let left_value = case_data.get(left_var).and_then(|v| v.as_f64());
        let right_value = case_data.get(right_var).and_then(|v| v.as_f64());

        if let (Some(left), Some(right)) = (left_value, right_value) {
            return left <= right;
        }
        return false;
    }

    // Handle "variable == value" pattern
    if let Some(eq_pos) = predicate.find("==") {
        let var_name = predicate[..eq_pos].trim();
        let expected_value = predicate[eq_pos + 2..].trim();

        if let Some(actual_value) = case_data.get(var_name) {
            match expected_value {
                "true" => actual_value.as_bool() == Some(true),
                "false" => actual_value.as_bool() == Some(false),
                _ => {
                    if let Some(actual_str) = actual_value.as_str() {
                        actual_str == expected_value.trim_matches('"')
                    } else if let (Some(actual_num), Ok(expected_num)) =
                        (actual_value.as_f64(), expected_value.parse::<f64>())
                    {
                        (actual_num - expected_num).abs() < f64::EPSILON
                    } else {
                        false
                    }
                }
            }
        } else {
            false
        }
    } else {
        false
    }
}

/// Execute a workflow from start to end condition
pub(super) fn execute_workflow<'a>(
    engine: &'a WorkflowEngine,
    case_id: CaseId,
    spec: &'a WorkflowSpec,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = WorkflowResult<()>> + Send + 'a>> {
    Box::pin(async move {
        let start_condition_id = spec
            .start_condition
            .as_ref()
            .ok_or_else(|| WorkflowError::InvalidSpecification("No start condition".into()))?;

        let end_condition_id = spec
            .end_condition
            .as_ref()
            .ok_or_else(|| WorkflowError::InvalidSpecification("No end condition".into()))?;

        // Track visited nodes to prevent infinite loops
        let mut visited = HashSet::new();
        // Queue of nodes to process (BFS traversal)
        let mut queue: VecDeque<String> = VecDeque::new();
        queue.push_back(start_condition_id.clone());

        // Track how many incoming flows each task has received
        // For AND join: task needs all incoming flows
        // For XOR join: task needs only one incoming flow
        // For OR join: task needs all active incoming flows (all that were enabled by OR-split)
        let mut task_incoming_count: HashMap<String, usize> = HashMap::new();
        for (task_id, task) in &spec.tasks {
            let required_count = if task.incoming_flows.is_empty() {
                0 // Task with no incoming flows (starts from start condition) is ready immediately
            } else if matches!(task.join_type, crate::parser::JoinType::And) {
                task.incoming_flows.len() // AND join needs all
            } else if matches!(task.join_type, crate::parser::JoinType::Or) {
                // OR join: needs all active branches (will be calculated dynamically)
                // For now, we'll track which branches are active and wait for all of them
                task.incoming_flows.len() // Will be adjusted based on active branches
            } else {
                1 // XOR join needs at least one
            };
            task_incoming_count.insert(task_id.clone(), required_count);
        }

        // Track active branches for OR joins (which branches were enabled by OR-splits)
        let mut or_join_active_branches: HashMap<String, HashSet<String>> = HashMap::new();

        // Track how many incoming flows each task has actually received
        let mut task_received_count: HashMap<String, usize> = HashMap::new();
        for task_id in spec.tasks.keys() {
            task_received_count.insert(task_id.clone(), 0);
        }

        // Track completed tasks
        let mut completed_tasks = HashSet::new();

        while let Some(node_id) = queue.pop_front() {
            // Check if we've reached the end condition
            if node_id == *end_condition_id {
                // Mark case as completed
                let mut case_ref = engine
                    .cases
                    .get_mut(&case_id)
                    .ok_or_else(|| WorkflowError::CaseNotFound(case_id.to_string()))?;

                case_ref.value_mut().state = CaseState::Completed;
                let case_clone = case_ref.value().clone();
                drop(case_ref);

                // Persist state
                let store_arc = engine.state_store.read().await;
                (*store_arc).save_case(case_id, &case_clone)?;

                // Save to state manager
                engine.state_manager.save_case(&case_clone).await?;

                return Ok(());
            }

            // Skip if already visited (for conditions)
            if !visited.insert(node_id.clone()) {
                continue;
            }

            // Check if this is a task or condition
            if let Some(task) = spec.tasks.get(&node_id) {
                // Check if task is ready to execute
                if !completed_tasks.contains(&node_id) {
                    let required = task_incoming_count.get(&node_id).copied().unwrap_or(0);
                    let received = task_received_count.get(&node_id).copied().unwrap_or(0);

                    if required > 0 && received < required {
                        // Task not ready yet - needs more incoming flows
                        continue;
                    }
                } else {
                    // Task already completed - skip
                    continue;
                }

                // Execute the task
                execute_task_with_allocation(engine, case_id, spec.id, task).await?;
                completed_tasks.insert(node_id.clone());

                // Find outgoing flows from this task
                let outgoing_flows: Vec<&Flow> =
                    spec.flows.iter().filter(|f| f.from == node_id).collect();

                // Evaluate predicates and add enabled flows to queue
                let case = engine.get_case(case_id).await?;
                let case_data = &case.data;

                for flow in &outgoing_flows {
                    let flow_enabled = if let Some(predicate) = &flow.predicate {
                        evaluate_predicate(predicate, case_data)
                    } else {
                        true // No predicate - flow is always enabled
                    };

                    if flow_enabled {
                        let target_id = &flow.to;

                        // If target is a task, increment received count
                        if let Some(target_task) = spec.tasks.get(target_id) {
                            // For OR joins, track which branches are active
                            if matches!(target_task.join_type, crate::parser::JoinType::Or) {
                                let active = or_join_active_branches
                                    .entry(target_id.clone())
                                    .or_insert_with(HashSet::new);
                                active.insert(node_id.clone()); // Mark this branch as active

                                // Update required count for OR join: need all active branches
                                let required = active.len();
                                task_incoming_count.insert(target_id.clone(), required);
                            }

                            let received =
                                task_received_count.entry(target_id.clone()).or_insert(0);
                            *received += 1;

                            let required = task_incoming_count.get(target_id).copied().unwrap_or(0);
                            // Add to queue if ready (received enough incoming flows)
                            if *received >= required {
                                queue.push_back(target_id.clone());
                            }
                        } else {
                            // Target is a condition - add to queue
                            queue.push_back(target_id.clone());
                        }
                    }
                }

                // Handle parallel split (AND split) - all outgoing flows should be taken
                if matches!(task.split_type, crate::parser::SplitType::And) {
                    // For AND split, we already added all enabled flows above
                    // Continue processing
                }
            } else if let Some(_condition) = spec.conditions.get(&node_id) {
                // Condition - find outgoing flows
                let outgoing_flows: Vec<&Flow> =
                    spec.flows.iter().filter(|f| f.from == node_id).collect();

                // Get case data for predicate evaluation
                let case = engine.get_case(case_id).await?;
                let case_data = &case.data;

                for flow in &outgoing_flows {
                    let flow_enabled = if let Some(predicate) = &flow.predicate {
                        evaluate_predicate(predicate, case_data)
                    } else {
                        true // No predicate - flow is always enabled
                    };

                    if flow_enabled {
                        let target_id = &flow.to;

                        // If target is a task, increment received count
                        if let Some(target_task) = spec.tasks.get(target_id) {
                            // For OR joins, track which branches are active
                            if matches!(target_task.join_type, crate::parser::JoinType::Or) {
                                let active = or_join_active_branches
                                    .entry(target_id.clone())
                                    .or_insert_with(HashSet::new);
                                active.insert(node_id.clone()); // Mark this branch as active

                                // Update required count for OR join: need all active branches
                                let required = active.len();
                                task_incoming_count.insert(target_id.clone(), required);
                            }

                            let received =
                                task_received_count.entry(target_id.clone()).or_insert(0);
                            *received += 1;

                            let required = task_incoming_count.get(target_id).copied().unwrap_or(0);
                            // Add to queue if ready (received enough incoming flows)
                            if *received >= required {
                                queue.push_back(target_id.clone());
                            }
                        } else {
                            // Target is a condition - add to queue
                            queue.push_back(target_id.clone());
                        }
                    }
                }
            }

            // Safety: prevent infinite loops
            if visited.len() > 1000 {
                return Err(WorkflowError::Internal(
                    "Workflow execution exceeded maximum iterations (possible cycle)".to_string(),
                ));
            }
        }

        // If we didn't reach the end condition, the workflow didn't complete
        // This could be due to predicates blocking all paths
        let case = engine.get_case(case_id).await?;
        if case.state != CaseState::Completed {
            // Workflow didn't complete - leave in Running state
            // (could be waiting for external event, milestone, etc.)
        }

        Ok(())
    })
}
