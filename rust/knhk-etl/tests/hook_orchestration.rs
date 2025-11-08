// rust/knhk-etl/tests/hook_orchestration.rs
// Chicago TDD tests for hook orchestration

use knhk_etl::hook_orchestration::{HookExecutionContext, HookExecutionPattern, HookOrchestrator};
use knhk_etl::hook_registry::HookRegistry;
use knhk_etl::load::{LoadResult, PredRun, SoAArrays};

// ============================================================================
// Test Helpers
// ============================================================================

fn create_test_load_result() -> LoadResult {
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

    LoadResult {
        soa_arrays: soa,
        runs,
    }
}

// ============================================================================
// Sequential Execution Tests
// ============================================================================

#[test]
fn test_sequential_hook_execution() {
    // Arrange
    let registry = HookRegistry::new();
    let load_result = create_test_load_result();
    let context = HookExecutionContext::from_load_result(registry, load_result, 8);

    let pattern = HookExecutionPattern::Sequence(vec![100, 200]);
    let orchestrator = HookOrchestrator::new();

    // Act
    let result = orchestrator.execute_with_pattern(&context, pattern);

    // Assert
    assert!(result.is_ok());
    let hook_result = result.unwrap();
    assert_eq!(hook_result.receipts.len(), 2);
    assert!(hook_result.max_ticks <= 8);
}

#[test]
fn test_sequential_execution_order() {
    // Arrange
    let registry = HookRegistry::new();
    let load_result = create_test_load_result();
    let context = HookExecutionContext::from_load_result(registry, load_result, 8);

    let pattern = HookExecutionPattern::Sequence(vec![200, 100]); // Reverse order
    let orchestrator = HookOrchestrator::new();

    // Act
    let result = orchestrator.execute_with_pattern(&context, pattern);

    // Assert
    assert!(result.is_ok());
    let hook_result = result.unwrap();
    assert_eq!(hook_result.receipts.len(), 2);
    // Receipts should be in pattern order
    assert_eq!(hook_result.receipts[0].hook_id, 200);
    assert_eq!(hook_result.receipts[1].hook_id, 100);
}

// ============================================================================
// Parallel Execution Tests
// ============================================================================

#[test]
#[cfg(feature = "parallel")]
fn test_parallel_hook_execution() {
    // Arrange
    let registry = HookRegistry::new();
    let load_result = create_test_load_result();
    let context = HookExecutionContext::from_load_result(registry, load_result, 8);

    let pattern = HookExecutionPattern::Parallel(vec![100, 200]);
    let orchestrator = HookOrchestrator::new();

    // Act
    let result = orchestrator.execute_with_pattern(&context, pattern);

    // Assert
    assert!(result.is_ok());
    let hook_result = result.unwrap();
    assert_eq!(hook_result.receipts.len(), 2);
    assert!(hook_result.max_ticks <= 8);
}

#[test]
#[cfg(not(feature = "parallel"))]
fn test_parallel_fallback_to_sequential() {
    // Arrange: When parallel feature is disabled, should fallback to sequential
    let registry = HookRegistry::new();
    let load_result = create_test_load_result();
    let context = HookExecutionContext::from_load_result(registry, load_result, 8);

    let pattern = HookExecutionPattern::Parallel(vec![100, 200]);
    let orchestrator = HookOrchestrator::new();

    // Act
    let result = orchestrator.execute_with_pattern(&context, pattern);

    // Assert: Should still work, just sequentially
    assert!(result.is_ok());
    let hook_result = result.unwrap();
    assert_eq!(hook_result.receipts.len(), 2);
}

// ============================================================================
// Conditional Routing Tests
// ============================================================================

#[test]
fn test_conditional_routing_first_match() {
    // Arrange
    let registry = HookRegistry::new();
    let load_result = create_test_load_result();
    let context = HookExecutionContext::from_load_result(registry, load_result, 8);

    let choices = vec![
        (
            Box::new(|ctx: &HookExecutionContext| ctx.predicate_runs.len() > 1) as Box<dyn Fn(&HookExecutionContext) -> bool + Send + Sync>,
            100u64,
        ),
        (
            Box::new(|_ctx: &HookExecutionContext| true) as Box<dyn Fn(&HookExecutionContext) -> bool + Send + Sync>,
            200u64,
        ),
    ];

    let pattern = HookExecutionPattern::Choice(choices);
    let orchestrator = HookOrchestrator::new();

    // Act
    let result = orchestrator.execute_with_pattern(&context, pattern);

    // Assert
    assert!(result.is_ok());
    let hook_result = result.unwrap();
    assert_eq!(hook_result.receipts.len(), 1);
    assert_eq!(hook_result.receipts[0].hook_id, 100); // First condition matched
}

#[test]
fn test_conditional_routing_no_match() {
    // Arrange
    let registry = HookRegistry::new();
    let load_result = create_test_load_result();
    let context = HookExecutionContext::from_load_result(registry, load_result, 8);

    let choices = vec![
        (
            Box::new(|_ctx: &HookExecutionContext| false) as Box<dyn Fn(&HookExecutionContext) -> bool + Send + Sync>,
            100u64,
        ),
        (
            Box::new(|_ctx: &HookExecutionContext| false) as Box<dyn Fn(&HookExecutionContext) -> bool + Send + Sync>,
            200u64,
        ),
    ];

    let pattern = HookExecutionPattern::Choice(choices);
    let orchestrator = HookOrchestrator::new();

    // Act
    let result = orchestrator.execute_with_pattern(&context, pattern);

    // Assert
    assert!(result.is_err());
}

// ============================================================================
// Retry Logic Tests
// ============================================================================

#[test]
fn test_retry_logic_success_on_first_attempt() {
    // Arrange
    let registry = HookRegistry::new();
    let load_result = create_test_load_result();
    let context = HookExecutionContext::from_load_result(registry, load_result, 8);

    // Condition: retry if ticks == 0 (failed)
    let should_retry = Box::new(|receipt: &knhk_etl::Receipt| receipt.ticks == 0)
        as Box<dyn Fn(&knhk_etl::Receipt) -> bool + Send + Sync>;

    let pattern = HookExecutionPattern::Retry {
        predicate: 100,
        should_retry,
        max_attempts: 3,
    };
    let orchestrator = HookOrchestrator::new();

    // Act
    let result = orchestrator.execute_with_pattern(&context, pattern);

    // Assert
    assert!(result.is_ok());
    let hook_result = result.unwrap();
    assert_eq!(hook_result.receipts.len(), 1);
}

#[test]
fn test_retry_logic_respects_max_attempts() {
    // Arrange
    let registry = HookRegistry::new();
    let load_result = create_test_load_result();
    let context = HookExecutionContext::from_load_result(registry, load_result, 8);

    // Condition: always retry (will hit max attempts)
    let should_retry = Box::new(|_receipt: &knhk_etl::Receipt| true)
        as Box<dyn Fn(&knhk_etl::Receipt) -> bool + Send + Sync>;

    let pattern = HookExecutionPattern::Retry {
        predicate: 100,
        should_retry,
        max_attempts: 2,
    };
    let orchestrator = HookOrchestrator::new();

    // Act
    let result = orchestrator.execute_with_pattern(&context, pattern);

    // Assert
    assert!(result.is_ok());
    let hook_result = result.unwrap();
    assert_eq!(hook_result.receipts.len(), 1);
    // Should have attempted 2 times (max_attempts)
}

// ============================================================================
// Receipt Aggregation Tests
// ============================================================================

#[test]
fn test_receipt_aggregation() {
    // Arrange
    let registry = HookRegistry::new();
    let load_result = create_test_load_result();
    let context = HookExecutionContext::from_load_result(registry, load_result, 8);

    let pattern = HookExecutionPattern::Sequence(vec![100, 200]);
    let orchestrator = HookOrchestrator::new();

    // Act
    let result = orchestrator.execute_with_pattern(&context, pattern);

    // Assert
    assert!(result.is_ok());
    let hook_result = result.unwrap();
    assert_eq!(hook_result.receipts.len(), 2);
    
    // Aggregated receipt should have max ticks
    assert!(hook_result.aggregated_receipt.ticks >= hook_result.receipts[0].ticks);
    assert!(hook_result.aggregated_receipt.ticks >= hook_result.receipts[1].ticks);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_tick_budget_validation() {
    // Arrange
    let registry = HookRegistry::new();
    let mut load_result = create_test_load_result();
    // Create a run that exceeds tick budget
    load_result.runs[0].len = 10; // Exceeds tick_budget of 8

    let context = HookExecutionContext::from_load_result(registry, load_result, 8);
    let pattern = HookExecutionPattern::Sequence(vec![100]);
    let orchestrator = HookOrchestrator::new();

    // Act
    let result = orchestrator.execute_with_pattern(&context, pattern);

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_empty_predicate_list() {
    // Arrange
    let registry = HookRegistry::new();
    let load_result = create_test_load_result();
    let context = HookExecutionContext::from_load_result(registry, load_result, 8);

    let pattern = HookExecutionPattern::Sequence(vec![]);
    let orchestrator = HookOrchestrator::new();

    // Act
    let result = orchestrator.execute_with_pattern(&context, pattern);

    // Assert: Should handle empty list gracefully
    assert!(result.is_ok());
    let hook_result = result.unwrap();
    assert_eq!(hook_result.receipts.len(), 0);
}

