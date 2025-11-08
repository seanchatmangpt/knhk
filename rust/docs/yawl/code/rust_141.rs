fn assert_order_processed(result: &PatternExecutionResult) {
    assert_pattern_success(result);
    assert_pattern_has_variable(result, "order_id");
    assert_pattern_has_variable(result, "order_status");
    assert_pattern_variable_equals(result, "order_status", "processed");
}