use knhk_workflow_engine::chicago_tdd_workflow_test;

chicago_tdd_workflow_test!(test_approval_workflow, |fixture| async move {
    // Arrange
    let spec = create_approval_workflow();
    let spec_id = fixture.register_workflow(spec).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;
    
    // Act
    let case = fixture.execute_case(case_id).await?;
    
    // Assert
    fixture.assert_case_completed(&case);
    Ok(())
});