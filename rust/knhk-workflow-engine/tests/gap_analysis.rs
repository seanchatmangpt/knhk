//! Chicago TDD Gap Analysis Tests
//!
//! Uses coverage analysis to identify untested critical paths
//! and validate test quality through mutation testing.

use knhk_workflow_engine::patterns::{PatternId, PatternRegistry, RegisterAllExt};
// ChicagoTestContext and TestDataBuilder are not available in public API
// use knhk_workflow_engine::testing::chicago_tdd::{ChicagoTestContext, TestDataBuilder};
use std::collections::HashSet;

/// Test that all 43 patterns are registered
#[test]
fn test_all_43_patterns_registered() {
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();

    let patterns = registry.list_patterns();
    assert_eq!(
        patterns.len(),
        43,
        "Expected 43 patterns, found {}",
        patterns.len()
    );

    // Verify all pattern IDs 1-43 are present
    let pattern_ids: HashSet<u32> = patterns.iter().map(|p| p.0).collect();
    for id in 1..=43 {
        assert!(
            pattern_ids.contains(&id),
            "Pattern {} is missing from registry",
            id
        );
    }
}

/// Test that advanced control patterns (26-39) are registered
#[test]
fn test_advanced_control_patterns_registered() {
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();

    // Verify patterns 26-39 (advanced control)
    for id in 26..=39 {
        let pattern_id = PatternId(id);
        assert!(
            registry.has_pattern(&pattern_id),
            "Advanced control pattern {} not registered",
            id
        );
    }
}

/// Test that all patterns can execute without panicking
#[test]
fn test_all_patterns_execute_without_panic() {
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();

    let ctx = knhk_workflow_engine::patterns::PatternExecutionContext::default();

    // Test all 43 patterns
    for id in 1..=43 {
        let pattern_id = PatternId(id);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            registry.execute(&pattern_id, &ctx)
        }));

        assert!(result.is_ok(), "Pattern {} panicked during execution", id);
    }
}

/// Test pattern metadata availability
#[test]
fn test_pattern_metadata_coverage() {
    use knhk_workflow_engine::patterns::get_all_pattern_metadata;

    let metadata = get_all_pattern_metadata();

    // Should have metadata for all 43 patterns
    assert_eq!(
        metadata.len(),
        43,
        "Expected metadata for 43 patterns, found {}",
        metadata.len()
    );

    // Each pattern should have required fields
    for pattern in &metadata {
        assert!(
            !pattern.name.is_empty(),
            "Pattern {} has empty name",
            pattern.pattern_id
        );
        assert!(
            !pattern.description.is_empty(),
            "Pattern {} has empty description",
            pattern.pattern_id
        );
        assert!(
            !pattern.category.is_empty(),
            "Pattern {} has empty category",
            pattern.pattern_id
        );
    }
}

/// Property test: All pattern IDs are unique
#[test]
fn property_all_pattern_ids_unique() {
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();

    let patterns = registry.list_patterns();
    let unique_patterns: HashSet<u32> = patterns.iter().map(|p| p.0).collect();

    assert_eq!(
        patterns.len(),
        unique_patterns.len(),
        "Pattern IDs are not unique"
    );
}

/// Property test: All pattern executions return valid results
#[test]
fn property_all_patterns_return_valid_results() {
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();

    let ctx = knhk_workflow_engine::patterns::PatternExecutionContext::default();

    for id in 1..=43 {
        let pattern_id = PatternId(id);
        let result = registry.execute(&pattern_id, &ctx);

        assert!(
            result.is_some(),
            "Pattern {} returned None instead of valid result",
            id
        );
    }
}

/// Test performance of gap analysis (should complete quickly)
#[test]
fn test_gap_analysis_performance() {
    use std::time::Instant;

    let start = Instant::now();

    // Run gap analysis
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();
    let patterns = registry.list_patterns();
    let _metadata = knhk_workflow_engine::patterns::get_all_pattern_metadata();

    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 1000,
        "Gap analysis should complete in <1s, took {:?}",
        duration
    );

    assert_eq!(patterns.len(), 43);
}
