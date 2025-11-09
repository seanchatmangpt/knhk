//! Simple workflow execution for testing
//!
//! This module provides a simplified execution engine for tests that:
//! - Follows workflow flows from start to end
//! - Transitions case states appropriately
//! - Doesn't require actual task implementation (mocked for tests)

use super::WorkflowEngine;
use crate::case::{CaseId, CaseState};
use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;
use std::collections::HashSet;
use tracing::info;

/// Evaluate a simple predicate against case data
/// Currently supports: "variable == value" format
fn evaluate_predicate(predicate: &str, case_data: &serde_json::Value) -> bool {
    let predicate = predicate.trim();

    // Handle "approved == true" pattern
    if let Some(eq_pos) = predicate.find("==") {
        let var_name = predicate[..eq_pos].trim();
        let expected_value = predicate[eq_pos + 2..].trim();

        // Get value from case data
        if let Some(actual_value) = case_data.get(var_name) {
            // Compare values
            match expected_value {
                "true" => actual_value.as_bool() == Some(true),
                "false" => actual_value.as_bool() == Some(false),
                _ => {
                    // Try string comparison
                    if let Some(actual_str) = actual_value.as_str() {
                        actual_str == expected_value.trim_matches('"')
                    } else {
                        // Try numeric comparison
                        if let (Some(actual_num), Ok(expected_num)) =
                            (actual_value.as_f64(), expected_value.parse::<f64>())
                        {
                            (actual_num - expected_num).abs() < f64::EPSILON
                        } else {
                            false
                        }
                    }
                }
            }
        } else {
            // Variable not found - predicate fails
            false
        }
    } else {
        // Unknown predicate format - default to false for safety
        eprintln!("  ⚠️  Unknown predicate format: {}", predicate);
        false
    }
}

impl WorkflowEngine {
    /// Simple workflow execution that follows flows from start to end
    /// This is used for tests and simple workflows where tasks don't require
    /// actual execution (automated or already complete)
    pub(super) async fn execute_simple_workflow(
        &self,
        case_id: CaseId,
        spec: &WorkflowSpec,
    ) -> WorkflowResult<()> {
        eprintln!("=== SIMPLE WORKFLOW EXECUTION START ===");
        eprintln!("Case ID: {}", case_id);
        eprintln!("Workflow: {}", spec.name);
        eprintln!("Tasks: {}", spec.tasks.len());
        eprintln!("Conditions: {}", spec.conditions.len());

        // Get start condition
        let start_condition_id = spec
            .start_condition
            .as_ref()
            .ok_or_else(|| WorkflowError::InvalidSpecification("No start condition".into()))?;
        eprintln!("Start condition: {}", start_condition_id);

        let end_condition_id = spec
            .end_condition
            .as_ref()
            .ok_or_else(|| WorkflowError::InvalidSpecification("No end condition".into()))?;
        eprintln!("End condition: {}", end_condition_id);

        // Track visited nodes to prevent infinite loops
        let mut visited = HashSet::new();
        let mut current_nodes: Vec<String> = vec![start_condition_id.clone()];
        let mut iteration = 0;

        // Follow the workflow flows
        while !current_nodes.is_empty() {
            iteration += 1;
            eprintln!("\n--- Iteration {} ---", iteration);
            eprintln!("Current nodes: {:?}", current_nodes);

            let mut next_nodes = Vec::new();

            for node_id in &current_nodes {
                // Check if we've visited this node
                if !visited.insert(node_id.clone()) {
                    eprintln!("Skipping already visited: {}", node_id);
                    continue; // Skip already visited nodes
                }

                eprintln!("Visiting: {}", node_id);

                // Check if we've reached the end condition
                if node_id == end_condition_id {
                    eprintln!("✅ Reached end condition! Completing case.");

                    // Mark case as completed
                    let mut case_ref = self
                        .cases
                        .get_mut(&case_id)
                        .ok_or_else(|| WorkflowError::CaseNotFound(case_id.to_string()))?;

                    case_ref.value_mut().state = CaseState::Completed;
                    let case_clone = case_ref.value().clone();
                    drop(case_ref);

                    // Persist state
                    let store_arc = self.state_store.read().await;
                    (*store_arc).save_case(case_id, &case_clone)?;

                    // Save to state manager
                    self.state_manager.save_case(&case_clone).await?;

                    eprintln!("=== WORKFLOW COMPLETED ===\n");
                    return Ok(());
                }

                // Find outgoing flows from this node
                let outgoing_targets = if let Some(task) = spec.tasks.get(node_id) {
                    eprintln!("  Found task: {}", task.name);
                    eprintln!("  Outgoing flows: {:?}", task.outgoing_flows);
                    task.outgoing_flows.clone()
                } else if let Some(condition) = spec.conditions.get(node_id) {
                    eprintln!("  Found condition: {}", condition.name);
                    eprintln!("  Outgoing flows: {:?}", condition.outgoing_flows);
                    condition.outgoing_flows.clone()
                } else {
                    eprintln!("  ⚠️  Node not found in tasks or conditions!");
                    Vec::new()
                };

                // Evaluate predicates on flows
                for target_id in &outgoing_targets {
                    // Find the flow object with predicate
                    let flow = spec
                        .flows
                        .iter()
                        .find(|f| &f.from == node_id && &f.to == target_id);

                    if let Some(flow) = flow {
                        if let Some(predicate) = &flow.predicate {
                            eprintln!(
                                "  Checking predicate for flow to {}: {}",
                                target_id, predicate
                            );

                            // Get case data
                            let case_ref = self
                                .cases
                                .get(&case_id)
                                .ok_or_else(|| WorkflowError::CaseNotFound(case_id.to_string()))?;
                            let case_data = &case_ref.value().data;

                            // Evaluate predicate (simplified for "approved == true")
                            let predicate_satisfied = evaluate_predicate(predicate, case_data);

                            eprintln!("  Predicate satisfied: {}", predicate_satisfied);

                            if predicate_satisfied {
                                next_nodes.push(target_id.clone());
                            } else {
                                eprintln!("  ⛔ Predicate NOT satisfied - blocking flow");
                            }
                        } else {
                            // No predicate - flow is allowed
                            eprintln!("  No predicate - flow allowed to {}", target_id);
                            next_nodes.push(target_id.clone());
                        }
                    } else {
                        // No Flow object found - allow for backward compatibility
                        eprintln!("  No Flow object found - allowing flow to {}", target_id);
                        next_nodes.push(target_id.clone());
                    }
                }
            }

            current_nodes = next_nodes;

            if iteration > 100 {
                eprintln!("⚠️  Breaking after 100 iterations (possible cycle)");
                break;
            }
        }

        eprintln!("=== WORKFLOW DID NOT COMPLETE ===");
        eprintln!("Final state: Running (did not reach end condition)\n");
        // If we didn't reach the end condition, leave the case as Running
        Ok(())
    }
}
