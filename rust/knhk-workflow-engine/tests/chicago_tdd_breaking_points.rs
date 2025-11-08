//! Breaking Point Tests for Most Complex Patterns
//!
//! Tests designed to find breaking points by pushing patterns to their limits:
//! - Boundary violations (max values exceeded)
//! - Invalid inputs (malformed data)
//! - Resource exhaustion (memory, threads)
//! - State corruption (invalid transitions)
//! - Race conditions (concurrent execution)
//! - Error conditions (missing required data)

use knhk_workflow_engine::case::CaseId;
use knhk_workflow_engine::parser::WorkflowSpecId;
use knhk_workflow_engine::patterns::{
    advanced_control, PatternExecutionContext, PatternExecutionResult, PatternId,
};
use std::collections::{HashMap, HashSet};

// ============================================================================
// PATTERN 26: BLOCKING DISCRIMINATOR - BREAKING POINTS
// ============================================================================

#[tokio::test]
async fn test_pattern_26_breaking_point_max_branches() {
    // BREAKING POINT: Test with extremely large number of branches

    let (_, executor) = advanced_control::create_pattern_26();

    // Test 1: Max branches (should handle gracefully)
    let mut variables = HashMap::new();
    variables.insert("expected_branches".to_string(), "1000000".to_string());
    let mut arrived_from = HashSet::new();
    // Add 1000 arrived branches (much less than expected)
    for i in 0..1000 {
        arrived_from.insert(format!("branch_{}", i));
    }

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from,
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle large expected_branches");
    assert!(
        result.next_state.as_ref().unwrap().contains("waiting"),
        "BREAKING POINT: Should be waiting when branches < expected"
    );
}

#[tokio::test]
async fn test_pattern_26_breaking_point_invalid_expected_branches() {
    // BREAKING POINT: Invalid expected_branches value

    let (_, executor) = advanced_control::create_pattern_26();

    // Test 1: Negative number (should default or handle gracefully)
    let mut variables = HashMap::new();
    variables.insert("expected_branches".to_string(), "-1".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    // Should either fail gracefully or default to 1
    // Current implementation defaults to arrived_from.len().max(1)
    assert!(result.success, "Should handle negative expected_branches");
}

#[tokio::test]
async fn test_pattern_26_breaking_point_empty_arrived_from() {
    // BREAKING POINT: Empty arrived_from with expected_branches > 0

    let (_, executor) = advanced_control::create_pattern_26();

    let mut variables = HashMap::new();
    variables.insert("expected_branches".to_string(), "5".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(), // Empty
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle empty arrived_from");
    assert_eq!(
        result.next_activities.len(),
        0,
        "BREAKING POINT: Should not schedule activities when no branches arrived"
    );
}

// ============================================================================
// PATTERN 27: CANCELLING DISCRIMINATOR - BREAKING POINTS
// ============================================================================

#[tokio::test]
async fn test_pattern_27_breaking_point_malformed_all_branches() {
    // BREAKING POINT: Malformed all_branches string

    let (_, executor) = advanced_control::create_pattern_27();

    // Test 1: Empty string
    let mut variables = HashMap::new();
    variables.insert("all_branches".to_string(), "".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    // Should default to inferred branches or handle gracefully
    assert!(result.success, "Should handle empty all_branches");

    // Test 2: Only commas
    let mut variables2 = HashMap::new();
    variables2.insert("all_branches".to_string(), ",,,".to_string());
    let context2 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: variables2,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result2 = executor.execute(&context2);
    assert!(result2.success, "Should handle malformed all_branches");
}

#[tokio::test]
async fn test_pattern_27_breaking_point_mismatched_branches() {
    // BREAKING POINT: arrived_from contains branches not in all_branches

    let (_, executor) = advanced_control::create_pattern_27();

    let mut variables = HashMap::new();
    variables.insert("all_branches".to_string(), "branch_0,branch_1".to_string());
    let mut arrived_from = HashSet::new();
    arrived_from.insert("branch_999".to_string()); // Not in all_branches

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from,
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle mismatched branches");
    // Should cancel all branches in all_branches since branch_999 is not in the list
    assert!(
        !result.cancel_activities.is_empty(),
        "BREAKING POINT: Should still cancel branches even with mismatch"
    );
}

// ============================================================================
// PATTERN 28: STRUCTURED LOOP - BREAKING POINTS
// ============================================================================

#[tokio::test]
async fn test_pattern_28_breaking_point_max_iterations_overflow() {
    // BREAKING POINT: Iteration count exceeds max_iterations

    let (_, executor) = advanced_control::create_pattern_28();

    // Test 1: Iteration > max_iterations
    let mut variables = HashMap::new();
    variables.insert("continue".to_string(), "true".to_string());
    variables.insert("iteration".to_string(), "2000".to_string());
    variables.insert("max_iterations".to_string(), "1000".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle iteration > max_iterations");
    assert!(
        result.next_state.as_ref().unwrap().contains("exited"),
        "BREAKING POINT: Should exit when iteration exceeds max"
    );
}

#[tokio::test]
async fn test_pattern_28_breaking_point_invalid_iteration_value() {
    // BREAKING POINT: Invalid iteration value

    let (_, executor) = advanced_control::create_pattern_28();

    // Test 1: Negative iteration
    let mut variables = HashMap::new();
    variables.insert("iteration".to_string(), "-1".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    // Should default to 0 or handle gracefully
    assert!(result.success, "Should handle negative iteration");

    // Test 2: Non-numeric iteration
    let mut variables2 = HashMap::new();
    variables2.insert("iteration".to_string(), "not_a_number".to_string());
    let context2 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: variables2,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result2 = executor.execute(&context2);
    assert!(result2.success, "Should handle non-numeric iteration");
    // Should default to 0, so next iteration should be 1
    assert_eq!(
        result2.variables.get("iteration"),
        Some(&"1".to_string()),
        "BREAKING POINT: Should default to 0 and increment to 1"
    );
}

#[tokio::test]
async fn test_pattern_28_breaking_point_max_iterations_zero() {
    // BREAKING POINT: max_iterations = 0

    let (_, executor) = advanced_control::create_pattern_28();

    let mut variables = HashMap::new();
    variables.insert("continue".to_string(), "true".to_string());
    variables.insert("iteration".to_string(), "0".to_string());
    variables.insert("max_iterations".to_string(), "0".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle max_iterations = 0");
    // Should exit immediately since next_iteration (1) >= max_iterations (0)
    assert!(
        result.next_state.as_ref().unwrap().contains("exited"),
        "BREAKING POINT: Should exit immediately when max_iterations = 0"
    );
}

// ============================================================================
// PATTERN 29: RECURSION - BREAKING POINTS
// ============================================================================

#[tokio::test]
async fn test_pattern_29_breaking_point_max_depth_overflow() {
    // BREAKING POINT: Depth exceeds max_depth

    let (_, executor) = advanced_control::create_pattern_29();

    let mut variables = HashMap::new();
    variables.insert("depth".to_string(), "200".to_string());
    variables.insert("max_depth".to_string(), "100".to_string());
    variables.insert("sub_case_done".to_string(), "false".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle depth > max_depth");
    // Should complete since next_depth (201) >= max_depth (100)
    assert!(
        result.next_state.as_ref().unwrap().contains("completed"),
        "BREAKING POINT: Should complete when depth exceeds max"
    );
}

#[tokio::test]
async fn test_pattern_29_breaking_point_max_depth_zero() {
    // BREAKING POINT: max_depth = 0

    let (_, executor) = advanced_control::create_pattern_29();

    let mut variables = HashMap::new();
    variables.insert("depth".to_string(), "0".to_string());
    variables.insert("max_depth".to_string(), "0".to_string());
    variables.insert("sub_case_done".to_string(), "false".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle max_depth = 0");
    // Should complete immediately since next_depth (1) >= max_depth (0)
    assert!(
        result.next_state.as_ref().unwrap().contains("completed"),
        "BREAKING POINT: Should complete immediately when max_depth = 0"
    );
}

#[tokio::test]
async fn test_pattern_29_breaking_point_invalid_depth_value() {
    // BREAKING POINT: Invalid depth value

    let (_, executor) = advanced_control::create_pattern_29();

    // Test 1: Negative depth
    let mut variables = HashMap::new();
    variables.insert("depth".to_string(), "-5".to_string());
    variables.insert("max_depth".to_string(), "10".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    // Should default to 0, so next_depth = 1
    assert!(result.success, "Should handle negative depth");
    assert_eq!(
        result.variables.get("depth"),
        Some(&"1".to_string()),
        "BREAKING POINT: Should default to 0 and increment to 1"
    );
}

// ============================================================================
// PATTERN 33: CANCEL PROCESS - BREAKING POINTS
// ============================================================================

#[tokio::test]
async fn test_pattern_33_breaking_point_always_terminates() {
    // BREAKING POINT: Pattern 33 MUST always terminate regardless of context

    let (_, executor) = advanced_control::create_pattern_33();

    // Test with various contexts - all should terminate
    let contexts = vec![
        // Empty context
        PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: String::new(),
        },
        // With scope_id
        PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: "test_scope".to_string(),
        },
        // With many variables
        {
            let mut vars = HashMap::new();
            for i in 0..1000 {
                vars.insert(format!("var_{}", i), format!("value_{}", i));
            }
            PatternExecutionContext {
                case_id: CaseId::new(),
                workflow_id: WorkflowSpecId::new(),
                variables: vars,
                arrived_from: HashSet::new(),
                scope_id: String::new(),
            }
        },
    ];

    for (i, context) in contexts.iter().enumerate() {
        let result = executor.execute(context);
        assert!(
            result.terminates,
            "BREAKING POINT: Pattern 33 MUST terminate in context {}",
            i
        );
        assert!(
            !result.cancel_activities.is_empty(),
            "BREAKING POINT: Pattern 33 MUST cancel in context {}",
            i
        );
    }
}

// ============================================================================
// PATTERN 38: MULTIPLE THREADS - BREAKING POINTS
// ============================================================================

#[tokio::test]
async fn test_pattern_38_breaking_point_max_threads() {
    // BREAKING POINT: Maximum thread count (should be capped at 1000)

    let (_, executor) = advanced_control::create_pattern_38();

    // Test 1: Thread count > 1000 (should be capped)
    let mut variables = HashMap::new();
    variables.insert("thread_count".to_string(), "10000".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle thread_count > 1000");
    assert_eq!(
        result.next_activities.len(),
        1000,
        "BREAKING POINT: Should cap thread count at 1000"
    );
}

#[tokio::test]
async fn test_pattern_38_breaking_point_zero_threads() {
    // BREAKING POINT: thread_count = 0 (should default to 1)

    let (_, executor) = advanced_control::create_pattern_38();

    let mut variables = HashMap::new();
    variables.insert("thread_count".to_string(), "0".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle thread_count = 0");
    assert_eq!(
        result.next_activities.len(),
        1,
        "BREAKING POINT: Should default to minimum 1 thread"
    );
}

#[tokio::test]
async fn test_pattern_38_breaking_point_invalid_thread_count() {
    // BREAKING POINT: Invalid thread_count value

    let (_, executor) = advanced_control::create_pattern_38();

    // Test 1: Negative number
    let mut variables = HashMap::new();
    variables.insert("thread_count".to_string(), "-5".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    // Should default to 2 or handle gracefully
    assert!(result.success, "Should handle negative thread_count");
    assert!(
        result.next_activities.len() >= 1,
        "BREAKING POINT: Should schedule at least 1 thread"
    );

    // Test 2: Non-numeric
    let mut variables2 = HashMap::new();
    variables2.insert("thread_count".to_string(), "not_a_number".to_string());
    let context2 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: variables2,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result2 = executor.execute(&context2);
    assert!(result2.success, "Should handle non-numeric thread_count");
    assert_eq!(
        result2.next_activities.len(),
        2,
        "BREAKING POINT: Should default to 2 threads"
    );
}

#[tokio::test]
async fn test_pattern_38_breaking_point_memory_pressure() {
    // BREAKING POINT: Memory pressure with many threads

    let (_, executor) = advanced_control::create_pattern_38();

    // Test with maximum allowed threads (1000)
    let mut variables = HashMap::new();
    variables.insert("thread_count".to_string(), "1000".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle 1000 threads");
    assert_eq!(
        result.next_activities.len(),
        1000,
        "BREAKING POINT: Should schedule exactly 1000 threads"
    );

    // Verify all thread IDs are unique
    let mut seen = HashSet::new();
    for thread in &result.next_activities {
        assert!(
            seen.insert(thread),
            "BREAKING POINT: Thread IDs must be unique: {}",
            thread
        );
    }
}

// ============================================================================
// PATTERN 39: THREAD MERGE - BREAKING POINTS
// ============================================================================

#[tokio::test]
async fn test_pattern_39_breaking_point_mismatched_threads() {
    // BREAKING POINT: arrived_from contains threads not matching expected pattern

    let (_, executor) = advanced_control::create_pattern_39();

    let mut variables = HashMap::new();
    variables.insert("expected_threads".to_string(), "3".to_string());
    let mut arrived_from = HashSet::new();
    arrived_from.insert("thread_1".to_string());
    arrived_from.insert("thread_2".to_string());
    arrived_from.insert("unexpected_thread".to_string()); // Not matching pattern

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from,
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle mismatched thread names");
    // Should still merge if arrived_from.len() >= expected_threads
    assert!(
        result.next_state.as_ref().unwrap().contains("merged"),
        "BREAKING POINT: Should merge when arrived count >= expected"
    );
}

#[tokio::test]
async fn test_pattern_39_breaking_point_zero_expected_threads() {
    // BREAKING POINT: expected_threads = 0

    let (_, executor) = advanced_control::create_pattern_39();

    let mut variables = HashMap::new();
    variables.insert("expected_threads".to_string(), "0".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle expected_threads = 0");
    // Should default to arrived_from.len().max(1) = 1, so should merge
    assert!(
        result.next_state.as_ref().unwrap().contains("merged"),
        "BREAKING POINT: Should merge when expected_threads = 0 (defaults to 1)"
    );
}

#[tokio::test]
async fn test_pattern_39_breaking_point_more_arrived_than_expected() {
    // BREAKING POINT: More threads arrived than expected

    let (_, executor) = advanced_control::create_pattern_39();

    let mut variables = HashMap::new();
    variables.insert("expected_threads".to_string(), "2".to_string());
    let mut arrived_from = HashSet::new();
    arrived_from.insert("thread_1".to_string());
    arrived_from.insert("thread_2".to_string());
    arrived_from.insert("thread_3".to_string()); // Extra thread

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from,
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle more arrived than expected");
    // Should merge since arrived_from.len() (3) >= expected_threads (2)
    assert!(
        result.next_state.as_ref().unwrap().contains("merged"),
        "BREAKING POINT: Should merge when arrived >= expected"
    );
}

// ============================================================================
// COMPREHENSIVE BREAKING POINT TESTS
// ============================================================================

#[tokio::test]
async fn test_all_patterns_breaking_point_empty_context() {
    // BREAKING POINT: All patterns with completely empty context

    let patterns = vec![
        ("26", advanced_control::create_pattern_26().1),
        ("27", advanced_control::create_pattern_27().1),
        ("28", advanced_control::create_pattern_28().1),
        ("29", advanced_control::create_pattern_29().1),
        ("33", advanced_control::create_pattern_33().1),
        ("38", advanced_control::create_pattern_38().1),
        ("39", advanced_control::create_pattern_39().1),
    ];

    let empty_context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    for (name, executor) in patterns {
        let result = executor.execute(&empty_context);
        assert!(
            result.success,
            "BREAKING POINT: Pattern {} should handle empty context",
            name
        );
    }
}

#[tokio::test]
async fn test_all_patterns_breaking_point_extreme_values() {
    // BREAKING POINT: All patterns with extreme values

    let (_, executor_26) = advanced_control::create_pattern_26();
    let (_, executor_28) = advanced_control::create_pattern_28();
    let (_, executor_29) = advanced_control::create_pattern_29();
    let (_, executor_38) = advanced_control::create_pattern_38();

    // Pattern 26: Extreme expected_branches
    let mut vars_26 = HashMap::new();
    vars_26.insert("expected_branches".to_string(), usize::MAX.to_string());
    let ctx_26 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: vars_26,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };
    let result_26 = executor_26.execute(&ctx_26);
    assert!(result_26.success, "Pattern 26 should handle usize::MAX");

    // Pattern 28: Extreme max_iterations
    let mut vars_28 = HashMap::new();
    vars_28.insert("max_iterations".to_string(), usize::MAX.to_string());
    vars_28.insert("iteration".to_string(), "0".to_string());
    let ctx_28 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: vars_28,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };
    let result_28 = executor_28.execute(&ctx_28);
    assert!(result_28.success, "Pattern 28 should handle usize::MAX");

    // Pattern 29: Extreme max_depth
    let mut vars_29 = HashMap::new();
    vars_29.insert("max_depth".to_string(), usize::MAX.to_string());
    vars_29.insert("depth".to_string(), "0".to_string());
    let ctx_29 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: vars_29,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };
    let result_29 = executor_29.execute(&ctx_29);
    assert!(result_29.success, "Pattern 29 should handle usize::MAX");

    // Pattern 38: Maximum threads (1000)
    let mut vars_38 = HashMap::new();
    vars_38.insert("thread_count".to_string(), "1000".to_string());
    let ctx_38 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: vars_38,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };
    let result_38 = executor_38.execute(&ctx_38);
    assert!(result_38.success, "Pattern 38 should handle 1000 threads");
    assert_eq!(
        result_38.next_activities.len(),
        1000,
        "BREAKING POINT: Should schedule exactly 1000 threads"
    );
}

#[tokio::test]
async fn test_all_patterns_breaking_point_state_consistency() {
    // BREAKING POINT: State consistency under rapid execution

    let patterns = vec![
        ("26", advanced_control::create_pattern_26().1),
        ("27", advanced_control::create_pattern_27().1),
        ("28", advanced_control::create_pattern_28().1),
        ("29", advanced_control::create_pattern_29().1),
        ("33", advanced_control::create_pattern_33().1),
        ("38", advanced_control::create_pattern_38().1),
        ("39", advanced_control::create_pattern_39().1),
    ];

    for (name, executor) in patterns {
        let context = PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: String::new(),
        };

        // Execute 1000 times rapidly
        for i in 0..1000 {
            let result = executor.execute(&context);
            assert!(
                result.success,
                "BREAKING POINT: Pattern {} should succeed on iteration {}",
                name, i
            );

            // Pattern 33 must always terminate
            if name == "33" {
                assert!(
                    result.terminates,
                    "BREAKING POINT: Pattern 33 MUST terminate on iteration {}",
                    i
                );
            }
        }
    }
}
