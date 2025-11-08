#[tokio::test]
async fn test_workflow_execution() {
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = WorkflowSpecBuilder::new("Test Workflow").build();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await.unwrap();
    let case = fixture.execute_case(case_id).await.unwrap();
    fixture.assert_case_completed(&case);
}