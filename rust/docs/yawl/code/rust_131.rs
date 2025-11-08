use knhk_workflow_engine::testing::chicago_tdd::*;

#[test]
fn test_all_patterns() {
    let registry = create_test_registry();
    
    for pattern_id in 1..=43 {
        let ctx = create_test_context();
        let result = registry.execute(&PatternId(pattern_id), &ctx)
            .expect(&format!("Pattern {} should be registered", pattern_id));
        
        assert_pattern_success(&result);
        assert_pattern_has_next_state(&result);
    }
}