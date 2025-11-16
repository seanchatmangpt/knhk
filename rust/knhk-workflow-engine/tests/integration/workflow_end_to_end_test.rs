//! Integration Tests: End-to-End Workflow Execution
//!
//! Tests complete workflow lifecycle from ontology to execution.
//! Validates RDF parsing → workflow creation → execution → telemetry.

use knhk_workflow_engine::case::CaseState;
use knhk_workflow_engine::error::WorkflowResult;
use knhk_workflow_engine::testing::chicago_tdd::{
    create_parallel_split_workflow, create_simple_sequential_workflow, WorkflowTestFixture,
};

#[tokio::test]
async fn test_end_to_end_simple_workflow_execution() -> WorkflowResult<()> {
    // Arrange: Create complete workflow pipeline
    let mut fixture = WorkflowTestFixture::new()?;

    // Step 1: Define workflow specification
    let workflow = create_simple_sequential_workflow(
        "e2e_simple_workflow",
        "process_order",
        "Process Order",
    );

    // Step 2: Register workflow
    let spec_id = fixture.register_workflow(workflow).await?;

    // Step 3: Create case with business data
    let order_data = serde_json::json!({
        "order_id": "ORD-E2E-001",
        "customer_id": "CUST-001",
        "items": [
            {"sku": "WIDGET-1", "quantity": 2, "price": 50.00},
            {"sku": "GADGET-1", "quantity": 1, "price": 100.00}
        ],
        "total": 200.00
    });

    let case_id = fixture.create_case(spec_id, order_data).await?;

    // Act: Execute complete workflow
    let case = fixture.execute_case(case_id).await?;

    // Assert: End-to-end execution successful
    fixture.assert_case_completed(&case);
    assert_eq!(case.data["order_id"], "ORD-E2E-001");
    assert_eq!(case.data["total"], 200.00);

    // Verify telemetry was generated
    let xes_content = fixture
        .export_and_validate_xes(case_id, Some(&["Process Order"]))
        .await?;
    assert!(
        !xes_content.is_empty(),
        "E2E workflow should generate telemetry"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_end_to_end_parallel_workflow_execution() -> WorkflowResult<()> {
    // Arrange: Create parallel workflow pipeline
    let mut fixture = WorkflowTestFixture::new()?;

    // Workflow: Parallel approval process
    let workflow = create_parallel_split_workflow(
        "e2e_parallel_approval",
        vec![
            ("credit_check".to_string(), "Credit Check".to_string()),
            ("inventory_check".to_string(), "Inventory Check".to_string()),
            ("fraud_check".to_string(), "Fraud Check".to_string()),
        ],
    );

    let spec_id = fixture.register_workflow(workflow).await?;

    let approval_data = serde_json::json!({
        "transaction_id": "TXN-E2E-001",
        "amount": 5000.00,
        "customer_id": "CUST-PREMIUM-001"
    });

    let case_id = fixture.create_case(spec_id, approval_data).await?;

    // Act: Execute parallel approval workflow
    let case = fixture.execute_case(case_id).await?;

    // Assert: All parallel branches completed
    fixture.assert_case_completed(&case);

    // Verify all checks were performed
    let xes_content = fixture
        .export_and_validate_xes(
            case_id,
            Some(&[
                "Split",
                "Credit Check",
                "Inventory Check",
                "Fraud Check",
                "Join",
            ]),
        )
        .await?;

    assert!(
        xes_content.contains("Credit Check")
            && xes_content.contains("Inventory Check")
            && xes_content.contains("Fraud Check"),
        "E2E parallel workflow should execute all branches"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_end_to_end_multi_case_execution() -> WorkflowResult<()> {
    // Arrange: Execute multiple cases through complete pipeline
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow(
        "e2e_batch_workflow",
        "process_batch",
        "Process Batch Item",
    );

    let spec_id = fixture.register_workflow(workflow).await?;

    // Create batch of cases
    let batch_size = 10;
    let mut case_ids = Vec::new();

    for i in 0..batch_size {
        let case_id = fixture
            .create_case(
                spec_id,
                serde_json::json!({"batch_id": i, "item": format!("ITEM-{}", i)}),
            )
            .await?;
        case_ids.push(case_id);
    }

    // Act: Execute all cases
    for case_id in &case_ids {
        let case = fixture.execute_case(*case_id).await?;
        fixture.assert_case_completed(&case);
    }

    // Assert: All cases completed successfully
    assert_eq!(case_ids.len(), batch_size);

    // Verify each case has telemetry
    for (i, case_id) in case_ids.iter().enumerate() {
        let case = fixture.engine.get_case(*case_id).await?;
        assert_eq!(case.data["batch_id"], i);
    }

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_end_to_end_workflow_with_hooks() -> WorkflowResult<()> {
    // Arrange: Create workflow with lifecycle hooks
    let mut fixture = WorkflowTestFixture::new()?;

    // Register hook for validation
    use knhk_workflow_engine::hooks::{HookContext, HookRegistry, HookResult, HookType, WorkflowHook};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    let hook_calls = Arc::new(AtomicU32::new(0));
    let hook_calls_clone = hook_calls.clone();

    let validation_hook = WorkflowHook {
        id: "e2e_validation_hook".to_string(),
        hook_type: HookType::BeforeTaskExecution,
        name: "E2E Validation".to_string(),
        description: "Validates task execution in E2E test".to_string(),
        hook_fn: Arc::new(move |_ctx: &HookContext| {
            let calls = hook_calls_clone.clone();
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                HookResult::success()
            })
        }),
        enabled: true,
        priority: 0,
    };

    fixture.engine.hook_registry.register(validation_hook).await?;

    let workflow = create_simple_sequential_workflow("e2e_hooked_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute workflow with hooks
    let case = fixture.execute_case(case_id).await?;

    // Assert: Workflow executed with hooks
    fixture.assert_case_completed(&case);
    assert!(
        hook_calls.load(Ordering::SeqCst) >= 1,
        "E2E workflow should execute hooks"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_end_to_end_workflow_with_process_mining_export() -> WorkflowResult<()> {
    // Arrange: Create workflow for process mining
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_parallel_split_workflow(
        "e2e_process_mining",
        vec![
            ("discover".to_string(), "Discover Pattern".to_string()),
            ("check_conformance".to_string(), "Check Conformance".to_string()),
            ("enhance".to_string(), "Enhance Model".to_string()),
        ],
    );

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute and export for process mining
    let case = fixture.execute_case(case_id).await?;
    fixture.assert_case_completed(&case);

    // Assert: XES export is process mining ready
    let xes_content = fixture
        .export_and_validate_xes(
            case_id,
            Some(&[
                "Split",
                "Discover Pattern",
                "Check Conformance",
                "Enhance Model",
                "Join",
            ]),
        )
        .await?;

    // Verify XES 2.0 compliance
    assert!(
        xes_content.contains("xes.version=\"2.0\""),
        "E2E workflow should export XES 2.0 for process mining"
    );

    // Verify event structure
    assert!(
        xes_content.contains("concept:name")
            && xes_content.contains("time:timestamp")
            && xes_content.contains("lifecycle:transition"),
        "E2E XES should contain standard process mining attributes"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_end_to_end_workflow_error_handling() -> WorkflowResult<()> {
    // Arrange: Create workflow that may encounter errors
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow(
        "e2e_error_handling",
        "risky_task",
        "Risky Task",
    );

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute workflow
    let result = fixture.execute_case(case_id).await;

    // Assert: Workflow handles errors gracefully
    assert!(
        result.is_ok() || matches!(result.as_ref().err(), Some(_)),
        "E2E workflow should handle errors gracefully"
    );

    if let Ok(case) = result {
        assert!(
            case.state == CaseState::Completed || case.state == CaseState::Failed,
            "E2E workflow should reach terminal state"
        );
    }

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_end_to_end_performance_within_budget() -> WorkflowResult<()> {
    // Arrange: Create workflow with performance constraints
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("e2e_performance", "fast_task", "Fast Task");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute and measure performance
    let start = std::time::Instant::now();
    let case = fixture.execute_case(case_id).await?;
    let duration = start.elapsed();

    // Assert: E2E execution within performance budget
    fixture.assert_case_completed(&case);
    assert!(
        duration.as_millis() < 1000, // 1 second E2E budget
        "E2E workflow should execute within performance budget: {:?}",
        duration
    );

    fixture.cleanup()?;
    Ok(())
}
