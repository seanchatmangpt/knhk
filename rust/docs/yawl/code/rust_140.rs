#[tokio::test]
async fn test_completion_property() {
    let mut tester = WorkflowPropertyTester::new(100).unwrap();
    let spec_id = register_test_workflow().await;
    let result = tester.test_completion_property(spec_id).await.unwrap();
    assert!(result);
}