use knhk_workflow_engine::testing::chicago_tdd::IntegrationTestHelper;

#[tokio::test]
async fn test_end_to_end_workflow() {
    let mut helper = IntegrationTestHelper::new().unwrap();
    
    let spec = create_test_workflow_spec();
    let data = TestDataBuilder::new()
        .with_order_data("ORD-001", "100.00")
        .build_json();
    
    // Execute complete workflow
    let case = helper.execute_complete_workflow(spec, data).await.unwrap();
    
    // Verify result
    helper.fixture().assert_case_completed(&case);
}