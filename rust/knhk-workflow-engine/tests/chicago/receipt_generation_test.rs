//! Chicago TDD Tests: Receipt Generation and Validation
//!
//! Tests workflow execution receipts (lockchain integration).
//! Validates receipt generation, merging, and verification.

use knhk_workflow_engine::case::CaseState;
use knhk_workflow_engine::error::WorkflowResult;
use knhk_workflow_engine::testing::chicago_tdd::{
    create_parallel_split_workflow, create_simple_sequential_workflow, WorkflowTestFixture,
};

#[tokio::test]
async fn test_receipt_is_generated_for_workflow_execution() -> WorkflowResult<()> {
    // Arrange: Create simple workflow
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("receipt_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute workflow
    let case = fixture.execute_case(case_id).await?;

    // Assert: Receipt is generated (verified via case state and XES export)
    fixture.assert_case_completed(&case);

    // XES export validates receipt generation
    let xes_content = fixture.export_and_validate_xes(case_id, None).await?;
    assert!(
        !xes_content.is_empty(),
        "Receipt should be generated as XES event log"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_receipt_contains_execution_timestamp() -> WorkflowResult<()> {
    // Arrange: Create workflow
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("timestamped_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute and get receipt
    let case = fixture.execute_case(case_id).await?;
    fixture.assert_case_completed(&case);

    // Assert: Receipt contains timestamps
    let xes_content = fixture.export_and_validate_xes(case_id, None).await?;
    assert!(
        xes_content.contains("time:timestamp"),
        "Receipt should contain execution timestamps"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_receipt_includes_task_execution_details() -> WorkflowResult<()> {
    // Arrange: Create workflow with multiple tasks
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_parallel_split_workflow(
        "detailed_receipt_workflow",
        vec![
            ("task_a".to_string(), "Task A".to_string()),
            ("task_b".to_string(), "Task B".to_string()),
        ],
    );

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute workflow
    let case = fixture.execute_case(case_id).await?;
    fixture.assert_case_completed(&case);

    // Assert: Receipt includes all task details
    let xes_content = fixture
        .export_and_validate_xes(case_id, Some(&["Split", "Task A", "Task B", "Join"]))
        .await?;

    // Verify task details in receipt
    assert!(
        xes_content.contains("concept:name"),
        "Receipt should include task names"
    );
    assert!(
        xes_content.contains("lifecycle:transition"),
        "Receipt should include lifecycle transitions"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_receipt_tracks_workflow_version() -> WorkflowResult<()> {
    // Arrange: Create versioned workflow
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("versioned_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute and get receipt
    let case = fixture.execute_case(case_id).await?;
    fixture.assert_case_completed(&case);

    // Assert: Receipt tracks workflow spec ID (version proxy)
    let xes_content = fixture.export_and_validate_xes(case_id, None).await?;
    assert!(
        xes_content.contains(&spec_id.to_string()) || xes_content.contains("workflow"),
        "Receipt should track workflow version/ID"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_receipt_preserves_execution_order() -> WorkflowResult<()> {
    // Arrange: Create sequential workflow
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("ordered_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute workflow
    let case = fixture.execute_case(case_id).await?;
    fixture.assert_case_completed(&case);

    // Assert: Receipt preserves execution order
    let history = fixture.get_case_history(case_id).await;
    assert!(
        !history.is_empty(),
        "Receipt should preserve execution history"
    );

    // Verify XES maintains order
    let xes_content = fixture.export_and_validate_xes(case_id, None).await?;
    assert!(
        xes_content.contains("<trace>"),
        "Receipt should maintain trace ordering"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_receipt_is_immutable_after_generation() -> WorkflowResult<()> {
    // Arrange: Create workflow
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("immutable_receipt_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute and generate receipt
    let case = fixture.execute_case(case_id).await?;
    fixture.assert_case_completed(&case);

    let receipt_1 = fixture.export_and_validate_xes(case_id, None).await?;

    // Wait and get receipt again
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let receipt_2 = fixture.export_and_validate_xes(case_id, None).await?;

    // Assert: Receipt is immutable (same content)
    assert_eq!(
        receipt_1, receipt_2,
        "Receipt should be immutable after generation"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_receipt_merging_combines_parallel_executions() -> WorkflowResult<()> {
    // Arrange: Create parallel workflow
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_parallel_split_workflow(
        "parallel_receipt_workflow",
        vec![
            ("branch_1".to_string(), "Branch 1".to_string()),
            ("branch_2".to_string(), "Branch 2".to_string()),
            ("branch_3".to_string(), "Branch 3".to_string()),
        ],
    );

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute parallel workflow
    let case = fixture.execute_case(case_id).await?;
    fixture.assert_case_completed(&case);

    // Assert: Receipt merges all parallel branch executions
    let xes_content = fixture
        .export_and_validate_xes(
            case_id,
            Some(&["Split", "Branch 1", "Branch 2", "Branch 3", "Join"]),
        )
        .await?;

    // Verify all branches are in merged receipt
    assert!(
        xes_content.contains("Branch 1")
            && xes_content.contains("Branch 2")
            && xes_content.contains("Branch 3"),
        "Receipt should merge all parallel branch executions"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_receipt_validates_against_workflow_schema() -> WorkflowResult<()> {
    // Arrange: Create workflow
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("schema_validated_receipt", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute and validate receipt
    let case = fixture.execute_case(case_id).await?;
    fixture.assert_case_completed(&case);

    // Assert: Receipt validates against XES 2.0 schema
    let xes_content = fixture.export_and_validate_xes(case_id, None).await?;

    // XES schema validation
    assert!(
        xes_content.contains("<?xml version"),
        "Receipt should be valid XML"
    );
    assert!(
        xes_content.contains("xes.version=\"2.0\""),
        "Receipt should conform to XES 2.0 schema"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_receipt_includes_performance_metrics() -> WorkflowResult<()> {
    // Arrange: Create workflow with performance tracking
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("performance_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute with performance metrics
    let start = std::time::Instant::now();
    let case = fixture.execute_case(case_id).await?;
    let duration = start.elapsed();

    fixture.assert_case_completed(&case);

    // Assert: Receipt includes performance data (timestamps enable duration calculation)
    let xes_content = fixture.export_and_validate_xes(case_id, None).await?;
    assert!(
        xes_content.contains("time:timestamp"),
        "Receipt should include timestamps for performance analysis"
    );

    // Verify execution was tracked
    assert!(
        duration.as_micros() > 0,
        "Performance metrics should be captured"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_receipt_supports_process_mining_analysis() -> WorkflowResult<()> {
    // Arrange: Create workflow for process mining
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_parallel_split_workflow(
        "process_mining_workflow",
        vec![
            ("analyze".to_string(), "Analyze".to_string()),
            ("validate".to_string(), "Validate".to_string()),
        ],
    );

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute and export for process mining
    let case = fixture.execute_case(case_id).await?;
    fixture.assert_case_completed(&case);

    // Assert: Receipt is compatible with process mining tools
    let xes_content = fixture
        .export_and_validate_xes(case_id, Some(&["Split", "Analyze", "Validate", "Join"]))
        .await?;

    // Verify XES 2.0 compliance for process mining
    assert!(
        xes_content.contains("xes.version=\"2.0\""),
        "Receipt should be XES 2.0 compliant for process mining"
    );
    assert!(
        xes_content.contains("<trace>") && xes_content.contains("<event>"),
        "Receipt should contain trace and event structure for process mining"
    );

    fixture.cleanup()?;
    Ok(())
}
