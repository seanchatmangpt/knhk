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
    advanced_control, trigger, PatternExecutionContext, PatternExecutionResult, PatternId,
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
// PATTERN 33: CANCEL PROCESS INSTANCE - BREAKING POINTS
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
// PATTERN 30 & 31: TRIGGERS - BREAKING POINTS
// ============================================================================

#[tokio::test]
async fn test_pattern_30_breaking_point_missing_trigger_fired() {
    // BREAKING POINT: Missing trigger_fired (should default to false/waiting)

    let (_, executor) = advanced_control::create_pattern_30();

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle missing trigger_fired");
    assert!(
        result.next_state.as_ref().unwrap().contains("waiting"),
        "BREAKING POINT: Should be waiting when trigger_fired not set"
    );
    assert_eq!(
        result.next_activities.len(),
        0,
        "BREAKING POINT: Should not schedule activities when waiting"
    );
}

#[tokio::test]
async fn test_pattern_30_breaking_point_trigger_fired() {
    // BREAKING POINT: Trigger fired should schedule activities

    let (_, executor) = advanced_control::create_pattern_30();

    let mut variables = HashMap::new();
    variables.insert("trigger_fired".to_string(), "true".to_string());
    variables.insert("trigger_source".to_string(), "timer".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle trigger_fired=true");
    assert!(
        result.next_state.as_ref().unwrap().contains("fired"),
        "BREAKING POINT: Should indicate fired when trigger_fired=true"
    );
    assert!(
        result.next_activities.contains(&"continue".to_string()),
        "BREAKING POINT: Should schedule continue when fired"
    );
}

#[tokio::test]
async fn test_pattern_30_breaking_point_extreme_trigger_source() {
    // BREAKING POINT: Extreme trigger_source length

    let (_, executor) = advanced_control::create_pattern_30();

    let mut variables = HashMap::new();
    variables.insert("trigger_source".to_string(), "x".repeat(10000));

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(
        result.success,
        "Should handle extreme trigger_source length"
    );
}

#[tokio::test]
async fn test_pattern_31_breaking_point_trigger_count_zero() {
    // BREAKING POINT: trigger_count = 0

    let (_, executor) = advanced_control::create_pattern_31();

    let mut variables = HashMap::new();
    variables.insert("trigger_count".to_string(), "0".to_string());
    variables.insert("fired_count".to_string(), "0".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle trigger_count = 0");
    assert!(
        result.next_state.as_ref().unwrap().contains("completed"),
        "BREAKING POINT: Should complete immediately when trigger_count = 0"
    );
}

#[tokio::test]
async fn test_pattern_31_breaking_point_fired_exceeds_count() {
    // BREAKING POINT: fired_count >= trigger_count

    let (_, executor) = advanced_control::create_pattern_31();

    let mut variables = HashMap::new();
    variables.insert("trigger_count".to_string(), "5".to_string());
    variables.insert("fired_count".to_string(), "10".to_string()); // Exceeds count

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle fired_count > trigger_count");
    assert!(
        result.next_state.as_ref().unwrap().contains("completed"),
        "BREAKING POINT: Should complete when fired_count >= trigger_count"
    );
    assert_eq!(
        result.next_activities.len(),
        0,
        "BREAKING POINT: Should not schedule activities when completed"
    );
}

#[tokio::test]
async fn test_pattern_31_breaking_point_invalid_trigger_count() {
    // BREAKING POINT: Invalid trigger_count value

    let (_, executor) = advanced_control::create_pattern_31();

    // Test 1: Negative trigger_count
    let mut variables = HashMap::new();
    variables.insert("trigger_count".to_string(), "-5".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    // Should default to 1
    assert!(result.success, "Should handle negative trigger_count");

    // Test 2: Non-numeric trigger_count
    let mut variables2 = HashMap::new();
    variables2.insert("trigger_count".to_string(), "not_a_number".to_string());

    let context2 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: variables2,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result2 = executor.execute(&context2);
    assert!(result2.success, "Should handle non-numeric trigger_count");
    // Should default to 1, so should fire
    assert!(
        result2.next_state.as_ref().unwrap().contains("fired"),
        "BREAKING POINT: Should fire when trigger_count defaults to 1"
    );
}

// ============================================================================
// PATTERN 32: CANCEL ACTIVITY INSTANCE - BREAKING POINTS
// ============================================================================

#[tokio::test]
async fn test_pattern_32_breaking_point_empty_activity_ids() {
    // BREAKING POINT: Empty activity_ids string

    let (_, executor) = advanced_control::create_pattern_32();

    let mut variables = HashMap::new();
    variables.insert("activity_ids".to_string(), "".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle empty activity_ids");
    // Should default to scope-based or "activity_instance"
    assert!(
        !result.cancel_activities.is_empty(),
        "BREAKING POINT: Should cancel default activity when activity_ids empty"
    );
}

#[tokio::test]
async fn test_pattern_32_breaking_point_malformed_activity_ids() {
    // BREAKING POINT: Malformed activity_ids

    let (_, executor) = advanced_control::create_pattern_32();

    // Test 1: Only commas
    let mut variables = HashMap::new();
    variables.insert("activity_ids".to_string(), ",,,".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle malformed activity_ids");
    assert_eq!(
        result.cancel_activities.len(),
        0,
        "BREAKING POINT: Should have no activities to cancel when all empty"
    );

    // Test 2: Mixed valid and invalid
    let mut variables2 = HashMap::new();
    variables2.insert(
        "activity_ids".to_string(),
        "activity_1,,activity_2,".to_string(),
    );

    let context2 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: variables2,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result2 = executor.execute(&context2);
    assert!(result2.success, "Should handle mixed activity_ids");
    assert_eq!(
        result2.cancel_activities.len(),
        2,
        "BREAKING POINT: Should cancel only valid activity IDs"
    );
}

#[tokio::test]
async fn test_pattern_32_breaking_point_extreme_activity_ids() {
    // BREAKING POINT: Extreme number of activity_ids

    let (_, executor) = advanced_control::create_pattern_32();

    let mut activity_ids = Vec::new();
    for i in 0..10000 {
        activity_ids.push(format!("activity_{}", i));
    }

    let mut variables = HashMap::new();
    variables.insert("activity_ids".to_string(), activity_ids.join(","));

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(
        result.success,
        "Should handle extreme number of activity_ids"
    );
    assert_eq!(
        result.cancel_activities.len(),
        10000,
        "BREAKING POINT: Should cancel all 10000 activities"
    );
}

#[tokio::test]
async fn test_pattern_32_breaking_point_missing_all_parameters() {
    // BREAKING POINT: Missing both activity_ids and instance_id

    let (_, executor) = advanced_control::create_pattern_32();

    // Test 1: Empty context
    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle missing parameters");
    assert!(
        !result.cancel_activities.is_empty(),
        "BREAKING POINT: Should default to activity_instance when all missing"
    );

    // Test 2: With scope_id
    let context2 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: "test_scope".to_string(),
    };

    let result2 = executor.execute(&context2);
    assert!(
        result2.cancel_activities[0].contains("test_scope"),
        "BREAKING POINT: Should use scope_id when provided"
    );
}

#[tokio::test]
async fn test_pattern_32_breaking_point_never_terminates() {
    // BREAKING POINT: Pattern 32 must NEVER terminate (unlike Pattern 33)

    let (_, executor) = advanced_control::create_pattern_32();

    let contexts = vec![
        PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: String::new(),
        },
        {
            let mut vars = HashMap::new();
            vars.insert(
                "activity_ids".to_string(),
                "activity_1,activity_2".to_string(),
            );
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
            !result.terminates,
            "BREAKING POINT: Pattern 32 MUST NOT terminate in context {}",
            i
        );
    }
}

// ============================================================================
// PATTERN 34: STOP PROCESS INSTANCE - BREAKING POINTS
// ============================================================================

#[tokio::test]
async fn test_pattern_34_breaking_point_always_terminates() {
    // BREAKING POINT: Pattern 34 MUST always terminate

    let (_, executor) = advanced_control::create_pattern_34();

    let contexts = vec![
        PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: String::new(),
        },
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
        PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: "test_scope".to_string(),
        },
    ];

    for (i, context) in contexts.iter().enumerate() {
        let result = executor.execute(context);
        assert!(
            result.terminates,
            "BREAKING POINT: Pattern 34 MUST terminate in context {}",
            i
        );
        assert!(
            result.cancel_activities.is_empty(),
            "BREAKING POINT: Pattern 34 should NOT cancel (unlike Pattern 33)"
        );
    }
}

#[tokio::test]
async fn test_pattern_34_breaking_point_vs_pattern_33() {
    // BREAKING POINT: Pattern 34 vs 33 difference

    let (_, executor_33) = advanced_control::create_pattern_33();
    let (_, executor_34) = advanced_control::create_pattern_34();

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result_33 = executor_33.execute(&context);
    let result_34 = executor_34.execute(&context);

    // Both must terminate
    assert!(result_33.terminates, "Pattern 33 must terminate");
    assert!(result_34.terminates, "Pattern 34 must terminate");

    // Pattern 33 cancels, Pattern 34 does not
    assert!(
        !result_33.cancel_activities.is_empty(),
        "BREAKING POINT: Pattern 33 should cancel"
    );
    assert!(
        result_34.cancel_activities.is_empty(),
        "BREAKING POINT: Pattern 34 should NOT cancel"
    );
}

// ============================================================================
// PATTERN 35: ABORT PROCESS INSTANCE - BREAKING POINTS
// ============================================================================

#[tokio::test]
async fn test_pattern_35_breaking_point_always_terminates() {
    // BREAKING POINT: Pattern 35 MUST always terminate

    let (_, executor) = advanced_control::create_pattern_35();

    let contexts = vec![
        PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: String::new(),
        },
        {
            let mut vars = HashMap::new();
            vars.insert("abort_reason".to_string(), "Test abort".to_string());
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
            "BREAKING POINT: Pattern 35 MUST terminate in context {}",
            i
        );
        assert!(
            result.cancel_activities.is_empty(),
            "BREAKING POINT: Pattern 35 should NOT cancel"
        );
    }
}

#[tokio::test]
async fn test_pattern_35_breaking_point_missing_abort_reason() {
    // BREAKING POINT: Missing abort_reason (should default)

    let (_, executor) = advanced_control::create_pattern_35();

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle missing abort_reason");
    assert!(
        result.variables.contains_key("abort_reason"),
        "BREAKING POINT: Should set default abort_reason"
    );
    assert!(
        result
            .variables
            .get("abort_reason")
            .unwrap()
            .contains("Aborted by pattern 35"),
        "BREAKING POINT: Should use default abort reason"
    );
}

#[tokio::test]
async fn test_pattern_35_breaking_point_extreme_abort_reason() {
    // BREAKING POINT: Extreme abort_reason length

    let (_, executor) = advanced_control::create_pattern_35();

    let mut variables = HashMap::new();
    variables.insert("abort_reason".to_string(), "x".repeat(100000));

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle extreme abort_reason length");
    assert_eq!(
        result.variables.get("abort_reason").unwrap().len(),
        100000,
        "BREAKING POINT: Should preserve extreme abort_reason length"
    );
}

// ============================================================================
// PATTERN 36: DISABLE ACTIVITY - BREAKING POINTS
// ============================================================================

#[tokio::test]
async fn test_pattern_36_breaking_point_empty_activity_ids() {
    // BREAKING POINT: Empty activity_ids

    let (_, executor) = advanced_control::create_pattern_36();

    let mut variables = HashMap::new();
    variables.insert("activity_ids".to_string(), "".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle empty activity_ids");
    // Should default to "activity"
    assert!(
        result.cancel_activities.is_empty(),
        "BREAKING POINT: Disable should NOT cancel activities"
    );
    assert!(
        !result.terminates,
        "BREAKING POINT: Disable should NOT terminate"
    );
}

#[tokio::test]
async fn test_pattern_36_breaking_point_extreme_activity_ids() {
    // BREAKING POINT: Extreme number of activity_ids

    let (_, executor) = advanced_control::create_pattern_36();

    let mut activity_ids = Vec::new();
    for i in 0..10000 {
        activity_ids.push(format!("activity_{}", i));
    }

    let mut variables = HashMap::new();
    variables.insert("activity_ids".to_string(), activity_ids.join(","));

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle extreme activity_ids");
    assert_eq!(
        result.variables.get("disabled_count").unwrap(),
        &"10000",
        "BREAKING POINT: Should track all disabled activities"
    );
}

#[tokio::test]
async fn test_pattern_36_breaking_point_never_terminates() {
    // BREAKING POINT: Pattern 36 must NEVER terminate

    let (_, executor) = advanced_control::create_pattern_36();

    let contexts = vec![
        PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: String::new(),
        },
        {
            let mut vars = HashMap::new();
            vars.insert(
                "activity_ids".to_string(),
                "activity_1,activity_2".to_string(),
            );
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
            !result.terminates,
            "BREAKING POINT: Pattern 36 MUST NOT terminate in context {}",
            i
        );
        assert!(
            result.cancel_activities.is_empty(),
            "BREAKING POINT: Pattern 36 MUST NOT cancel in context {}",
            i
        );
    }
}

// ============================================================================
// PATTERN 37: SKIP ACTIVITY - BREAKING POINTS
// ============================================================================

#[tokio::test]
async fn test_pattern_37_breaking_point_empty_activity_ids() {
    // BREAKING POINT: Empty activity_ids

    let (_, executor) = advanced_control::create_pattern_37();

    let mut variables = HashMap::new();
    variables.insert("activity_ids".to_string(), "".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle empty activity_ids");
    // Should default to "activity"
    assert_eq!(
        result.next_activities.len(),
        0,
        "BREAKING POINT: Skip should NOT schedule activities"
    );
    assert!(
        result.cancel_activities.is_empty(),
        "BREAKING POINT: Skip should NOT cancel"
    );
    assert!(
        !result.terminates,
        "BREAKING POINT: Skip should NOT terminate"
    );
}

#[tokio::test]
async fn test_pattern_37_breaking_point_extreme_activity_ids() {
    // BREAKING POINT: Extreme number of activity_ids

    let (_, executor) = advanced_control::create_pattern_37();

    let mut activity_ids = Vec::new();
    for i in 0..10000 {
        activity_ids.push(format!("activity_{}", i));
    }

    let mut variables = HashMap::new();
    variables.insert("activity_ids".to_string(), activity_ids.join(","));

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle extreme activity_ids");
    assert_eq!(
        result.variables.get("skipped_count").unwrap(),
        &"10000",
        "BREAKING POINT: Should track all skipped activities"
    );
    assert_eq!(
        result.next_activities.len(),
        0,
        "BREAKING POINT: Skip should never schedule activities"
    );
}

#[tokio::test]
async fn test_pattern_37_breaking_point_never_schedules() {
    // BREAKING POINT: Pattern 37 must NEVER schedule activities

    let (_, executor) = advanced_control::create_pattern_37();

    let contexts = vec![
        PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: String::new(),
        },
        {
            let mut vars = HashMap::new();
            vars.insert(
                "activity_ids".to_string(),
                "activity_1,activity_2".to_string(),
            );
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
        assert_eq!(
            result.next_activities.len(),
            0,
            "BREAKING POINT: Pattern 37 MUST NOT schedule activities in context {}",
            i
        );
        assert!(
            !result.terminates,
            "BREAKING POINT: Pattern 37 MUST NOT terminate in context {}",
            i
        );
    }
}

// ============================================================================
// PATTERNS 40-43: TRIGGER PATTERNS - BREAKING POINTS
// ============================================================================

#[tokio::test]
async fn test_pattern_40_breaking_point_missing_trigger_source() {
    // BREAKING POINT: Missing trigger_source (should default)

    let (_, executor) = trigger::create_pattern_40();

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle missing trigger_source");
    assert_eq!(
        result.variables.get("trigger_source").unwrap(),
        &"external",
        "BREAKING POINT: Should default trigger_source to 'external'"
    );
}

#[tokio::test]
async fn test_pattern_40_breaking_point_extreme_trigger_source() {
    // BREAKING POINT: Extreme trigger_source length

    let (_, executor) = trigger::create_pattern_40();

    let mut variables = HashMap::new();
    variables.insert("trigger_source".to_string(), "x".repeat(100000));

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(
        result.success,
        "Should handle extreme trigger_source length"
    );
    assert!(
        result.next_state.as_ref().unwrap().contains("received"),
        "BREAKING POINT: Should indicate received"
    );
}

#[tokio::test]
async fn test_pattern_41_breaking_point_missing_event_type() {
    // BREAKING POINT: Missing event_type (should default)

    let (_, executor) = trigger::create_pattern_41();

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle missing event_type");
    assert_eq!(
        result.variables.get("event_type").unwrap(),
        &"unknown",
        "BREAKING POINT: Should default event_type to 'unknown'"
    );
}

#[tokio::test]
async fn test_pattern_41_breaking_point_extreme_event_type() {
    // BREAKING POINT: Extreme event_type length

    let (_, executor) = trigger::create_pattern_41();

    let mut variables = HashMap::new();
    variables.insert("event_type".to_string(), "x".repeat(100000));

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle extreme event_type length");
}

#[tokio::test]
async fn test_pattern_42_breaking_point_trigger_count_zero() {
    // BREAKING POINT: trigger_count = 0

    let (_, executor) = trigger::create_pattern_42();

    let mut variables = HashMap::new();
    variables.insert("trigger_count".to_string(), "0".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle trigger_count = 0");
    // Should default to 1
    assert_eq!(
        result.variables.get("trigger_count").unwrap(),
        &"0",
        "BREAKING POINT: Should preserve trigger_count = 0"
    );
}

#[tokio::test]
async fn test_pattern_42_breaking_point_invalid_trigger_count() {
    // BREAKING POINT: Invalid trigger_count

    let (_, executor) = trigger::create_pattern_42();

    // Test 1: Negative
    let mut variables = HashMap::new();
    variables.insert("trigger_count".to_string(), "-5".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    // Should default to 1
    assert!(result.success, "Should handle negative trigger_count");
    assert_eq!(
        result.variables.get("trigger_count").unwrap(),
        &"1",
        "BREAKING POINT: Should default to 1 when invalid"
    );

    // Test 2: Non-numeric
    let mut variables2 = HashMap::new();
    variables2.insert("trigger_count".to_string(), "not_a_number".to_string());

    let context2 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: variables2,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result2 = executor.execute(&context2);
    assert!(result2.success, "Should handle non-numeric trigger_count");
    assert_eq!(
        result2.variables.get("trigger_count").unwrap(),
        &"1",
        "BREAKING POINT: Should default to 1 when non-numeric"
    );
}

#[tokio::test]
async fn test_pattern_43_breaking_point_missing_trigger_id() {
    // BREAKING POINT: Missing trigger_id (should default)

    let (_, executor) = trigger::create_pattern_43();

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle missing trigger_id");
    assert_eq!(
        result.variables.get("trigger_id").unwrap(),
        &"unknown",
        "BREAKING POINT: Should default trigger_id to 'unknown'"
    );
}

#[tokio::test]
async fn test_pattern_43_breaking_point_extreme_trigger_id() {
    // BREAKING POINT: Extreme trigger_id length

    let (_, executor) = trigger::create_pattern_43();

    let mut variables = HashMap::new();
    variables.insert("trigger_id".to_string(), "x".repeat(100000));

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should handle extreme trigger_id length");
    assert!(
        result.next_state.as_ref().unwrap().contains("cancelled"),
        "BREAKING POINT: Should indicate cancelled"
    );
}

#[tokio::test]
async fn test_patterns_40_43_breaking_point_never_terminate() {
    // BREAKING POINT: Trigger patterns must NEVER terminate

    let patterns = vec![
        ("40", trigger::create_pattern_40().1),
        ("41", trigger::create_pattern_41().1),
        ("42", trigger::create_pattern_42().1),
        ("43", trigger::create_pattern_43().1),
    ];

    for (name, executor) in patterns {
        let context = PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: String::new(),
        };

        let result = executor.execute(&context);
        assert!(
            !result.terminates,
            "BREAKING POINT: Pattern {} MUST NOT terminate",
            name
        );
    }
}

// ============================================================================
// COMPREHENSIVE: ALL REMAINING PATTERNS BREAKING POINTS
// ============================================================================

#[tokio::test]
async fn test_all_remaining_patterns_breaking_point_empty_context() {
    // BREAKING POINT: All remaining patterns with empty context

    let patterns = vec![
        ("30", advanced_control::create_pattern_30().1),
        ("31", advanced_control::create_pattern_31().1),
        ("32", advanced_control::create_pattern_32().1),
        ("34", advanced_control::create_pattern_34().1),
        ("35", advanced_control::create_pattern_35().1),
        ("36", advanced_control::create_pattern_36().1),
        ("37", advanced_control::create_pattern_37().1),
        ("40", trigger::create_pattern_40().1),
        ("41", trigger::create_pattern_41().1),
        ("42", trigger::create_pattern_42().1),
        ("43", trigger::create_pattern_43().1),
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

        // Patterns 34-35 must terminate
        if name == "34" || name == "35" {
            assert!(
                result.terminates,
                "BREAKING POINT: Pattern {} MUST terminate",
                name
            );
        } else {
            assert!(
                !result.terminates,
                "BREAKING POINT: Pattern {} MUST NOT terminate",
                name
            );
        }
    }
}

#[tokio::test]
async fn test_all_remaining_patterns_breaking_point_state_consistency() {
    // BREAKING POINT: State consistency under rapid execution

    let patterns = vec![
        ("30", advanced_control::create_pattern_30().1),
        ("31", advanced_control::create_pattern_31().1),
        ("32", advanced_control::create_pattern_32().1),
        ("34", advanced_control::create_pattern_34().1),
        ("35", advanced_control::create_pattern_35().1),
        ("36", advanced_control::create_pattern_36().1),
        ("37", advanced_control::create_pattern_37().1),
        ("40", trigger::create_pattern_40().1),
        ("41", trigger::create_pattern_41().1),
        ("42", trigger::create_pattern_42().1),
        ("43", trigger::create_pattern_43().1),
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

            // Patterns 34-35 must always terminate
            if name == "34" || name == "35" {
                assert!(
                    result.terminates,
                    "BREAKING POINT: Pattern {} MUST terminate on iteration {}",
                    name, i
                );
            }
        }
    }
}
