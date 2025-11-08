//! Self-validation tests
//!
//! Tests that use the workflow engine to validate itself - "eating our own dog food".

use knhk_workflow_engine::capabilities::{validate_capabilities, CapabilityRegistry};
use knhk_workflow_engine::case::{Case, CaseId, CaseState};
use knhk_workflow_engine::error::WorkflowResult;
use knhk_workflow_engine::executor::WorkflowEngine;
use knhk_workflow_engine::parser::WorkflowParser;
use knhk_workflow_engine::patterns::PatternRegistry;
use knhk_workflow_engine::self_validation::SelfValidationManager;
use knhk_workflow_engine::state::StateStore;
use tempfile::TempDir;

#[test]
fn test_engine_validates_itself() {
    // Arrange: Create self-validation manager
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let manager = SelfValidationManager::new(temp_dir.path())
        .expect("Failed to create self-validation manager");

    // Act: Validate capabilities using the engine
    let report = futures::executor::block_on(manager.validate_capabilities())
        .expect("Capability validation should succeed");

    // Assert: All required capabilities are available
    assert!(
        report.all_required_available(),
        "All required capabilities should be available"
    );
    assert_eq!(
        report.production_readiness(),
        100.0,
        "Production readiness should be 100%"
    );
}

#[test]
fn test_engine_creates_validation_case() {
    // Arrange: Create engine and validation manager
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let manager = SelfValidationManager::new(temp_dir.path())
        .expect("Failed to create self-validation manager");

    // Act: Validate engine using itself
    let engine_report = futures::executor::block_on(manager.validate_engine())
        .expect("Engine validation should succeed");

    // Assert: Validation case was created and executed
    assert_eq!(
        engine_report.case_state,
        CaseState::Active,
        "Validation case should be active"
    );
    assert!(
        engine_report.engine_operational,
        "Engine should be operational"
    );
    assert_eq!(
        engine_report.patterns_validated, 43,
        "All 43 patterns should be validated"
    );
}

#[test]
fn test_engine_parses_validation_workflow() {
    // Arrange: Create parser and manager
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let manager = SelfValidationManager::new(temp_dir.path())
        .expect("Failed to create self-validation manager");
    let mut parser = WorkflowParser::new().expect("Failed to create parser");

    // Act: Parse validation workflow
    let spec = manager
        .create_validation_spec()
        .expect("Failed to create spec");
    let parsed_spec = parser
        .parse_turtle(&spec)
        .expect("Failed to parse workflow");

    // Assert: Workflow was parsed successfully
    assert!(!parsed_spec.tasks.is_empty(), "Workflow should have tasks");
    assert!(
        parsed_spec.start_condition.is_some(),
        "Workflow should have start condition"
    );
    assert!(
        parsed_spec.end_condition.is_some(),
        "Workflow should have end condition"
    );
}

#[test]
fn test_engine_validates_pattern_registry() {
    // Arrange: Create manager
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let manager = SelfValidationManager::new(temp_dir.path())
        .expect("Failed to create self-validation manager");

    // Act: Validate pattern registry
    let result = manager.validate_pattern_registry();

    // Assert: Pattern registry is valid
    assert!(result.is_ok(), "Pattern registry should be valid");
}

#[test]
fn test_engine_runs_validation_workflow() {
    // Arrange: Create self-validation manager
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let manager = SelfValidationManager::new(temp_dir.path())
        .expect("Failed to create self-validation manager");

    // Act: Run complete validation workflow
    let report = futures::executor::block_on(manager.run_validation_workflow())
        .expect("Validation workflow should succeed");

    // Assert: Validation passed
    assert!(report.validation_passed, "Self-validation should pass");
    assert!(
        report.capability_report.all_required_available(),
        "All required capabilities should be available"
    );
    assert!(
        report.engine_report.engine_operational,
        "Engine should be operational"
    );
}

#[test]
fn test_engine_manages_own_lifecycle() {
    // Arrange: Create engine
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");
    let engine = WorkflowEngine::new(state_store);

    // Act: Create a case for self-validation
    let case = Case::new(
        knhk_workflow_engine::parser::WorkflowSpecId::new(),
        serde_json::json!({
            "validation_type": "lifecycle",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
    );
    let case_id = engine.create_case(case).expect("Failed to create case");

    // Assert: Case was created successfully
    let retrieved_case = engine.get_case(&case_id).expect("Failed to get case");
    assert_eq!(
        retrieved_case.state,
        CaseState::Active,
        "Case should be active"
    );
}

#[test]
fn test_engine_validates_all_patterns() {
    // Arrange: Create pattern registry and manager
    let registry = PatternRegistry::new();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let manager = SelfValidationManager::new(temp_dir.path())
        .expect("Failed to create self-validation manager");

    // Act: Validate all patterns
    let result = manager.validate_pattern_registry();

    // Assert: All 43 patterns are registered
    assert!(result.is_ok(), "All patterns should be registered");
    let patterns = registry.list_patterns();
    assert_eq!(patterns.len(), 43, "All 43 patterns should be registered");
}

#[test]
fn test_engine_uses_itself_for_validation() {
    // Arrange: Create self-validation manager
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let manager = SelfValidationManager::new(temp_dir.path())
        .expect("Failed to create self-validation manager");

    // Act: Use engine to validate itself
    let report = futures::executor::block_on(manager.validate_engine())
        .expect("Engine validation should succeed");

    // Assert: Engine successfully validated itself
    assert!(report.engine_operational, "Engine should be operational");
    assert!(
        report.capability_report.all_required_available(),
        "All required capabilities should be available"
    );
}
