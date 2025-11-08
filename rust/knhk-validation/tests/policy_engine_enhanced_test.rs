// rust/knhk-validation/tests/policy_engine_enhanced_test.rs
// Chicago TDD tests for enhanced PolicyEngine with Rego support preparation
// Tests focus on behavior: policy evaluation, context handling, unified evaluation

#![cfg(feature = "policy-engine")]

extern crate alloc;
use alloc::vec;
use knhk_validation::policy_engine::{BuiltinPolicy, PolicyContext, PolicyEngine, PolicyViolation};

#[test]
fn test_policy_context_creation() {
    // Arrange & Act: Create policy context with various fields
    let context = PolicyContext {
        run_len: Some(9),
        ticks: Some(10),
        receipt: Some(("receipt-1".to_string(), vec![1, 2, 3], vec![4, 5, 6])),
        additional: alloc::collections::BTreeMap::new(),
    };

    // Assert: Context fields set correctly
    assert_eq!(context.run_len, Some(9));
    assert_eq!(context.ticks, Some(10));
    assert!(context.receipt.is_some());
}

#[test]
fn test_policy_context_default() {
    // Arrange & Act: Create default context
    let context = PolicyContext::default();

    // Assert: All fields are None or empty
    assert_eq!(context.run_len, None);
    assert_eq!(context.ticks, None);
    assert_eq!(context.receipt, None);
    assert!(context.additional.is_empty());
}

#[test]
fn test_evaluate_all_with_guard_violation() {
    // Arrange: Create engine and context with guard violation
    let engine = PolicyEngine::new();
    let context = PolicyContext {
        run_len: Some(9), // Violation: > 8
        ticks: None,
        receipt: None,
        additional: alloc::collections::BTreeMap::new(),
    };

    // Act: Evaluate all policies
    let violations = engine.evaluate_all(&context);

    // Assert: Guard constraint violation detected
    assert_eq!(violations.len(), 1);
    match &violations[0] {
        PolicyViolation::GuardConstraintViolation {
            actual_run_len,
            max_run_len,
            ..
        } => {
            assert_eq!(*actual_run_len, 9);
            assert_eq!(*max_run_len, 8);
        }
        _ => panic!("Expected GuardConstraintViolation"),
    }
}

#[test]
fn test_evaluate_all_with_performance_violation() {
    // Arrange: Create engine and context with performance violation
    let engine = PolicyEngine::new();
    let context = PolicyContext {
        run_len: None,
        ticks: Some(10), // Violation: > 8
        receipt: None,
        additional: alloc::collections::BTreeMap::new(),
    };

    // Act: Evaluate all policies
    let violations = engine.evaluate_all(&context);

    // Assert: Performance budget violation detected
    assert_eq!(violations.len(), 1);
    match &violations[0] {
        PolicyViolation::PerformanceBudgetViolation {
            actual_ticks,
            max_ticks,
            ..
        } => {
            assert_eq!(*actual_ticks, 10);
            assert_eq!(*max_ticks, 8);
        }
        _ => panic!("Expected PerformanceBudgetViolation"),
    }
}

#[test]
fn test_evaluate_all_with_receipt_violation() {
    // Arrange: Create engine and context with receipt violation
    let engine = PolicyEngine::new();
    let context = PolicyContext {
        run_len: None,
        ticks: None,
        receipt: Some((
            "receipt-1".to_string(),
            vec![1, 2, 3], // Expected hash
            vec![4, 5, 6], // Actual hash (different)
        )),
        additional: alloc::collections::BTreeMap::new(),
    };

    // Act: Evaluate all policies
    let violations = engine.evaluate_all(&context);

    // Assert: Receipt validation violation detected
    assert_eq!(violations.len(), 1);
    match &violations[0] {
        PolicyViolation::ReceiptValidationViolation { receipt_id, .. } => {
            assert_eq!(receipt_id, "receipt-1");
        }
        _ => panic!("Expected ReceiptValidationViolation"),
    }
}

#[test]
fn test_evaluate_all_with_multiple_violations() {
    // Arrange: Create engine and context with multiple violations
    let engine = PolicyEngine::new();
    let context = PolicyContext {
        run_len: Some(9), // Violation
        ticks: Some(10),  // Violation
        receipt: Some((
            "receipt-1".to_string(),
            vec![1, 2, 3],
            vec![4, 5, 6], // Violation
        )),
        additional: alloc::collections::BTreeMap::new(),
    };

    // Act: Evaluate all policies
    let violations = engine.evaluate_all(&context);

    // Assert: All violations detected
    assert_eq!(violations.len(), 3);

    let guard_violations: Vec<_> = violations
        .iter()
        .filter(|v| matches!(v, PolicyViolation::GuardConstraintViolation { .. }))
        .collect();
    let perf_violations: Vec<_> = violations
        .iter()
        .filter(|v| matches!(v, PolicyViolation::PerformanceBudgetViolation { .. }))
        .collect();
    let receipt_violations: Vec<_> = violations
        .iter()
        .filter(|v| matches!(v, PolicyViolation::ReceiptValidationViolation { .. }))
        .collect();

    assert_eq!(guard_violations.len(), 1);
    assert_eq!(perf_violations.len(), 1);
    assert_eq!(receipt_violations.len(), 1);
}

#[test]
fn test_evaluate_all_with_no_violations() {
    // Arrange: Create engine and context with valid values
    let engine = PolicyEngine::new();
    let context = PolicyContext {
        run_len: Some(8), // Valid: ≤ 8
        ticks: Some(8),   // Valid: ≤ 8
        receipt: Some((
            "receipt-1".to_string(),
            vec![1, 2, 3],
            vec![1, 2, 3], // Valid: hashes match
        )),
        additional: alloc::collections::BTreeMap::new(),
    };

    // Act: Evaluate all policies
    let violations = engine.evaluate_all(&context);

    // Assert: No violations detected
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_evaluate_all_with_partial_context() {
    // Arrange: Create engine and context with only run_len
    let engine = PolicyEngine::new();
    let context = PolicyContext {
        run_len: Some(9), // Violation
        ticks: None,
        receipt: None,
        additional: alloc::collections::BTreeMap::new(),
    };

    // Act: Evaluate all policies
    let violations = engine.evaluate_all(&context);

    // Assert: Only guard constraint violation detected
    assert_eq!(violations.len(), 1);
    assert!(matches!(
        &violations[0],
        PolicyViolation::GuardConstraintViolation { .. }
    ));
}

#[test]
fn test_evaluate_all_with_custom_policies() {
    // Arrange: Create engine with only guard constraint policy
    let engine = PolicyEngine::with_policies(vec![BuiltinPolicy::GuardConstraint]);
    let context = PolicyContext {
        run_len: Some(9), // Violation
        ticks: Some(10),  // Violation (but policy not enabled)
        receipt: None,
        additional: alloc::collections::BTreeMap::new(),
    };

    // Act: Evaluate all policies
    let violations = engine.evaluate_all(&context);

    // Assert: Only guard constraint violation detected (performance policy not enabled)
    assert_eq!(violations.len(), 1);
    assert!(matches!(
        &violations[0],
        PolicyViolation::GuardConstraintViolation { .. }
    ));
}

#[test]
fn test_policy_context_additional_fields() {
    // Arrange: Create context with additional fields
    let mut additional = alloc::collections::BTreeMap::new();
    additional.insert("custom_key".to_string(), "custom_value".to_string());

    let context = PolicyContext {
        run_len: None,
        ticks: None,
        receipt: None,
        additional,
    };

    // Act & Assert: Additional fields preserved
    assert_eq!(context.additional.len(), 1);
    assert_eq!(
        context.additional.get("custom_key"),
        Some(&"custom_value".to_string())
    );
}
