use knhk_workflow_engine::testing::chicago_tdd::*;

#[test]
fn test_pattern_execution() {
    // Arrange: Create registry and context
    let registry = create_test_registry();
    let mut vars = HashMap::new();
    vars.insert("condition".to_string(), "true".to_string());
    let ctx = create_test_context_with_vars(vars);
    
    // Act: Execute pattern
    let result = registry.execute(&PatternId(4), &ctx)
        .expect("Pattern should be registered");
    
    // Assert: Pattern executed successfully
    assert_pattern_success(&result);
    assert_pattern_has_next_state(&result);
    assert_pattern_variable_equals(&result, "condition", "true");
}