#[test]
fn test_pattern_1_sequence() {
    let registry = create_test_registry();
    let ctx = create_test_context();
    let result = registry.execute(&PatternId(1), &ctx).unwrap();
    assert_pattern_success(&result);
}