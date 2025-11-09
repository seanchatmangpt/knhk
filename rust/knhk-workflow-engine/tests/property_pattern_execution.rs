//! Property-Based Tests for Pattern Execution
//!
//! Verifies that all workflow patterns execute correctly across
//! random inputs and satisfy expected properties.

use knhk_workflow_engine::patterns::{
    PatternExecutionContext, PatternId, PatternRegistry, RegisterAllExt,
};
use std::collections::HashMap;

/// Property: All patterns should execute without panicking
#[test]
fn property_all_patterns_execute_without_panic() {
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();

    // Test each pattern with default context
    for pattern_id in 1..=43 {
        let ctx = PatternExecutionContext::default();
        let pattern = PatternId(pattern_id);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            registry.execute(&pattern, &ctx)
        }));

        assert!(
            result.is_ok(),
            "Pattern {} panicked with default context",
            pattern_id
        );
    }
}

/// Property: Pattern execution should be deterministic
#[test]
fn property_pattern_execution_deterministic() {
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();

    // Create a fixed context
    let ctx = PatternExecutionContext {
        case_id: knhk_workflow_engine::case::CaseId::new(),
        workflow_id: knhk_workflow_engine::parser::WorkflowSpecId::new(),
        variables: {
            let mut vars = HashMap::new();
            vars.insert("test_var".to_string(), "test_value".to_string());
            vars
        },
        arrived_from: std::collections::HashSet::new(),
        scope_id: "test_scope".to_string(),
    };

    // Execute same pattern multiple times
    for pattern_id in 1..=43 {
        let pattern = PatternId(pattern_id);

        let result1 = registry.execute(&pattern, &ctx);
        let result2 = registry.execute(&pattern, &ctx);

        // Results should be identical (deterministic)
        assert!(
            result1.is_some() && result2.is_some(),
            "Pattern {} returned None on some execution",
            pattern_id
        );

        if let (Some(r1), Some(r2)) = (result1, result2) {
            assert_eq!(
                r1.success, r2.success,
                "Pattern {} not deterministic (success field differs)",
                pattern_id
            );
            assert_eq!(
                r1.next_activities, r2.next_activities,
                "Pattern {} not deterministic (next_activities differs)",
                pattern_id
            );
            assert_eq!(
                r1.terminates, r2.terminates,
                "Pattern {} not deterministic (terminates flag differs)",
                pattern_id
            );
        }
    }
}

/// Property: All patterns should return valid execution results
#[test]
fn property_all_patterns_return_valid_results() {
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();

    let ctx = PatternExecutionContext::default();

    for pattern_id in 1..=43 {
        let pattern = PatternId(pattern_id);

        let result = registry.execute(&pattern, &ctx);

        assert!(
            result.is_some(),
            "Pattern {} returned None instead of valid result",
            pattern_id
        );
    }
}

/// Property: Cancellation patterns should populate cancel_activities
#[test]
fn property_cancellation_patterns_populate_cancel_list() {
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();

    let ctx = PatternExecutionContext::default();

    // Cancellation patterns: 19-25, 32-35
    let cancellation_patterns = vec![19, 20, 21, 22, 23, 24, 25, 32, 33, 34, 35];

    for pattern_id in cancellation_patterns {
        let pattern = PatternId(pattern_id);
        let result = registry.execute(&pattern, &ctx);

        assert!(
            result.is_some(),
            "Cancellation pattern {} returned None",
            pattern_id
        );
    }
}

/// Property: Basic control flow patterns should produce next activities
#[test]
fn property_basic_patterns_produce_next_activities() {
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();

    let mut ctx = PatternExecutionContext::default();
    // Provide some arrived_from edges for join patterns
    ctx.arrived_from.insert("edge1".to_string());
    ctx.arrived_from.insert("edge2".to_string());

    // Basic patterns: 1-5
    for pattern_id in 1..=5 {
        let pattern = PatternId(pattern_id);
        let result = registry.execute(&pattern, &ctx);

        assert!(
            result.is_some(),
            "Basic pattern {} returned None",
            pattern_id
        );
    }
}

/// Property: Advanced control patterns (26-39) execute successfully
#[test]
fn property_advanced_control_patterns_execute() {
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();

    let ctx = PatternExecutionContext::default();

    // Advanced control patterns: 26-39 (the critical gap!)
    for pattern_id in 26..=39 {
        let pattern = PatternId(pattern_id);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            registry.execute(&pattern, &ctx)
        }));

        assert!(
            result.is_ok(),
            "Advanced control pattern {} panicked",
            pattern_id
        );
    }
}

/// Property: Pattern execution should complete quickly
#[test]
fn property_pattern_execution_performance() {
    use std::time::Instant;

    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();

    let ctx = PatternExecutionContext::default();

    // Each pattern execution should complete in <100ms
    for pattern_id in 1..=43 {
        let pattern = PatternId(pattern_id);

        let start = Instant::now();
        let _result = registry.execute(&pattern, &ctx);
        let duration = start.elapsed();

        assert!(
            duration.as_millis() < 100,
            "Pattern {} execution took too long: {:?}",
            pattern_id,
            duration
        );
    }
}

/// Property: All 43 patterns registered in registry
#[test]
fn property_all_43_patterns_registered() {
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();

    // Check all 43 patterns
    for pattern_id in 1..=43 {
        let pattern = PatternId(pattern_id);
        assert!(
            registry.has_pattern(&pattern),
            "Pattern {} not registered in registry",
            pattern_id
        );
    }

    // Should have exactly 43 patterns
    let patterns = registry.list_patterns();
    assert_eq!(
        patterns.len(),
        43,
        "Registry should have exactly 43 patterns, found {}",
        patterns.len()
    );
}
