use knhk_workflow_engine::chicago_tdd_pattern_test;

chicago_tdd_pattern_test!(test_pattern_1_sequence, 1, |result| {
    assert_pattern_success(result);
    assert_pattern_has_next_state(result);
});