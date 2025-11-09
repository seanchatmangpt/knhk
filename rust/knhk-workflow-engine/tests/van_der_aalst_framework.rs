//! Van der Aalst End-to-End Validation Framework Test Harness
//!
//! Unified test harness that integrates all validation phases:
//! - Fitness validation
//! - Precision validation
//! - Generalization validation
//! - Process mining analysis
//! - Formal verification

use knhk_workflow_engine::{
    case::CaseId,
    parser::{Flow, JoinType, SplitType, Task, TaskType, WorkflowSpec, WorkflowSpecId},
    state::StateStore,
    validation::ValidationFramework,
    WorkflowEngine,
};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;

/// Create a test workflow for validation
fn create_test_workflow() -> WorkflowSpec {
    let mut tasks = HashMap::new();
    tasks.insert(
        "task_a".to_string(),
        Task {
            id: "task_a".to_string(),
            name: "Task A".to_string(),
            task_type: TaskType::Atomic,
            flows: Vec::new(),
            allocation_policy: None,
        },
    );
    tasks.insert(
        "task_b".to_string(),
        Task {
            id: "task_b".to_string(),
            name: "Task B".to_string(),
            task_type: TaskType::Atomic,
            flows: Vec::new(),
            allocation_policy: None,
        },
    );

    let mut flows = Vec::new();
    flows.push(Flow {
        from: "start".to_string(),
        to: "task_a".to_string(),
        split_type: SplitType::None,
        join_type: JoinType::None,
    });
    flows.push(Flow {
        from: "task_a".to_string(),
        to: "task_b".to_string(),
        split_type: SplitType::None,
        join_type: JoinType::None,
    });
    flows.push(Flow {
        from: "task_b".to_string(),
        to: "end".to_string(),
        split_type: SplitType::None,
        join_type: JoinType::None,
    });

    WorkflowSpec {
        id: WorkflowSpecId::new(),
        name: "Test Workflow".to_string(),
        tasks,
        conditions: HashMap::new(),
        flows,
        start_condition: Some("start".to_string()),
        end_condition: Some("end".to_string()),
        source_turtle: None,
    }
}

#[tokio::test]
async fn test_validation_framework_complete() {
    println!("[TEST] Van der Aalst End-to-End Validation Framework");
    println!("  Testing complete validation framework");

    // Arrange: Setup engine and workflow
    let temp_dir = TempDir::new().unwrap();
    let state_store = Arc::new(StateStore::new(temp_dir.path()).unwrap());
    let engine = Arc::new(WorkflowEngine::new(state_store.clone()));

    let spec = create_test_workflow();
    engine.register_workflow(spec.clone()).await.unwrap();

    // Create and execute case
    let case_id = engine
        .create_case(spec.id, serde_json::json!({"test": true}))
        .await
        .unwrap();
    engine.start_case(case_id).await.unwrap();

    // Wait for execution
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Act: Run complete validation framework
    let framework = ValidationFramework::new(engine.clone());
    let report = framework
        .run_complete_validation(spec.id)
        .await
        .expect("Validation framework should complete");

    // Assert: Verify report structure
    assert_eq!(report.spec_id, spec.id);
    assert!(
        report.phases.len() > 0,
        "Report should contain phase results"
    );

    // Verify all phases are present
    assert!(
        report.phases.contains_key("fitness"),
        "Report should contain fitness phase"
    );
    assert!(
        report.phases.contains_key("precision"),
        "Report should contain precision phase"
    );
    assert!(
        report.phases.contains_key("generalization"),
        "Report should contain generalization phase"
    );
    assert!(
        report.phases.contains_key("process_mining"),
        "Report should contain process_mining phase"
    );
    assert!(
        report.phases.contains_key("formal"),
        "Report should contain formal phase"
    );

    // Verify summary
    assert_eq!(report.summary.total_phases, 5);
    assert!(report.summary.passed_tests > 0, "Should have passed tests");

    println!("  ✅ Complete validation framework test passed");
}

#[tokio::test]
async fn test_validation_framework_phase() {
    println!("[TEST] Van der Aalst Validation Framework - Single Phase");
    println!("  Testing individual phase validation");

    // Arrange: Setup engine and workflow
    let temp_dir = TempDir::new().unwrap();
    let state_store = Arc::new(StateStore::new(temp_dir.path()).unwrap());
    let engine = Arc::new(WorkflowEngine::new(state_store.clone()));

    let spec = create_test_workflow();
    engine.register_workflow(spec.clone()).await.unwrap();

    // Create and execute case
    let case_id = engine
        .create_case(spec.id, serde_json::json!({"test": true}))
        .await
        .unwrap();
    engine.start_case(case_id).await.unwrap();

    // Wait for execution
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Act: Run fitness phase only
    let framework = ValidationFramework::new(engine.clone());
    let result = framework
        .run_phase("fitness", spec.id)
        .await
        .expect("Fitness phase should complete");

    // Assert: Verify result
    assert_eq!(result.phase, "fitness");
    assert!(
        result.passed > 0 || result.failed > 0,
        "Should have test results"
    );

    println!("  ✅ Single phase validation test passed");
}

#[tokio::test]
async fn test_validation_report_generation() {
    println!("[TEST] Validation Report Generation");
    println!("  Testing report generation in multiple formats");

    // Arrange: Setup engine and workflow
    let temp_dir = TempDir::new().unwrap();
    let state_store = Arc::new(StateStore::new(temp_dir.path()).unwrap());
    let engine = Arc::new(WorkflowEngine::new(state_store.clone()));

    let spec = create_test_workflow();
    engine.register_workflow(spec.clone()).await.unwrap();

    // Create and execute case
    let case_id = engine
        .create_case(spec.id, serde_json::json!({"test": true}))
        .await
        .unwrap();
    engine.start_case(case_id).await.unwrap();

    // Wait for execution
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Act: Run validation and generate reports
    let framework = ValidationFramework::new(engine.clone());
    let report = framework
        .run_complete_validation(spec.id)
        .await
        .expect("Validation framework should complete");

    // Generate Markdown report
    let markdown = report.to_markdown();
    assert!(markdown.contains("# Van der Aalst Validation Report"));
    assert!(markdown.contains(&spec.id.to_string()));

    // Generate JSON report
    let json = report.to_json().expect("JSON generation should succeed");
    assert!(json.contains(&spec.id.to_string()));

    // Generate HTML report
    let html = report.to_html();
    assert!(html.contains("Van der Aalst Validation Report"));

    println!("  ✅ Report generation test passed");
}
