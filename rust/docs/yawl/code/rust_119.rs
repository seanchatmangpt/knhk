use knhk_workflow_engine::testing::chicago_tdd::WorkflowTestFixture;

#[tokio::test]
async fn test_workflow_execution() {
    // Arrange: Set up test fixture
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = create_test_workflow_spec();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await.unwrap();
    
    // Act: Execute case
    let case = fixture.execute_case(case_id).await.unwrap();
    
    // Assert: Case completes successfully
    fixture.assert_case_completed(&case);
    fixture.assert_case_running(&case); // or assert_case_failed, assert_case_cancelled
}