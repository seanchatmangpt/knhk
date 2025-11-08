#[test]
fn test_pattern_4_exclusive_choice() {
    let registry = create_test_registry();
    let mut vars = HashMap::new();
    vars.insert("condition".to_string(), "true".to_string());
    let ctx = create_test_context_with_vars(vars);
    let result = registry.execute(&PatternId(4), &ctx).unwrap();
    assert_pattern_success(&result);
    assert_pattern_variable_equals(&result, "condition", "true");
}