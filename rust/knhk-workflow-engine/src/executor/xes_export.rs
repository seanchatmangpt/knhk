//! XES export methods for WorkflowEngine
//!
//! Provides process mining integration by exporting workflow execution logs
//! in IEEE XES format compatible with ProM.

use crate::case::CaseId;
use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpecId;
use crate::process_mining::{WorkflowEvent, XesExporter};

use super::WorkflowEngine;

impl WorkflowEngine {
    /// Export single case execution to XES format
    ///
    /// Converts case history from StateManager into XES-compatible event stream
    /// for process mining analysis in ProM.
    ///
    /// # Arguments
    /// * `case_id` - Case identifier to export
    ///
    /// # Returns
    /// XES XML string ready for ProM import
    pub async fn export_case_to_xes(&self, case_id: CaseId) -> WorkflowResult<String> {
        // Get case history from StateManager
        let events = self.state_manager.get_case_history(case_id).await;

        // Convert StateEvents to WorkflowEvents
        let xes_events: Vec<WorkflowEvent> = events
            .into_iter()
            .filter_map(XesExporter::state_event_to_workflow_event)
            .collect();

        // Export to XES XML
        Ok(XesExporter::export_case_log(&case_id, xes_events))
    }

    /// Export all cases for a workflow to XES format
    ///
    /// Generates XES log with multiple traces, one per case.
    ///
    /// # Arguments
    /// * `spec_id` - Workflow specification identifier
    ///
    /// # Returns
    /// XES XML string with all cases as separate traces
    pub async fn export_workflow_to_xes(&self, spec_id: WorkflowSpecId) -> WorkflowResult<String> {
        // Get all cases for this workflow
        let case_ids = self.list_cases(spec_id).await?;

        // Collect events for each case
        let mut all_cases = Vec::new();
        for case_id in case_ids {
            let events = self.state_manager.get_case_history(case_id).await;
            let xes_events: Vec<WorkflowEvent> = events
                .into_iter()
                .filter_map(XesExporter::state_event_to_workflow_event)
                .collect();

            all_cases.push((case_id, xes_events));
        }

        // Export all cases to XES
        Ok(XesExporter::export_multiple_cases(all_cases))
    }

    /// Export all cases (across all workflows) to XES format
    ///
    /// Generates comprehensive XES log for organization-wide process mining.
    ///
    /// # Returns
    /// XES XML string with all cases from all workflows
    pub async fn export_all_cases_to_xes(&self) -> WorkflowResult<String> {
        // Get all cases from in-memory cache
        let cases = self.cases();

        // Collect events for each case
        let mut all_cases = Vec::new();
        for entry in cases.iter() {
            let case_id = *entry.key();
            let events = self.state_manager.get_case_history(case_id).await;
            let xes_events: Vec<WorkflowEvent> = events
                .into_iter()
                .filter_map(XesExporter::state_event_to_workflow_event)
                .collect();

            all_cases.push((case_id, xes_events));
        }

        // Export all cases to XES
        Ok(XesExporter::export_multiple_cases(all_cases))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateStore;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_export_case_to_xes() {
        // Create test state store
        let temp_dir = TempDir::new().unwrap();
        let state_store = StateStore::new(temp_dir.path()).unwrap();
        let engine = WorkflowEngine::new(state_store);

        // Create simple workflow spec
        let spec = crate::parser::WorkflowSpec {
            id: crate::parser::WorkflowSpecId::new(),
            name: "test_workflow".to_string(),
            start_condition: None,
            end_condition: None,
            tasks: std::collections::HashMap::new(),
            conditions: std::collections::HashMap::new(),
            source_turtle: None,
        };

        // Register workflow
        engine.register_workflow(spec.clone()).await.unwrap();

        // Create case
        let case_id = engine
            .create_case(spec.id, serde_json::json!({}))
            .await
            .unwrap();

        // Export to XES
        let xes = engine.export_case_to_xes(case_id).await.unwrap();

        // Validate XES format
        assert!(xes.contains("<?xml version"));
        assert!(xes.contains("<log xes.version=\"2.0\""));
        assert!(xes.contains("<trace>"));
        assert!(xes.contains(&format!("concept:name\" value=\"{}\"", case_id)));
    }

    #[tokio::test]
    async fn test_export_workflow_to_xes() {
        let temp_dir = TempDir::new().unwrap();
        let state_store = StateStore::new(temp_dir.path()).unwrap();
        let engine = WorkflowEngine::new(state_store);

        // Create workflow
        let spec = crate::parser::WorkflowSpec {
            id: crate::parser::WorkflowSpecId::new(),
            name: "test_workflow".to_string(),
            start_condition: None,
            end_condition: None,
            tasks: std::collections::HashMap::new(),
            conditions: std::collections::HashMap::new(),
            source_turtle: None,
        };

        engine.register_workflow(spec.clone()).await.unwrap();

        // Create multiple cases
        let case1 = engine
            .create_case(spec.id, serde_json::json!({}))
            .await
            .unwrap();
        let case2 = engine
            .create_case(spec.id, serde_json::json!({}))
            .await
            .unwrap();

        // Export workflow to XES
        let xes = engine.export_workflow_to_xes(spec.id).await.unwrap();

        // Validate multiple traces
        assert_eq!(xes.matches("<trace>").count(), 2);
        assert!(xes.contains(&case1.to_string()));
        assert!(xes.contains(&case2.to_string()));
    }
}
