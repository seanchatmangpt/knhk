use knhk_workflow_engine::testing::chicago_tdd::*;

#[tokio::test]
async fn test_worklet_exception_handling() {
    // Arrange: Set up fixture and worklet
    let mut fixture = WorkflowTestFixture::new().unwrap();
    
    let worklet = create_test_worklet(
        "Resource Unavailable Handler",
        vec!["resource_unavailable".to_string()]
    );
    
    fixture.engine.worklet_repository()
        .register(worklet)
        .await
        .unwrap();
    
    // Create workflow with exception worklet
    let spec = create_workflow_with_exception_worklet();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    
    // Act: Execute workflow that triggers exception
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await.unwrap();
    let case = fixture.execute_case(case_id).await.unwrap();
    
    // Assert: Exception handled via worklet
    fixture.assert_case_completed(&case);
}