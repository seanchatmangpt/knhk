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

    /// Import XES log file
    ///
    /// Loads workflow execution history from XES format (ProM export).
    /// Parses XES content and returns the number of traces (cases) found.
    ///
    /// **80/20 Focus:**
    /// - Parse XES format correctly
    /// - Extract trace count (cases)
    /// - Validate XES structure
    ///
    /// **Note:**
    /// Full import would create workflow specs from event log structure
    /// and replay cases from traces. Currently returns trace count only.
    ///
    /// # Arguments
    /// * `xes_content` - XES XML string to import
    ///
    /// # Returns
    /// Number of traces (cases) found in XES log
    ///
    /// # Errors
    /// Returns `WorkflowError::Internal` if XES parsing fails
    pub async fn import_xes(&self, xes_content: &str) -> WorkflowResult<usize> {
        use process_mining::{import_xes_file, XESImportOptions};

        // Validate XES content is not empty
        if xes_content.trim().is_empty() {
            return Err(WorkflowError::Internal("XES content is empty".to_string()));
        }

        // Write XES content to temporary file (import_xes_file expects a file path)
        let temp_dir = tempfile::tempdir().map_err(|e| {
            WorkflowError::Internal(format!("Failed to create temp directory: {}", e))
        })?;
        let temp_file = temp_dir.path().join("import.xes");
        std::fs::write(&temp_file, xes_content).map_err(|e| {
            WorkflowError::Internal(format!("Failed to write temp XES file: {}", e))
        })?;

        // Import XES file using process_mining crate
        let event_log = import_xes_file(&temp_file, XESImportOptions::default())
            .map_err(|e| WorkflowError::Internal(format!("Failed to import XES: {:?}", e)))?;

        // Validate event log structure
        let trace_count = event_log.traces.len();
        if trace_count == 0 {
            return Err(WorkflowError::Internal(
                "XES log contains no traces".to_string(),
            ));
        }

        Ok(trace_count)
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
            tasks: std::collections::HashMap::new(),
            conditions: std::collections::HashMap::new(),
            flows: Vec::new(),
            start_condition: None,
            end_condition: None,
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
            tasks: std::collections::HashMap::new(),
            conditions: std::collections::HashMap::new(),
            flows: Vec::new(),
            start_condition: None,
            end_condition: None,
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

    #[tokio::test]
    async fn test_import_xes_validates_empty_content() {
        let temp_dir = TempDir::new().unwrap();
        let state_store = StateStore::new(temp_dir.path()).unwrap();
        let engine = WorkflowEngine::new(state_store);

        // Test empty XES content
        let result = engine.import_xes("").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[tokio::test]
    async fn test_import_xes_validates_structure() {
        let temp_dir = TempDir::new().unwrap();
        let state_store = StateStore::new(temp_dir.path()).unwrap();
        let engine = WorkflowEngine::new(state_store);

        // Create valid XES content
        let spec = crate::parser::WorkflowSpec {
            id: crate::parser::WorkflowSpecId::new(),
            name: "test_workflow".to_string(),
            tasks: std::collections::HashMap::new(),
            conditions: std::collections::HashMap::new(),
            flows: Vec::new(),
            start_condition: None,
            end_condition: None,
            source_turtle: None,
        };

        engine.register_workflow(spec.clone()).await.unwrap();
        let case_id = engine
            .create_case(spec.id, serde_json::json!({}))
            .await
            .unwrap();

        // Export to XES
        let xes_content = engine.export_case_to_xes(case_id).await.unwrap();

        // Import should succeed
        let imported = engine.import_xes(&xes_content).await.unwrap();
        assert_eq!(imported, 1);
    }
}
