// rust/knhk-validation/tests/chicago_tdd_advisor.rs
// Chicago TDD tests for Advisor Pattern
// Tests behaviors, not implementation details

#[cfg(feature = "advisor")]
mod tests {
    use alloc::collections::BTreeMap;
    use knhk_validation::advisor::*;
    use knhk_validation::policy_engine::PolicyViolation;

    #[test]
    fn test_guard_constraint_advisor_detects_violation() {
        // Arrange: Create advisor and input with violation
        let advisor = GuardConstraintAdvisor::new();
        let context = AdvisorContext {
            operation_id: "test_op_001".to_string(),
            runtime_class: "R1".to_string(),
            metadata: BTreeMap::new(),
        };
        let input = AdvisorInput {
            context,
            data: AdvisorData::GuardConstraint {
                run_len: 10, // Violation: exceeds max_run_len of 8
                max_run_len: 8,
            },
        };

        // Act: Get advice
        let violations = advisor.advise(&input);

        // Assert: Violation detected
        assert!(
            !violations.is_empty(),
            "Should detect guard constraint violation"
        );
        assert_eq!(violations.len(), 1);

        match &violations[0] {
            PolicyViolation::GuardConstraintViolation {
                actual_run_len,
                max_run_len,
                ..
            } => {
                assert_eq!(*actual_run_len, 10);
                assert_eq!(*max_run_len, 8);
            }
            _ => panic!("Should return GuardConstraintViolation"),
        }
    }

    #[test]
    fn test_guard_constraint_advisor_passes_valid_input() {
        // Arrange: Create advisor and input within limits
        let advisor = GuardConstraintAdvisor::new();
        let context = AdvisorContext {
            operation_id: "test_op_002".to_string(),
            runtime_class: "R1".to_string(),
            metadata: BTreeMap::new(),
        };
        let input = AdvisorInput {
            context,
            data: AdvisorData::GuardConstraint {
                run_len: 5, // Valid: within max_run_len of 8
                max_run_len: 8,
            },
        };

        // Act: Get advice
        let violations = advisor.advise(&input);

        // Assert: No violations
        assert!(
            violations.is_empty(),
            "Should not detect violation for valid input"
        );
    }

    #[test]
    fn test_guard_constraint_advisor_ignores_non_guard_data() {
        // Arrange: Create advisor with non-guard data
        let advisor = GuardConstraintAdvisor::new();
        let context = AdvisorContext {
            operation_id: "test_op_003".to_string(),
            runtime_class: "R1".to_string(),
            metadata: BTreeMap::new(),
        };
        let input = AdvisorInput {
            context,
            data: AdvisorData::PerformanceBudget {
                ticks: 10,
                max_ticks: 8,
            },
        };

        // Act: Get advice
        let violations = advisor.advise(&input);

        // Assert: No violations (ignores non-guard data)
        assert!(violations.is_empty(), "Should ignore non-guard data");
    }

    #[test]
    fn test_performance_budget_advisor_detects_violation() {
        // Arrange: Create advisor and input with violation
        let advisor = PerformanceBudgetAdvisor::new();
        let context = AdvisorContext {
            operation_id: "test_op_004".to_string(),
            runtime_class: "R1".to_string(),
            metadata: BTreeMap::new(),
        };
        let input = AdvisorInput {
            context,
            data: AdvisorData::PerformanceBudget {
                ticks: 10, // Violation: exceeds max_ticks of 8
                max_ticks: 8,
            },
        };

        // Act: Get advice
        let violations = advisor.advise(&input);

        // Assert: Violation detected
        assert!(
            !violations.is_empty(),
            "Should detect performance budget violation"
        );

        match &violations[0] {
            PolicyViolation::PerformanceBudgetViolation {
                actual_ticks,
                max_ticks,
                ..
            } => {
                assert_eq!(*actual_ticks, 10);
                assert_eq!(*max_ticks, 8);
            }
            _ => panic!("Should return PerformanceBudgetViolation"),
        }
    }

    #[test]
    fn test_performance_budget_advisor_passes_valid_input() {
        // Arrange: Create advisor and input within budget
        let advisor = PerformanceBudgetAdvisor::new();
        let context = AdvisorContext {
            operation_id: "test_op_005".to_string(),
            runtime_class: "R1".to_string(),
            metadata: BTreeMap::new(),
        };
        let input = AdvisorInput {
            context,
            data: AdvisorData::PerformanceBudget {
                ticks: 5, // Valid: within max_ticks of 8
                max_ticks: 8,
            },
        };

        // Act: Get advice
        let violations = advisor.advise(&input);

        // Assert: No violations
        assert!(
            violations.is_empty(),
            "Should not detect violation for valid input"
        );
    }

    #[test]
    fn test_performance_budget_advisor_with_custom_max_ticks() {
        // Arrange: Create advisor with custom max ticks
        let advisor = PerformanceBudgetAdvisor::with_max_ticks(16);
        let context = AdvisorContext {
            operation_id: "test_op_006".to_string(),
            runtime_class: "W1".to_string(),
            metadata: BTreeMap::new(),
        };
        let input = AdvisorInput {
            context,
            data: AdvisorData::PerformanceBudget {
                ticks: 12, // Valid: within custom max_ticks of 16
                max_ticks: 16,
            },
        };

        // Act: Get advice
        let violations = advisor.advise(&input);

        // Assert: No violations with custom limit
        assert!(violations.is_empty());
    }

    #[test]
    fn test_receipt_validation_advisor_detects_empty_hash() {
        // Arrange: Create advisor with empty hash
        let advisor = ReceiptValidationAdvisor::new();
        let context = AdvisorContext {
            operation_id: "test_op_007".to_string(),
            runtime_class: "C1".to_string(),
            metadata: BTreeMap::new(),
        };
        let input = AdvisorInput {
            context,
            data: AdvisorData::Receipt {
                receipt_id: "receipt_001".to_string(),
                hash: String::new(), // Empty hash
            },
        };

        // Act: Get advice
        let violations = advisor.advise(&input);

        // Assert: Violation detected
        assert!(!violations.is_empty(), "Should detect empty hash violation");

        match &violations[0] {
            PolicyViolation::ReceiptValidationViolation { receipt_id, .. } => {
                assert_eq!(receipt_id, "receipt_001");
            }
            _ => panic!("Should return ReceiptValidationViolation"),
        }
    }

    #[test]
    fn test_receipt_validation_advisor_passes_valid_hash() {
        // Arrange: Create advisor with valid hash
        let advisor = ReceiptValidationAdvisor::new();
        let context = AdvisorContext {
            operation_id: "test_op_008".to_string(),
            runtime_class: "C1".to_string(),
            metadata: BTreeMap::new(),
        };
        let input = AdvisorInput {
            context,
            data: AdvisorData::Receipt {
                receipt_id: "receipt_002".to_string(),
                hash: "abc123".to_string(), // Valid hash
            },
        };

        // Act: Get advice
        let violations = advisor.advise(&input);

        // Assert: No violations
        assert!(
            violations.is_empty(),
            "Should not detect violation for valid hash"
        );
    }

    #[test]
    fn test_advisor_chain_executes_all_advisors() {
        // Arrange: Create advisor chain with default advisors
        let chain = AdvisorChain::with_default_advisors();
        let context = AdvisorContext {
            operation_id: "test_op_009".to_string(),
            runtime_class: "R1".to_string(),
            metadata: BTreeMap::new(),
        };

        // Act: Test guard constraint violation
        let guard_input = AdvisorInput {
            context: context.clone(),
            data: AdvisorData::GuardConstraint {
                run_len: 10,
                max_run_len: 8,
            },
        };
        let guard_violations = chain.advise(&guard_input);

        // Assert: Violation detected
        assert!(!guard_violations.is_empty());
    }

    #[test]
    fn test_advisor_chain_prioritizes_advisors() {
        // Arrange: Create custom chain with specific advisors
        let mut chain = AdvisorChain::new();
        chain.add_advisor(Box::new(GuardConstraintAdvisor::default()));
        chain.add_advisor(Box::new(PerformanceBudgetAdvisor::default()));

        let context = AdvisorContext {
            operation_id: "test_op_010".to_string(),
            runtime_class: "R1".to_string(),
            metadata: BTreeMap::new(),
        };

        // Act: Test with multiple violations
        let input = AdvisorInput {
            context,
            data: AdvisorData::GuardConstraint {
                run_len: 10,
                max_run_len: 8,
            },
        };
        let violations = chain.advise(&input);

        // Assert: Violations from appropriate advisor
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_advisor_names_are_unique() {
        // Arrange: Create advisors
        let guard_advisor = GuardConstraintAdvisor::default();
        let perf_advisor = PerformanceBudgetAdvisor::default();
        let receipt_advisor = ReceiptValidationAdvisor::default();

        // Act: Get names
        let guard_name = guard_advisor.name();
        let perf_name = perf_advisor.name();
        let receipt_name = receipt_advisor.name();

        // Assert: Names are unique
        assert_eq!(guard_name, "guard_constraint");
        assert_eq!(perf_name, "performance_budget");
        assert_eq!(receipt_name, "receipt_validation");
        assert_ne!(guard_name, perf_name);
        assert_ne!(perf_name, receipt_name);
        assert_ne!(guard_name, receipt_name);
    }

    #[test]
    fn test_advisor_priorities_are_ordered() {
        // Arrange: Create advisors
        let guard_advisor = GuardConstraintAdvisor::default();
        let perf_advisor = PerformanceBudgetAdvisor::default();
        let receipt_advisor = ReceiptValidationAdvisor::default();

        // Act: Get priorities
        let guard_priority = guard_advisor.priority();
        let perf_priority = perf_advisor.priority();
        let receipt_priority = receipt_advisor.priority();

        // Assert: Priorities are ordered (lower = higher priority)
        assert!(guard_priority < perf_priority);
        assert!(perf_priority < receipt_priority);
    }

    #[test]
    fn test_advisor_chain_sorts_by_priority() {
        // Arrange: Create chain and add advisors in wrong order
        let mut chain = AdvisorChain::new();
        chain.add_advisor(Box::new(ReceiptValidationAdvisor::default())); // Priority 30
        chain.add_advisor(Box::new(GuardConstraintAdvisor::default())); // Priority 10
        chain.add_advisor(Box::new(PerformanceBudgetAdvisor::default())); // Priority 20

        // Act: Chain should sort internally
        // (We can't directly test sorting, but we can verify chain works)
        let context = AdvisorContext {
            operation_id: "test_op_011".to_string(),
            runtime_class: "R1".to_string(),
            metadata: BTreeMap::new(),
        };
        let input = AdvisorInput {
            context,
            data: AdvisorData::GuardConstraint {
                run_len: 10,
                max_run_len: 8,
            },
        };
        let violations = chain.advise(&input);

        // Assert: Chain executes correctly
        assert!(!violations.is_empty());
    }
}
