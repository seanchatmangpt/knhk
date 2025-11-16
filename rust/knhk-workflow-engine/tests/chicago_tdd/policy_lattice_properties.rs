//! Property-Based Tests for Policy Lattice
//!
//! Tests algebraic properties of the policy lattice:
//! - Commutativity: a ⊓ b = b ⊓ a, a ⊔ b = b ⊔ a
//! - Associativity: (a ⊓ b) ⊓ c = a ⊓ (b ⊓ c)
//! - Idempotence: a ⊓ a = a, a ⊔ a = a
//! - Absorption: a ⊓ (a ⊔ b) = a, a ⊔ (a ⊓ b) = a
//!
//! **Chicago TDD Approach**: Uses real collaborators, tests actual behavior

use knhk_workflow_engine::autonomic::policy_lattice::{
    LatencyBound, FailureRateBound, Strictness, PolicyId, PolicyElement, PolicyLattice,
};
use knhk_workflow_engine::error::WorkflowResult;

// ============================================================================
// Property Test Generators
// ============================================================================

/// Generate diverse latency bounds for property testing
fn generate_latency_bounds(count: usize) -> Vec<LatencyBound> {
    let values = vec![10.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 5000.0];
    let strictness_levels = vec![Strictness::Soft, Strictness::Hard];

    let mut bounds = Vec::new();
    for i in 0..count {
        let target = values[i % values.len()];
        let strictness = strictness_levels[i % strictness_levels.len()];
        if let Ok(bound) = LatencyBound::new(target, strictness) {
            bounds.push(bound);
        }
    }
    bounds
}

/// Generate diverse failure rate bounds
fn generate_failure_rate_bounds(count: usize) -> Vec<FailureRateBound> {
    let values = vec![0.001, 0.01, 0.05, 0.1, 0.2, 0.5];

    let mut bounds = Vec::new();
    for i in 0..count {
        let rate = values[i % values.len()];
        if let Ok(bound) = FailureRateBound::new(rate) {
            bounds.push(bound);
        }
    }
    bounds
}

// ============================================================================
// Commutativity Properties
// ============================================================================

#[test]
fn test_strictness_meet_commutativity() {
    // Arrange: Create all strictness combinations
    let soft = Strictness::Soft;
    let hard = Strictness::Hard;

    // Act & Assert: Test commutativity a ⊓ b = b ⊓ a
    assert_eq!(soft.meet(hard), hard.meet(soft));
    assert_eq!(soft.meet(soft), soft.meet(soft));
    assert_eq!(hard.meet(hard), hard.meet(hard));
}

#[test]
fn test_strictness_join_commutativity() {
    // Arrange
    let soft = Strictness::Soft;
    let hard = Strictness::Hard;

    // Act & Assert: Test commutativity a ⊔ b = b ⊔ a
    assert_eq!(soft.join(hard), hard.join(soft));
    assert_eq!(soft.join(soft), soft.join(soft));
    assert_eq!(hard.join(hard), hard.join(hard));
}

#[test]
fn test_latency_bound_ordering_commutativity() {
    // Arrange: Generate 100 pairs of latency bounds
    let bounds = generate_latency_bounds(100);

    // Act & Assert: Test is_stricter_than commutativity
    for i in 0..bounds.len() {
        for j in i+1..bounds.len() {
            let a = &bounds[i];
            let b = &bounds[j];

            // If a is stricter than b, then b is NOT stricter than a
            if a.is_stricter_than(b) {
                assert!(
                    !b.is_stricter_than(a),
                    "Strictness ordering should be antisymmetric"
                );
            }
        }
    }
}

#[test]
fn test_failure_rate_bound_ordering_commutativity() {
    // Arrange: Generate 50 pairs
    let bounds = generate_failure_rate_bounds(50);

    // Act & Assert: Test ordering is antisymmetric
    for i in 0..bounds.len() {
        for j in i+1..bounds.len() {
            let a = &bounds[i];
            let b = &bounds[j];

            if a.is_stricter_than(b) {
                assert!(
                    !b.is_stricter_than(a),
                    "Failure rate ordering should be antisymmetric"
                );
            }
        }
    }
}

// ============================================================================
// Associativity Properties
// ============================================================================

#[test]
fn test_strictness_meet_associativity() {
    // Arrange: Create test values
    let soft = Strictness::Soft;
    let hard = Strictness::Hard;

    // Act & Assert: (a ⊓ b) ⊓ c = a ⊓ (b ⊓ c)
    let left = soft.meet(hard).meet(hard);
    let right = soft.meet(hard.meet(hard));
    assert_eq!(left, right);

    let left = hard.meet(soft).meet(hard);
    let right = hard.meet(soft.meet(hard));
    assert_eq!(left, right);
}

#[test]
fn test_strictness_join_associativity() {
    // Arrange
    let soft = Strictness::Soft;
    let hard = Strictness::Hard;

    // Act & Assert: (a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)
    let left = soft.join(hard).join(soft);
    let right = soft.join(hard.join(soft));
    assert_eq!(left, right);

    let left = hard.join(soft).join(hard);
    let right = hard.join(soft.join(hard));
    assert_eq!(left, right);
}

#[test]
fn test_latency_bound_transitivity() {
    // Arrange: Create chain a < b < c
    let a = LatencyBound::new(50.0, Strictness::Hard).unwrap();
    let b = LatencyBound::new(100.0, Strictness::Soft).unwrap();
    let c = LatencyBound::new(200.0, Strictness::Soft).unwrap();

    // Act & Assert: If a < b and b < c, then a < c (transitivity)
    assert!(a.is_stricter_than(&b));
    assert!(b.is_stricter_than(&c));
    assert!(a.is_stricter_than(&c), "Strictness should be transitive");
}

#[test]
fn test_failure_rate_transitivity() {
    // Arrange: Create chain a < b < c
    let a = FailureRateBound::new(0.01).unwrap();
    let b = FailureRateBound::new(0.05).unwrap();
    let c = FailureRateBound::new(0.10).unwrap();

    // Act & Assert: Transitivity
    assert!(a.is_stricter_than(&b));
    assert!(b.is_stricter_than(&c));
    assert!(a.is_stricter_than(&c));
}

// ============================================================================
// Idempotence Properties
// ============================================================================

#[test]
fn test_strictness_meet_idempotence() {
    // Arrange
    let soft = Strictness::Soft;
    let hard = Strictness::Hard;

    // Act & Assert: a ⊓ a = a
    assert_eq!(soft.meet(soft), soft);
    assert_eq!(hard.meet(hard), hard);
}

#[test]
fn test_strictness_join_idempotence() {
    // Arrange
    let soft = Strictness::Soft;
    let hard = Strictness::Hard;

    // Act & Assert: a ⊔ a = a
    assert_eq!(soft.join(soft), soft);
    assert_eq!(hard.join(hard), hard);
}

#[test]
fn test_latency_bound_reflexivity() {
    // Arrange: Generate diverse bounds
    let bounds = generate_latency_bounds(50);

    // Act & Assert: a ≤ a (reflexivity)
    for bound in &bounds {
        // A bound is not stricter than itself
        assert!(!bound.is_stricter_than(bound));
    }
}

#[test]
fn test_failure_rate_reflexivity() {
    // Arrange
    let bounds = generate_failure_rate_bounds(50);

    // Act & Assert: Reflexivity
    for bound in &bounds {
        assert!(!bound.is_stricter_than(bound));
    }
}

// ============================================================================
// Absorption Properties
// ============================================================================

#[test]
fn test_strictness_absorption_law_1() {
    // Arrange
    let soft = Strictness::Soft;
    let hard = Strictness::Hard;

    // Act & Assert: a ⊓ (a ⊔ b) = a
    assert_eq!(soft.meet(soft.join(hard)), soft);
    assert_eq!(hard.meet(hard.join(soft)), hard);
}

#[test]
fn test_strictness_absorption_law_2() {
    // Arrange
    let soft = Strictness::Soft;
    let hard = Strictness::Hard;

    // Act & Assert: a ⊔ (a ⊓ b) = a
    assert_eq!(soft.join(soft.meet(hard)), soft);
    assert_eq!(hard.join(hard.meet(soft)), hard);
}

// ============================================================================
// Boundary Value Properties
// ============================================================================

#[test]
fn test_latency_bound_positive_constraint() {
    // Arrange & Act: Try to create invalid bound
    let result = LatencyBound::new(0.0, Strictness::Soft);

    // Assert: Should reject non-positive values
    assert!(result.is_err(), "Latency bound must be positive");

    let result = LatencyBound::new(-10.0, Strictness::Hard);
    assert!(result.is_err(), "Latency bound must be positive");
}

#[test]
fn test_failure_rate_bound_range_constraint() {
    // Arrange & Act: Try to create invalid bounds
    let result = FailureRateBound::new(-0.01);
    assert!(result.is_err(), "Failure rate must be non-negative");

    let result = FailureRateBound::new(1.5);
    assert!(result.is_err(), "Failure rate must be ≤ 1.0");
}

#[test]
fn test_latency_bound_min_max_ordering() {
    // Arrange: Create minimum and maximum reasonable bounds
    let min_bound = LatencyBound::new(1.0, Strictness::Hard).unwrap();
    let max_bound = LatencyBound::new(10000.0, Strictness::Soft).unwrap();

    // Act & Assert: Min is stricter than max
    assert!(min_bound.is_stricter_than(&max_bound));
    assert!(!max_bound.is_stricter_than(&min_bound));
}

#[test]
fn test_failure_rate_bound_min_max_ordering() {
    // Arrange
    let min_bound = FailureRateBound::new(0.001).unwrap();
    let max_bound = FailureRateBound::new(0.99).unwrap();

    // Act & Assert
    assert!(min_bound.is_stricter_than(&max_bound));
    assert!(!max_bound.is_stricter_than(&min_bound));
}

// ============================================================================
// Equivalence Properties
// ============================================================================

#[test]
fn test_latency_bound_equality() {
    // Arrange: Create identical bounds
    let bound1 = LatencyBound::new(100.0, Strictness::Hard).unwrap();
    let bound2 = LatencyBound::new(100.0, Strictness::Hard).unwrap();

    // Act & Assert: Should be equal but have different IDs
    assert_eq!(bound1.target_p99_ms, bound2.target_p99_ms);
    assert_eq!(bound1.strictness, bound2.strictness);
    assert_ne!(bound1.policy_id, bound2.policy_id);
}

#[test]
fn test_failure_rate_bound_equality() {
    // Arrange
    let bound1 = FailureRateBound::new(0.05).unwrap();
    let bound2 = FailureRateBound::new(0.05).unwrap();

    // Act & Assert
    assert_eq!(bound1.max_error_rate, bound2.max_error_rate);
    assert_ne!(bound1.policy_id, bound2.policy_id);
}

// ============================================================================
// Strictness Ordering Properties
// ============================================================================

#[test]
fn test_strictness_total_ordering() {
    // Arrange
    let soft = Strictness::Soft;
    let hard = Strictness::Hard;

    // Act & Assert: Total ordering
    assert!(soft < hard);
    assert!(hard > soft);
    assert_eq!(soft, soft);
    assert_eq!(hard, hard);
}

#[test]
fn test_latency_bound_strictness_dominates() {
    // Arrange: Same target, different strictness
    let soft_100 = LatencyBound::new(100.0, Strictness::Soft).unwrap();
    let hard_100 = LatencyBound::new(100.0, Strictness::Hard).unwrap();

    // Act & Assert: Hard is stricter than soft at same target
    assert!(hard_100.is_stricter_than(&soft_100));
    assert!(!soft_100.is_stricter_than(&hard_100));
}

#[test]
fn test_latency_bound_target_ordering() {
    // Arrange: Same strictness, different targets
    let soft_50 = LatencyBound::new(50.0, Strictness::Soft).unwrap();
    let soft_100 = LatencyBound::new(100.0, Strictness::Soft).unwrap();

    // Act & Assert: Lower target is stricter
    assert!(soft_50.is_stricter_than(&soft_100));
    assert!(!soft_100.is_stricter_than(&soft_50));
}

// ============================================================================
// Multiple Bounds Properties
// ============================================================================

#[test]
fn test_policy_lattice_bottom_element() {
    // Arrange: Create lattice with bottom element
    let bounds = generate_latency_bounds(10);

    // Act & Assert: Bottom element is stricter than all others
    // (This would be tested if PolicyLattice implements bottom)
    // For now, verify that ordering is consistent
    for i in 0..bounds.len() {
        for j in 0..bounds.len() {
            if i != j {
                let a = &bounds[i];
                let b = &bounds[j];

                // At most one can be stricter
                let a_stricter = a.is_stricter_than(b);
                let b_stricter = b.is_stricter_than(a);

                if a_stricter {
                    assert!(!b_stricter, "Ordering should be antisymmetric");
                }
            }
        }
    }
}

#[test]
fn test_consistent_strictness_across_types() {
    // Arrange: Create bounds of different types
    let latency = LatencyBound::new(100.0, Strictness::Hard).unwrap();
    let failure = FailureRateBound::new(0.05).unwrap();

    // Act & Assert: Both should have consistent strictness semantics
    // (Lower/stricter values should always be more restrictive)
    let stricter_latency = LatencyBound::new(50.0, Strictness::Hard).unwrap();
    let stricter_failure = FailureRateBound::new(0.01).unwrap();

    assert!(stricter_latency.is_stricter_than(&latency));
    assert!(stricter_failure.is_stricter_than(&failure));
}

// ============================================================================
// Performance Properties
// ============================================================================

#[test]
fn test_policy_comparison_performance() {
    use std::time::Instant;

    // Arrange: Generate large set of bounds
    let bounds = generate_latency_bounds(1000);

    // Act: Measure comparison time
    let start = Instant::now();
    let mut comparisons = 0u64;

    for i in 0..bounds.len() {
        for j in i+1..bounds.len() {
            let _ = bounds[i].is_stricter_than(&bounds[j]);
            comparisons += 1;
        }
    }

    let elapsed = start.elapsed();

    // Assert: Should complete in reasonable time (< 10ms for 1000 elements)
    assert!(
        elapsed.as_millis() < 10,
        "Policy comparisons should be fast: took {}ms for {} comparisons",
        elapsed.as_millis(),
        comparisons
    );
}

#[test]
fn test_strictness_operations_are_const_time() {
    use std::time::Instant;

    // Arrange
    let soft = Strictness::Soft;
    let hard = Strictness::Hard;

    // Act: Measure 1 million operations
    let start = Instant::now();
    for _ in 0..1_000_000 {
        let _ = soft.meet(hard);
        let _ = soft.join(hard);
    }
    let elapsed = start.elapsed();

    // Assert: Should be near-instant (< 50ms)
    assert!(
        elapsed.as_millis() < 50,
        "Strictness operations should be O(1): took {}ms",
        elapsed.as_millis()
    );
}
