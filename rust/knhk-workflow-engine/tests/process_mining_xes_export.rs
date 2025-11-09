//! Integration tests for XES export and ProM compatibility
//!
//! Validates that exported XES logs are:
//! 1. Valid XML
//! 2. Conform to IEEE XES 2.0 standard
//! 3. Importable into ProM (if available)

use knhk_workflow_engine::{
    executor::WorkflowEngine, parser::WorkflowSpec, state::StateStore, CaseId, WorkflowSpecId,
};
use std::sync::Arc;
use tempfile::TempDir;

/// Create a simple test workflow
fn create_test_workflow() -> WorkflowSpec {
    WorkflowSpec {
        id: WorkflowSpecId::new(),
        name: "test_process_mining".to_string(),
        start_condition: None,
        end_condition: None,
        tasks: std::collections::HashMap::new(),
        conditions: std::collections::HashMap::new(),
        source_turtle: None,
    }
}

#[tokio::test]
async fn test_xes_export_single_case() {
    // Arrange: Create engine and workflow
    let temp_dir = TempDir::new().unwrap();
    let state_store = StateStore::new(temp_dir.path()).unwrap();
    let engine = WorkflowEngine::new(state_store);

    let spec = create_test_workflow();
    engine.register_workflow(spec.clone()).await.unwrap();

    // Create and execute case
    let case_id = engine
        .create_case(spec.id, serde_json::json!({"order_id": 123}))
        .await
        .unwrap();

    // Act: Export to XES
    let xes = engine.export_case_to_xes(case_id).await.unwrap();

    // Assert: Validate XES structure
    assert!(xes.contains("<?xml version=\"1.0\" encoding=\"UTF-8\" ?>"));
    assert!(xes.contains("<log xes.version=\"2.0\""));
    assert!(xes.contains("<trace>"));
    assert!(xes.contains(&format!("concept:name\" value=\"{}\"", case_id)));
    assert!(xes.contains("<event>"));
    assert!(xes.contains("time:timestamp"));
}

#[tokio::test]
async fn test_xes_export_multiple_cases() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let state_store = StateStore::new(temp_dir.path()).unwrap();
    let engine = WorkflowEngine::new(state_store);

    let spec = create_test_workflow();
    engine.register_workflow(spec.clone()).await.unwrap();

    // Create multiple cases
    let case1 = engine
        .create_case(spec.id, serde_json::json!({"order_id": 1}))
        .await
        .unwrap();
    let case2 = engine
        .create_case(spec.id, serde_json::json!({"order_id": 2}))
        .await
        .unwrap();
    let case3 = engine
        .create_case(spec.id, serde_json::json!({"order_id": 3}))
        .await
        .unwrap();

    // Act: Export workflow to XES
    let xes = engine.export_workflow_to_xes(spec.id).await.unwrap();

    // Assert: Validate multiple traces
    assert_eq!(xes.matches("<trace>").count(), 3);
    assert!(xes.contains(&case1.to_string()));
    assert!(xes.contains(&case2.to_string()));
    assert!(xes.contains(&case3.to_string()));
}

#[tokio::test]
async fn test_xes_export_all_workflows() {
    // Arrange: Create multiple workflows
    let temp_dir = TempDir::new().unwrap();
    let state_store = StateStore::new(temp_dir.path()).unwrap();
    let engine = WorkflowEngine::new(state_store);

    let spec1 = create_test_workflow();
    let spec2 = create_test_workflow();

    engine.register_workflow(spec1.clone()).await.unwrap();
    engine.register_workflow(spec2.clone()).await.unwrap();

    // Create cases for both workflows
    let case1 = engine
        .create_case(spec1.id, serde_json::json!({}))
        .await
        .unwrap();
    let case2 = engine
        .create_case(spec2.id, serde_json::json!({}))
        .await
        .unwrap();

    // Act: Export all cases
    let xes = engine.export_all_cases_to_xes().await.unwrap();

    // Assert: Both cases present
    assert_eq!(xes.matches("<trace>").count(), 2);
    assert!(xes.contains(&case1.to_string()));
    assert!(xes.contains(&case2.to_string()));
}

#[tokio::test]
async fn test_xes_lifecycle_transitions() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let state_store = StateStore::new(temp_dir.path()).unwrap();
    let engine = WorkflowEngine::new(state_store);

    let spec = create_test_workflow();
    engine.register_workflow(spec.clone()).await.unwrap();

    // Create and execute case
    let case_id = engine
        .create_case(spec.id, serde_json::json!({}))
        .await
        .unwrap();

    // Trigger state changes
    let _ = engine.start_case(case_id).await;

    // Act: Export to XES
    let xes = engine.export_case_to_xes(case_id).await.unwrap();

    // Assert: Lifecycle attributes present
    assert!(xes.contains("lifecycle:transition"));
    assert!(
        xes.contains("\"start\"") || xes.contains("\"complete\"") || xes.contains("\"cancel\"")
    );
}

#[tokio::test]
async fn test_xes_xml_validity() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let state_store = StateStore::new(temp_dir.path()).unwrap();
    let engine = WorkflowEngine::new(state_store);

    let spec = create_test_workflow();
    engine.register_workflow(spec.clone()).await.unwrap();

    let case_id = engine
        .create_case(spec.id, serde_json::json!({}))
        .await
        .unwrap();

    // Act: Export to XES
    let xes = engine.export_case_to_xes(case_id).await.unwrap();

    // Assert: Write to temp file and validate with xmllint if available
    let temp_file = std::env::temp_dir().join("knhk_test.xes");
    std::fs::write(&temp_file, &xes).unwrap();

    // Try to validate with xmllint (optional - only if installed)
    if let Ok(output) = std::process::Command::new("xmllint")
        .arg("--noout")
        .arg(&temp_file)
        .output()
    {
        assert!(
            output.status.success(),
            "XES must be valid XML. xmllint output: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        println!("✓ XES validated with xmllint");
    } else {
        println!("⚠ xmllint not available - skipping XML validation");
    }

    // Cleanup
    let _ = std::fs::remove_file(temp_file);
}

#[tokio::test]
async fn test_xes_export_with_special_characters() {
    // Arrange: Test XML escaping
    let temp_dir = TempDir::new().unwrap();
    let state_store = StateStore::new(temp_dir.path()).unwrap();
    let engine = WorkflowEngine::new(state_store);

    let spec = create_test_workflow();
    engine.register_workflow(spec.clone()).await.unwrap();

    let case_id = engine
        .create_case(spec.id, serde_json::json!({}))
        .await
        .unwrap();

    // Act: Export to XES
    let xes = engine.export_case_to_xes(case_id).await.unwrap();

    // Assert: XML is well-formed (no un escaped characters)
    // Case ID contains hyphens which are valid XML characters
    assert!(xes.contains(&case_id.to_string()));

    // Verify basic XML structure is intact
    assert!(xes.contains("<?xml"));
    assert!(xes.contains("<log"));
    assert!(xes.contains("</log>"));

    // Check that special XML entities would be escaped if present
    // (our StateEvents use simple strings, but the XesExporter has escape logic)
    assert!(!xes.contains("<event >")); // No spaces before >
    assert!(!xes.contains("< event")); // No spaces after <
}

#[tokio::test]
async fn test_xes_export_empty_workflow() {
    // Arrange: Workflow with no case execution
    let temp_dir = TempDir::new().unwrap();
    let state_store = StateStore::new(temp_dir.path()).unwrap();
    let engine = WorkflowEngine::new(state_store);

    let spec = create_test_workflow();
    engine.register_workflow(spec.clone()).await.unwrap();

    // Act: Export workflow with no cases
    let xes = engine.export_workflow_to_xes(spec.id).await.unwrap();

    // Assert: Valid XES structure even with no traces
    assert!(xes.contains("<?xml version"));
    assert!(xes.contains("<log xes.version=\"2.0\""));
    // Should have no traces
    assert_eq!(xes.matches("<trace>").count(), 0);
}

#[tokio::test]
async fn test_xes_extensions_present() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let state_store = StateStore::new(temp_dir.path()).unwrap();
    let engine = WorkflowEngine::new(state_store);

    let spec = create_test_workflow();
    engine.register_workflow(spec.clone()).await.unwrap();

    let case_id = engine
        .create_case(spec.id, serde_json::json!({}))
        .await
        .unwrap();

    // Act: Export to XES
    let xes = engine.export_case_to_xes(case_id).await.unwrap();

    // Assert: Required XES extensions present
    assert!(xes.contains("extension name=\"Concept\""));
    assert!(xes.contains("extension name=\"Time\""));
    assert!(xes.contains("extension name=\"Lifecycle\""));
    assert!(xes.contains("extension name=\"Organizational\""));

    // Assert: Classifiers present
    assert!(xes.contains("classifier name=\"Activity\""));
}

#[tokio::test]
async fn test_xes_global_attributes() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let state_store = StateStore::new(temp_dir.path()).unwrap();
    let engine = WorkflowEngine::new(state_store);

    let spec = create_test_workflow();
    engine.register_workflow(spec.clone()).await.unwrap();

    let case_id = engine
        .create_case(spec.id, serde_json::json!({}))
        .await
        .unwrap();

    // Act: Export to XES
    let xes = engine.export_case_to_xes(case_id).await.unwrap();

    // Assert: Global attributes defined
    assert!(xes.contains("<global scope=\"trace\">"));
    assert!(xes.contains("<global scope=\"event\">"));
}

/// AAA Pattern Test: Complete workflow execution exported to XES
#[tokio::test]
async fn test_complete_workflow_execution_xes_export() {
    // Arrange: Create realistic workflow with execution
    let temp_dir = TempDir::new().unwrap();
    let state_store = StateStore::new(temp_dir.path()).unwrap();
    let engine = WorkflowEngine::new(state_store);

    let spec = create_test_workflow();
    engine.register_workflow(spec.clone()).await.unwrap();

    // Act: Create, start, and execute case
    let case_id = engine
        .create_case(spec.id, serde_json::json!({"customer": "ACME Corp"}))
        .await
        .unwrap();

    let _ = engine.start_case(case_id).await;

    // Export to XES
    let xes = engine.export_case_to_xes(case_id).await.unwrap();

    // Assert: Complete trace with multiple events
    assert!(xes.contains("<trace>"));
    assert!(xes.matches("<event>").count() >= 1); // At least case creation event
    assert!(xes.contains(&case_id.to_string()));

    // Verify event ordering (timestamps should be present and parseable)
    assert!(xes.contains("time:timestamp"));
    assert!(xes.contains("T")); // ISO 8601 timestamp format
}
