#[test]
fn test_pattern_performance() {
    let perf = PerformanceTestHelper::new(8);
    let registry = create_test_registry();
    let ctx = create_test_context();
    let _result = registry.execute(&PatternId(1), &ctx).unwrap();
    perf.verify_tick_budget();
}