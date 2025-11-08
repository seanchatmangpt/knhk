use knhk_workflow_engine::testing::chicago_tdd::PerformanceTestHelper;

#[test]
fn test_pattern_performance() {
    let perf = PerformanceTestHelper::new(8); // Max 8 ticks
    
    // Execute pattern
    let result = execute_pattern();
    
    // Verify tick budget
    perf.verify_tick_budget();
    assert!(perf.elapsed_ticks() <= 8);
}