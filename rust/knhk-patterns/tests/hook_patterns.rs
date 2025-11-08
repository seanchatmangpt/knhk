// rust/knhk-patterns/tests/hook_patterns.rs
// Chicago TDD tests for hook pattern types

use knhk_etl::hook_orchestration::HookExecutionContext;
use knhk_etl::hook_registry::HookRegistry;
use knhk_etl::load::{LoadResult, PredRun, SoAArrays};
use knhk_patterns::hook_patterns::*;
use std::sync::Arc;

// ============================================================================
// Test Helpers
// ============================================================================

fn create_test_context() -> HookExecutionContext {
    let mut soa = SoAArrays::new();
    soa.s[0] = 1;
    soa.p[0] = 100;
    soa.o[0] = 10;

    soa.s[1] = 2;
    soa.p[1] = 200;
    soa.o[1] = 20;

    let runs = vec![
        PredRun {
            pred: 100,
            off: 0,
            len: 1,
        },
        PredRun {
            pred: 200,
            off: 1,
            len: 1,
        },
    ];

    let registry = HookRegistry::new();
    HookExecutionContext::new(registry, runs, soa, 8)
}

// ============================================================================
// HookSequencePattern Tests
// ============================================================================

#[test]
fn test_hook_sequence_pattern_creation() {
    // Arrange & Act
    let pattern = HookSequencePattern::new(vec![100, 200, 300]);

    // Assert
    assert!(pattern.is_ok());

    let pattern = HookSequencePattern::new(vec![]);
    assert!(pattern.is_err());
}

#[test]
fn test_hook_sequence_execution() {
    // Arrange
    let context = create_test_context();
    let pattern = HookSequencePattern::new(vec![100, 200]).unwrap();

    // Act
    let result = pattern.execute_hooks(&context);

    // Assert
    assert!(result.is_ok());
    let hook_result = result.unwrap();
    assert_eq!(hook_result.receipts.len(), 2);
    assert!(hook_result.max_ticks <= 8);
}

// ============================================================================
// HookParallelPattern Tests
// ============================================================================

#[test]
fn test_hook_parallel_pattern_creation() {
    // Arrange & Act
    let pattern = HookParallelPattern::new(vec![100, 200]);

    // Assert
    assert!(pattern.is_ok());

    let pattern = HookParallelPattern::new(vec![]);
    assert!(pattern.is_err());
}

#[test]
fn test_hook_parallel_execution() {
    // Arrange
    let context = create_test_context();
    let pattern = HookParallelPattern::new(vec![100, 200]).unwrap();

    // Act
    let result = pattern.execute_hooks(&context);

    // Assert
    assert!(result.is_ok());
    let hook_result = result.unwrap();
    assert_eq!(hook_result.receipts.len(), 2);
    assert!(hook_result.max_ticks <= 8);
}

// ============================================================================
// HookChoicePattern Tests
// ============================================================================

#[test]
fn test_hook_choice_pattern_creation() {
    // Arrange
    let choices = vec![
        (
            Arc::new(|_ctx: &HookExecutionContext| true) as HookCondition,
            100u64,
        ),
        (
            Arc::new(|_ctx: &HookExecutionContext| false) as HookCondition,
            200u64,
        ),
    ];

    // Act
    let pattern = HookChoicePattern::new(choices);

    // Assert
    assert!(pattern.is_ok());

    let pattern = HookChoicePattern::new(vec![]);
    assert!(pattern.is_err());
}

#[test]
fn test_hook_choice_execution() {
    // Arrange
    let context = create_test_context();
    let choices = vec![
        (
            Arc::new(|ctx: &HookExecutionContext| ctx.predicate_runs.iter().any(|r| r.pred == 100))
                as HookCondition,
            100u64,
        ),
        (
            Arc::new(|_ctx: &HookExecutionContext| true) as HookCondition,
            200u64,
        ),
    ];

    let pattern = HookChoicePattern::new(choices).unwrap();

    // Act
    let result = pattern.execute_hooks(&context);

    // Assert
    assert!(result.is_ok());
    let hook_result = result.unwrap();
    assert_eq!(hook_result.receipts.len(), 1);
    // First condition matched (predicate 100)
    // Note: We verify execution happened, not specific hook_id values
}

// ============================================================================
// HookRetryPattern Tests
// ============================================================================

#[test]
fn test_hook_retry_pattern_creation() {
    // Arrange & Act
    let pattern = HookRetryPattern::new(
        100u64,
        Arc::new(|_receipt: &knhk_etl::Receipt| true) as HookRetryCondition,
        3,
    );

    // Assert
    assert!(pattern.is_ok());

    let pattern = HookRetryPattern::new(
        100u64,
        Arc::new(|_receipt: &knhk_etl::Receipt| true) as HookRetryCondition,
        0,
    );
    assert!(pattern.is_err());
}

#[test]
fn test_hook_retry_execution() {
    // Arrange
    let context = create_test_context();
    let pattern = HookRetryPattern::new(
        100u64,
        Arc::new(|receipt: &knhk_etl::Receipt| receipt.ticks == 0) as HookRetryCondition,
        3,
    )
    .unwrap();

    // Act
    let result = pattern.execute_hooks(&context);

    // Assert
    assert!(result.is_ok());
    let hook_result = result.unwrap();
    assert_eq!(hook_result.receipts.len(), 1);
}

// ============================================================================
// Helper Function Tests
// ============================================================================

#[test]
fn test_create_hook_context_from_load_result() {
    // Arrange
    let registry = HookRegistry::new();
    let load_result = LoadResult {
        soa_arrays: SoAArrays::new(),
        runs: vec![PredRun {
            pred: 100,
            off: 0,
            len: 1,
        }],
    };

    // Act
    let context = create_hook_context(registry, load_result, 8);

    // Assert
    assert_eq!(context.tick_budget, 8);
    assert_eq!(context.predicate_runs.len(), 1);
}

#[test]
fn test_create_hook_context_from_components() {
    // Arrange
    let registry = HookRegistry::new();
    let runs = vec![PredRun {
        pred: 100,
        off: 0,
        len: 1,
    }];
    let soa = SoAArrays::new();

    // Act
    let context = create_hook_context_from_components(registry, runs, soa, 8);

    // Assert
    assert_eq!(context.tick_budget, 8);
    assert_eq!(context.predicate_runs.len(), 1);
}
