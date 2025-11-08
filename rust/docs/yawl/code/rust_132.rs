use knhk_workflow_engine::testing::chicago_tdd::*;

#[tokio::test]
async fn test_order_processing_workflow() {
    // Arrange: Set up fixture and create realistic test data
    let mut fixture = WorkflowTestFixture::new().unwrap();
    
    let spec = WorkflowSpecBuilder::new("Order Processing")
        .with_start_condition("condition:start")
        .with_end_condition("condition:end")
        .add_task(
            TaskBuilder::new("task:validate", "Validate Order")
                .with_max_ticks(8)
                .build()
        )
        .add_task(
            TaskBuilder::new("task:process_payment", "Process Payment")
                .with_max_ticks(8)
                .build()
        )
        .build();
    
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    
    let test_data = TestDataBuilder::new()
        .with_order_data("ORD-2024-001234", "149.99")
        .with_customer_data("CUST-789456")
        .with_var("payment_method", "credit_card")
        .build_json();
    
    let case_id = fixture.create_case(spec_id, test_data).await.unwrap();
    
    // Act: Execute workflow
    let perf = PerformanceTestHelper::new(8);
    let case = fixture.execute_case(case_id).await.unwrap();
    
    // Assert: Verify completion and performance
    fixture.assert_case_completed(&case);
    perf.verify_tick_budget();
}