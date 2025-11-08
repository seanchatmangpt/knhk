#[tokio::test]
async fn test_order_processing() {
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = create_order_processing_workflow();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    
    let data = TestDataBuilder::new()
        .with_order_data("ORD-001", "100.00")
        .with_customer_data("CUST-001")
        .build_json();
    
    let case_id = fixture.create_case(spec_id, data).await.unwrap();
    let case = fixture.execute_case(case_id).await.unwrap();
    fixture.assert_case_completed(&case);
}