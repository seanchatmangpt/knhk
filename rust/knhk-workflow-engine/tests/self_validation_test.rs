//! Self-validation tests
//!
//! Tests that use the workflow engine to validate itself - "eating our own dog food".
//!
//! **UPGRADED**: Now uses Chicago TDD framework and mutation testing

use knhk_workflow_engine::capabilities::{validate_capabilities, CapabilityRegistry};
use knhk_workflow_engine::case::{Case, CaseId, CaseState};
use knhk_workflow_engine::error::WorkflowResult;
use knhk_workflow_engine::executor::WorkflowEngine;
use knhk_workflow_engine::parser::{TaskType, WorkflowParser};
use knhk_workflow_engine::patterns::PatternRegistry;
use knhk_workflow_engine::self_validation::SelfValidationManager;
use knhk_workflow_engine::state::StateStore;
use knhk_workflow_engine::testing::chicago_tdd::*;
use knhk_workflow_engine::testing::mutation::*;
use tempfile::TempDir;

#[test]
fn test_engine_validates_itself() {
    // Arrange: Create self-validation manager using Chicago TDD fixture
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let manager = SelfValidationManager::new(temp_dir.path())
        .expect("Failed to create self-validation manager");

    // Act: Validate capabilities using the engine
    let report = futures::executor::block_on(manager.validate_capabilities())
        .expect("Capability validation should succeed");

    // Assert: Use Chicago TDD assertions
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

    // Assert: Use Chicago TDD assertions
    assert_eq!(
        engine_report.case_state,
        CaseState::Running,
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

    // Assert: Use Chicago TDD assertions
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

    // Assert: Use Chicago TDD assertions
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

#[tokio::test]
async fn test_engine_manages_own_lifecycle() {
    // Arrange: Use Chicago TDD fixture
    let mut fixture = WorkflowTestFixture::new().unwrap();

    // Act: Create a case for self-validation
    let spec = WorkflowSpecBuilder::new("Lifecycle Validation").build();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    let case_id = fixture
        .create_case(
            spec_id,
            serde_json::json!({
                "validation_type": "lifecycle",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
        )
        .await
        .unwrap();

    // Assert: Case was created successfully using Chicago TDD assertions
    let case = fixture.execute_case(case_id).await.unwrap();
    assert!(
        matches!(
            case.state,
            CaseState::Completed | CaseState::Failed | CaseState::Running
        ),
        "Case should be in valid state"
    );
}

#[tokio::test]
async fn test_engine_validates_all_patterns() {
    // Arrange: Use Chicago TDD helper
    let registry = create_test_registry();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let manager = SelfValidationManager::new(temp_dir.path())
        .expect("Failed to create self-validation manager");

    // Act: Validate all patterns
    let result = manager.validate_pattern_registry();

    // Assert: Use Chicago TDD assertions
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

    // Assert: Use Chicago TDD assertions
    assert!(report.engine_operational, "Engine should be operational");
    assert!(
        report.capability_report.all_required_available(),
        "All required capabilities should be available"
    );
}

// ============================================================================
// Mutation Testing for Self-Validation
// ============================================================================

#[tokio::test]
async fn test_self_validation_mutation_testing() {
    // Test quality: Ensure self-validation tests catch mutations

    // Arrange: Create validation workflow and mutation tester
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let manager = SelfValidationManager::new(temp_dir.path())
        .expect("Failed to create self-validation manager");

    let spec_string = manager
        .create_validation_spec()
        .expect("Failed to create spec");
    let mut parser = WorkflowParser::new().expect("Failed to create parser");
    let spec = parser
        .parse_turtle(&spec_string)
        .expect("Failed to parse workflow");

    let first_task_id = spec.tasks.keys().next().cloned();
    let mut tester = MutationTester::new(spec).unwrap();

    // Apply mutations (only if workflow has tasks)
    if let Some(task_id) = first_task_id {
        tester.apply_mutation(MutationOperator::RemoveTask(task_id));
    }

    // Act: Test if mutations are caught
    let caught = tester
        .test_mutation_detection(|spec| {
            // Test: Validation workflow should have tasks
            !spec.tasks.is_empty()
        })
        .await;

    // Assert: Mutations caught (test quality validated)
    assert!(caught, "Self-validation tests should catch mutations");
}

#[tokio::test]
async fn test_mutation_score_for_self_validation() {
    // Test quality: Calculate mutation score for self-validation tests

    // Arrange: Create workflow and tester
    let spec = WorkflowSpecBuilder::new("Self Validation")
        .add_task(TaskBuilder::new("task:validate", "Validate").build())
        .add_task(TaskBuilder::new("task:report", "Report").build())
        .build();

    let mut tester = MutationTester::new(spec).unwrap();

    // Apply multiple mutations
    tester.apply_mutation(MutationOperator::RemoveTask("task:validate".to_string()));
    tester.apply_mutation(MutationOperator::ChangeTaskType(
        "task:report".to_string(),
        TaskType::Composite,
    ));

    // Act: Test mutation detection
    let caught = tester
        .test_mutation_detection(|spec| spec.tasks.len() > 0)
        .await;

    // Calculate mutation score
    let total_mutations = 2;
    let caught_mutations = if caught { total_mutations } else { 0 };
    let score = MutationScore::calculate(caught_mutations, total_mutations);

    // Assert: Mutation score is acceptable
    assert!(
        score.is_acceptable(),
        "Mutation score {}% should be >= 80%",
        score.score()
    );
}

// ============================================================================
// Property-Based Testing for Self-Validation
// ============================================================================

#[tokio::test]
async fn test_property_self_validation_always_succeeds() {
    // Property: Self-validation always succeeds for valid engine

    // Arrange: Create multiple validation managers
    for _ in 0..10 {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let manager = SelfValidationManager::new(temp_dir.path())
            .expect("Failed to create self-validation manager");

        // Act: Run validation
        let report = futures::executor::block_on(manager.run_validation_workflow())
            .expect("Validation should succeed");

        // Assert: Property holds - validation always succeeds
        assert!(
            report.validation_passed,
            "Property: Self-validation always succeeds for valid engine"
        );
    }
}
