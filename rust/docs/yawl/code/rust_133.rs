use knhk_workflow_engine::testing::chicago_tdd::*;

#[tokio::test]
async fn test_resource_allocation() {
    // Arrange: Set up fixture and resources
    let mut fixture = WorkflowTestFixture::new().unwrap();
    
    // Register resources
    let approver_role = create_test_role("approver", "Approver");
    let approval_capability = create_test_capability("approval", "Approval", 100);
    let resource = create_test_resource("User1", vec![approver_role], vec![approval_capability]);
    
    fixture.engine.resource_allocator()
        .register_resource(resource)
        .await
        .unwrap();
    
    // Create workflow with resource requirements
    let spec = create_approval_workflow_with_resources();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    
    // Act: Execute workflow
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await.unwrap();
    let case = fixture.execute_case(case_id).await.unwrap();
    
    // Assert: Workflow completed with resource allocation
    fixture.assert_case_completed(&case);
}