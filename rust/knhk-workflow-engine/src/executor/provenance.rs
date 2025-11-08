//! Provenance receipt generation for workflow execution
//!
//! Integrates A = μ(O) and hash(A) = hash(μ(O)) into workflow execution,
//! generating receipts that prove every assertion is cryptographically linked to its operation.

use crate::case::{Case, CaseId};
use crate::compliance::provenance_law::{
    hash_actions, hash_mu_o, WorkflowAction, WorkflowObservations, WorkflowReceipt,
};
use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpecId;
use std::collections::HashMap;

use super::WorkflowEngine;

impl WorkflowEngine {
    /// Generate a receipt proving hash(A) = hash(μ(O)) for a workflow execution step
    ///
    /// Implements the foundational law: **A = μ(O)**
    /// and provenance law: **hash(A) = hash(μ(O))**
    pub fn generate_receipt(
        &self,
        case_id: CaseId,
        workflow_id: WorkflowSpecId,
        actions: &[WorkflowAction],
        observations: &WorkflowObservations,
    ) -> WorkflowResult<WorkflowReceipt> {
        // Compute hash(A) from actions
        let a_hash = hash_actions(actions);

        // Compute hash(μ(O)) from observations
        let mu_hash = hash_mu_o(observations);

        // Verify provenance law: hash(A) = hash(μ(O))
        if a_hash != mu_hash {
            return Err(WorkflowError::Validation(format!(
                "Provenance violation: hash(A)={} != hash(μ(O))={}",
                a_hash, mu_hash
            )));
        }

        // Generate receipt ID
        let receipt_id = format!("receipt_{}_{}", case_id, workflow_id);

        // Create receipt
        let receipt = WorkflowReceipt::new(
            receipt_id,
            case_id.to_string(),
            workflow_id.to_string(),
            a_hash,
            mu_hash,
        );

        Ok(receipt)
    }

    /// Extract workflow observations (O) from a case
    ///
    /// This represents the knowledge state after applying the reflex map μ.
    pub fn extract_observations(case: &Case) -> WorkflowObservations {
        let task_states: HashMap<String, String> = case
            .task_states
            .iter()
            .map(|(k, v)| (k.clone(), format!("{:?}", v)))
            .collect();

        let variables: HashMap<String, serde_json::Value> = if let Some(obj) = case.data.as_object()
        {
            obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        } else {
            HashMap::new()
        };

        WorkflowObservations {
            case_id: case.id.to_string(),
            workflow_id: case.spec_id.to_string(),
            case_state: format!("{:?}", case.state),
            task_states,
            variables,
        }
    }

    /// Extract workflow actions (A) from pattern execution result
    ///
    /// Actions are the assertions/state changes produced by workflow execution.
    pub fn extract_actions(
        pattern_result: &crate::patterns::PatternExecutionResult,
    ) -> Vec<WorkflowAction> {
        let mut actions = Vec::new();

        // State change action
        if let Some(ref next_state) = pattern_result.next_state {
            actions.push(WorkflowAction {
                action_type: "state_change".to_string(),
                payload: Some(serde_json::json!({"next_state": next_state})),
                variables: None,
            });
        }

        // Variable update action
        if !pattern_result.variables.is_empty() {
            let variables: HashMap<String, serde_json::Value> = pattern_result
                .variables
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();

            actions.push(WorkflowAction {
                action_type: "variable_update".to_string(),
                payload: None,
                variables: Some(variables),
            });
        }

        // Activity scheduling action
        if !pattern_result.next_activities.is_empty() {
            actions.push(WorkflowAction {
                action_type: "schedule_activities".to_string(),
                payload: Some(serde_json::json!({
                    "activities": pattern_result.next_activities
                })),
                variables: None,
            });
        }

        // Cancellation action
        if !pattern_result.cancel_activities.is_empty() {
            actions.push(WorkflowAction {
                action_type: "cancel_activities".to_string(),
                payload: Some(serde_json::json!({
                    "activities": pattern_result.cancel_activities
                })),
                variables: None,
            });
        }

        // Termination action
        if pattern_result.terminates {
            actions.push(WorkflowAction {
                action_type: "terminate".to_string(),
                payload: Some(serde_json::json!({"terminates": true})),
                variables: None,
            });
        }

        actions
    }
}
