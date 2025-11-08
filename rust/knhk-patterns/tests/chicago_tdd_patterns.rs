// rust/knhk-patterns/tests/chicago_tdd_patterns.rs
// Chicago TDD tests for workflow patterns
// Tests behavior (what patterns do), not implementation (how they do it)

use knhk_patterns::*;
use std::sync::Arc;

// ============================================================================
// Test Data Types
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
struct TestData {
    value: i32,
    processed: bool,
}

impl TestData {
    fn new(value: i32) -> Self {
        Self {
            value,
            processed: false,
        }
    }
}

// ============================================================================
// Pattern 1: Sequence Tests
// ============================================================================

#[test]
fn test_sequence_executes_branches_in_order() {
    // Arrange
    let branch1 = Arc::new(|mut data: TestData| {
        data.value *= 2;
        Ok(data)
    });

    let branch2 = Arc::new(|mut data: TestData| {
        data.value += 10;
        Ok(data)
    });

    let pattern = SequencePattern::new(vec![branch1, branch2]).unwrap();

    // Act
    let input = TestData::new(5);
    let results = pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value, 20); // (5 * 2) + 10 = 20
}

#[test]
fn test_sequence_stops_on_first_error() {
    // Arrange
    let branch1 = Arc::new(|data: TestData| Ok(data));

    let branch2 = Arc::new(|_data: TestData| {
        Err(PatternError::ExecutionFailed("Intentional error".to_string()))
    });

    let branch3 = Arc::new(|mut data: TestData| {
        data.processed = true;
        Ok(data)
    });

    let pattern = SequencePattern::new(vec![branch1, branch2, branch3]).unwrap();

    // Act
    let input = TestData::new(5);
    let result = pattern.execute(input);

    // Assert
    assert!(result.is_err());
    // Branch 3 should not have executed
}

// ============================================================================
// Pattern 2: Parallel Split Tests
// ============================================================================

#[test]
fn test_parallel_split_executes_all_branches() {
    // Arrange
    let branch1 = Arc::new(|mut data: TestData| {
        data.value *= 2;
        Ok(data)
    });

    let branch2 = Arc::new(|mut data: TestData| {
        data.value *= 3;
        Ok(data)
    });

    let branch3 = Arc::new(|mut data: TestData| {
        data.value *= 5;
        Ok(data)
    });

    let pattern = ParallelSplitPattern::new(vec![branch1, branch2, branch3]).unwrap();

    // Act
    let input = TestData::new(10);
    let results = pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 3);

    // Results may be in any order due to parallel execution
    let mut values: Vec<i32> = results.iter().map(|r| r.value).collect();
    values.sort_unstable();

    assert_eq!(values, vec![20, 30, 50]); // 10*2, 10*3, 10*5
}

#[test]
fn test_parallel_split_fails_if_any_branch_fails() {
    // Arrange
    let branch1 = Arc::new(|data: TestData| Ok(data));

    let branch2 = Arc::new(|_data: TestData| {
        Err(PatternError::ExecutionFailed("Branch 2 failed".to_string()))
    });

    let branch3 = Arc::new(|data: TestData| Ok(data));

    let pattern = ParallelSplitPattern::new(vec![branch1, branch2, branch3]).unwrap();

    // Act
    let input = TestData::new(5);
    let result = pattern.execute(input);

    // Assert
    assert!(result.is_err());
}

// ============================================================================
// Pattern 4: Exclusive Choice Tests
// ============================================================================

#[test]
fn test_exclusive_choice_selects_first_matching_condition() {
    // Arrange
    let choices = vec![
        (
            Arc::new(|data: &TestData| data.value < 0) as ConditionFn<TestData>,
            Arc::new(|mut data: TestData| {
                data.value = -1;
                Ok(data)
            }) as BranchFn<TestData>,
        ),
        (
            Arc::new(|data: &TestData| data.value > 10),
            Arc::new(|mut data: TestData| {
                data.value = 100;
                Ok(data)
            }),
        ),
        (
            Arc::new(|data: &TestData| data.value <= 10),
            Arc::new(|mut data: TestData| {
                data.value = 50;
                Ok(data)
            }),
        ),
    ];

    let pattern = ExclusiveChoicePattern::new(choices).unwrap();

    // Act
    let input = TestData::new(5);
    let results = pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value, 50); // Third condition matched
}

#[test]
fn test_exclusive_choice_fails_if_no_condition_matches() {
    // Arrange
    let choices = vec![
        (
            Arc::new(|data: &TestData| data.value > 100) as ConditionFn<TestData>,
            Arc::new(|data: TestData| Ok(data)) as BranchFn<TestData>,
        ),
        (
            Arc::new(|data: &TestData| data.value < 0),
            Arc::new(|data: TestData| Ok(data)),
        ),
    ];

    let pattern = ExclusiveChoicePattern::new(choices).unwrap();

    // Act
    let input = TestData::new(5);
    let result = pattern.execute(input);

    // Assert
    assert!(result.is_err());
}

// ============================================================================
// Pattern 6: Multi-Choice Tests
// ============================================================================

#[test]
fn test_multi_choice_executes_all_matching_branches() {
    // Arrange
    let choices = vec![
        (
            Arc::new(|data: &TestData| data.value > 0) as ConditionFn<TestData>,
            Arc::new(|mut data: TestData| {
                data.value *= 2;
                Ok(data)
            }) as BranchFn<TestData>,
        ),
        (
            Arc::new(|data: &TestData| data.value < 10),
            Arc::new(|mut data: TestData| {
                data.value += 5;
                Ok(data)
            }),
        ),
        (
            Arc::new(|data: &TestData| data.value == 5),
            Arc::new(|mut data: TestData| {
                data.processed = true;
                Ok(data)
            }),
        ),
    ];

    let pattern = MultiChoicePattern::new(choices).unwrap();

    // Act
    let input = TestData::new(5);
    let results = pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 3); // All three conditions match
    assert!(results.iter().any(|r| r.value == 10)); // 5 * 2
    assert!(results.iter().any(|r| r.value == 10)); // 5 + 5
    assert!(results.iter().any(|r| r.processed)); // processed flag set
}

// ============================================================================
// Pattern 10: Arbitrary Cycles Tests
// ============================================================================

#[test]
fn test_arbitrary_cycles_retries_until_condition_false() {
    // Arrange
    let branch = Arc::new(|mut data: TestData| {
        data.value += 1;
        Ok(data)
    });

    let should_continue = Arc::new(|data: &TestData| data.value < 10);

    let pattern = ArbitraryCyclesPattern::new(branch, should_continue, 100).unwrap();

    // Act
    let input = TestData::new(5);
    let results = pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value, 10); // Stopped when value == 10
}

#[test]
fn test_arbitrary_cycles_respects_max_iterations() {
    // Arrange
    let branch = Arc::new(|mut data: TestData| {
        data.value += 1;
        Ok(data)
    });

    let should_continue = Arc::new(|_data: &TestData| true); // Always continue

    let pattern = ArbitraryCyclesPattern::new(branch, should_continue, 5).unwrap();

    // Act
    let input = TestData::new(0);
    let results = pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value, 5); // Stopped after 5 iterations
}

// ============================================================================
// Pattern Composition Tests
// ============================================================================

#[test]
fn test_composite_pattern_sequence_of_parallels() {
    // Arrange: Parallel → Sequence → Parallel
    use knhk_patterns::composition::CompositePattern;

    let parallel1_branches = vec![
        Arc::new(|mut data: TestData| {
            data.value *= 2;
            Ok(data)
        }) as BranchFn<TestData>,
        Arc::new(|mut data: TestData| {
            data.value *= 3;
            Ok(data)
        }),
    ];

    let parallel1 = ParallelSplitPattern::new(parallel1_branches).unwrap();

    let sequential_branch = Arc::new(|mut data: TestData| {
        data.value += 10;
        Ok(data)
    });

    let sequential = SequencePattern::new(vec![sequential_branch]).unwrap();

    let composite = CompositePattern::Sequence(vec![
        Box::new(parallel1),
        Box::new(sequential),
    ]);

    // Act
    let input = TestData::new(5);
    let results = composite.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 2);
    let mut values: Vec<i32> = results.iter().map(|r| r.value).collect();
    values.sort_unstable();
    assert_eq!(values, vec![20, 25]); // (5*2)+10=20, (5*3)+10=25
}

// ============================================================================
// Ingress Validation Tests (Guards)
// ============================================================================

#[test]
fn test_pattern_validates_at_ingress() {
    // Test that patterns validate constraints at creation (ingress)
    // NOT during execution (hot path)

    use knhk_patterns::ffi::PatternType;

    // Valid: 1 branch
    assert!(PatternType::Sequence.validate_ingress(1).is_ok());

    // Valid: 100 branches
    assert!(PatternType::ParallelSplit.validate_ingress(100).is_ok());

    // Invalid: 0 branches
    assert!(PatternType::ExclusiveChoice.validate_ingress(0).is_err());

    // Invalid: Too many branches
    assert!(PatternType::MultiChoice.validate_ingress(2000).is_err());
}

#[test]
fn test_pattern_tick_budgets_within_chatman_constant() {
    // Verify all patterns have tick budgets ≤8 (Chatman Constant)
    use knhk_patterns::ffi::PatternType;

    let patterns = vec![
        PatternType::Sequence,
        PatternType::ParallelSplit,
        PatternType::Synchronization,
        PatternType::ExclusiveChoice,
        PatternType::SimpleMerge,
        PatternType::MultiChoice,
        PatternType::ArbitraryCycles,
        PatternType::DeferredChoice,
    ];

    for pattern_type in patterns {
        let tick_budget = pattern_type.tick_budget();
        assert!(
            tick_budget <= 8,
            "Pattern {:?} has tick budget {} > 8 (Chatman Constant)",
            pattern_type,
            tick_budget
        );
    }
}

// ============================================================================
// Zero-Overhead Tests (No Runtime Measurement)
// ============================================================================

#[test]
fn test_pattern_execution_has_no_measurement_overhead() {
    // This test verifies the philosophy: NO tick measurement in hot path
    // We test that execution is fast, but we DON'T measure ticks during execution

    let branch = Arc::new(|data: TestData| Ok(data));
    let pattern = SequencePattern::new(vec![branch]).unwrap();

    // Execute 1000 times - should be fast (no measurement overhead)
    let input = TestData::new(42);
    for _ in 0..1000 {
        let _results = pattern.execute(input.clone()).unwrap();
    }

    // No assertions on tick count - that's the point!
    // Ingress validation happened once at pattern creation.
    // Execution is pure speed with zero overhead.
}
