//! 80/20 Break-Finding Tests for Most Complex Patterns
//!
//! Focuses on the critical 20% of complex patterns (26-39) that provide 80% of value:
//! - Pattern 26 & 27: Discriminators (race conditions, critical for performance)
//! - Pattern 28 & 29: Loops/Recursion (iteration, critical for workflows)
//! - Pattern 33: Cancel Process Instance (critical cancellation)
//! - Pattern 38 & 39: Threading (parallelism, critical for scalability)
//!
//! Tests designed to find breaks:
//! - Edge cases
//! - Boundary conditions
//! - State corruption
//! - Race conditions
//! - Resource leaks
//! - Invalid state transitions

use knhk_workflow_engine::case::CaseId;
use knhk_workflow_engine::parser::WorkflowSpecId;
use knhk_workflow_engine::patterns::{
    advanced_control, PatternExecutionContext, PatternExecutionResult, PatternId,
};
use std::collections::{HashMap, HashSet};

// ============================================================================
// PATTERN 26 & 27: DISCRIMINATORS (Critical 20% - Race Conditions)
// ============================================================================

#[tokio::test]
async fn test_pattern_26_blocking_discriminator_edge_cases() {
    // Break-finding: Test edge cases for blocking discriminator

    // Test 1: Empty context
    let (pattern_id, executor) = advanced_control::create_pattern_26();
    let empty_context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&empty_context);
    assert!(result.success, "Should handle empty context");
    assert_eq!(pattern_id.0, 26);

    // Test 2: Context with many variables
    let mut large_context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };
    for i in 0..1000 {
        large_context
            .variables
            .insert(format!("var_{}", i), format!("value_{}", i));
    }

    let result = executor.execute(&large_context);
    assert!(result.success, "Should handle large context");

    // Test 3: Verify state consistency
    let result1 = executor.execute(&empty_context);
    let result2 = executor.execute(&empty_context);
    assert_eq!(
        result1.next_state, result2.next_state,
        "Same context should produce same state"
    );
}

#[tokio::test]
async fn test_pattern_27_cancelling_discriminator_break_finding() {
    // Break-finding: Test cancelling discriminator for breaks

    let (pattern_id, executor) = advanced_control::create_pattern_27();

    // Test 1: With first branch arrived - should cancel others
    let mut variables = HashMap::new();
    variables.insert(
        "all_branches".to_string(),
        "branch_0,branch_1,branch_2".to_string(),
    );
    let mut arrived_from = HashSet::new();
    arrived_from.insert("branch_0".to_string()); // First branch arrived

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from,
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should execute successfully");
    assert!(
        !result.cancel_activities.is_empty(),
        "BREAK: Cancelling discriminator should cancel activities"
    );
    // Should cancel branch_1 and branch_2 (not branch_0 which arrived first)
    assert!(
        result.cancel_activities.contains(&"branch_1".to_string()),
        "BREAK: Should cancel branch_1"
    );
    assert!(
        result.cancel_activities.contains(&"branch_2".to_string()),
        "BREAK: Should cancel branch_2"
    );
    assert!(
        !result.cancel_activities.contains(&"branch_0".to_string()),
        "BREAK: Should NOT cancel branch_0 (first arrived)"
    );

    // Test 2: Multiple executions should be consistent
    let result1 = executor.execute(&context);
    let result2 = executor.execute(&context);
    assert_eq!(
        result1.cancel_activities, result2.cancel_activities,
        "BREAK: Cancellation should be deterministic"
    );

    // Test 3: Verify terminates is false (should not terminate workflow)
    assert!(
        !result.terminates,
        "BREAK: Cancelling discriminator should not terminate workflow"
    );

    // Test 4: With no branches arrived - should cancel all except first
    let mut variables2 = HashMap::new();
    variables2.insert("all_branches".to_string(), "branch_0,branch_1".to_string());
    let context2 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: variables2,
        arrived_from: HashSet::new(), // No branches arrived yet
        scope_id: String::new(),
    };
    let result2 = executor.execute(&context2);
    assert!(
        !result2.cancel_activities.is_empty(),
        "BREAK: Should cancel branches even when none arrived"
    );
}

#[tokio::test]
async fn test_pattern_26_27_discriminator_state_corruption() {
    // Break-finding: Test for state corruption between discriminator patterns

    let (pattern_id_26, executor_26) = advanced_control::create_pattern_26();
    let (pattern_id_27, executor_27) = advanced_control::create_pattern_27();

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    // Execute pattern 26 multiple times
    for _ in 0..100 {
        let result = executor_26.execute(&context);
        assert!(result.success, "Pattern 26 should always succeed");
        assert_eq!(
            result.cancel_activities.len(),
            0,
            "BREAK: Blocking discriminator should not cancel activities"
        );
    }

    // Execute pattern 27 multiple times
    for _ in 0..100 {
        let result = executor_27.execute(&context);
        assert!(result.success, "Pattern 27 should always succeed");
        assert!(
            !result.cancel_activities.is_empty(),
            "BREAK: Cancelling discriminator should cancel activities"
        );
    }

    // Verify pattern 26 still works after pattern 27
    let result = executor_26.execute(&context);
    assert!(
        result.success,
        "Pattern 26 should still work after pattern 27"
    );
    assert_eq!(
        result.cancel_activities.len(),
        0,
        "BREAK: Pattern 26 should not be affected by pattern 27"
    );
}

// ============================================================================
// PATTERN 28 & 29: LOOPS/RECURSION (Critical 20% - Iteration)
// ============================================================================

#[tokio::test]
async fn test_pattern_28_structured_loop_break_finding() {
    // Break-finding: Test structured loop for breaks

    let (pattern_id, executor) = advanced_control::create_pattern_28();
    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    // Test 1: Verify loop state
    let result = executor.execute(&context);
    assert!(result.success, "Should execute successfully");
    assert!(
        result.next_state.is_some(),
        "BREAK: Structured loop should set next state"
    );
    assert!(
        result.next_state.as_ref().unwrap().contains("iterating"),
        "BREAK: State should indicate iteration"
    );

    // Test 2: Verify no cancellation (loops don't cancel)
    assert_eq!(
        result.cancel_activities.len(),
        0,
        "BREAK: Structured loop should not cancel activities"
    );

    // Test 3: Verify no termination (loops continue)
    assert!(
        !result.terminates,
        "BREAK: Structured loop should not terminate"
    );
}

#[tokio::test]
async fn test_pattern_29_recursion_break_finding() {
    // Break-finding: Test recursion for breaks

    let (pattern_id, executor) = advanced_control::create_pattern_29();

    // Test 1: Recursion should continue when sub-case not done
    let mut variables = HashMap::new();
    variables.insert("depth".to_string(), "0".to_string());
    variables.insert("max_depth".to_string(), "10".to_string());
    variables.insert("sub_case_done".to_string(), "false".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should execute successfully");
    assert!(
        result.next_state.is_some(),
        "BREAK: Recursion should set next state"
    );
    assert!(
        result.next_state.as_ref().unwrap().contains("recursing"),
        "BREAK: State should indicate recursion when sub-case not done"
    );
    assert!(
        result.next_activities.contains(&"recurse".to_string()),
        "BREAK: Should schedule recurse activity"
    );

    // Test 2: Recursion should stop when sub-case done
    let mut variables2 = HashMap::new();
    variables2.insert("depth".to_string(), "5".to_string());
    variables2.insert("max_depth".to_string(), "10".to_string());
    variables2.insert("sub_case_done".to_string(), "true".to_string());

    let context2 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: variables2,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result2 = executor.execute(&context2);
    assert!(
        result2.next_state.as_ref().unwrap().contains("completed"),
        "BREAK: State should indicate completion when sub-case done"
    );
    assert!(
        result2.next_activities.contains(&"return".to_string()),
        "BREAK: Should schedule return activity when done"
    );

    // Test 3: Verify recursion doesn't terminate
    assert!(
        !result.terminates,
        "BREAK: Recursion should not terminate (allows continuation)"
    );

    // Test 4: Verify depth tracking
    assert_eq!(
        result.variables.get("depth"),
        Some(&"1".to_string()),
        "BREAK: Should increment depth"
    );
}

#[tokio::test]
async fn test_pattern_28_29_loop_recursion_infinite_break() {
    // Break-finding: Test for infinite loop/recursion breaks

    let (pattern_id_28, executor_28) = advanced_control::create_pattern_28();
    let (pattern_id_29, executor_29) = advanced_control::create_pattern_29();

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    // Execute loop many times - should not hang
    for i in 0..1000 {
        let result = executor_28.execute(&context);
        assert!(result.success, "Loop should succeed on iteration {}", i);
        assert!(
            !result.terminates,
            "BREAK: Loop should not terminate unexpectedly on iteration {}",
            i
        );
    }

    // Execute recursion many times - should not hang
    for i in 0..1000 {
        let result = executor_29.execute(&context);
        assert!(
            result.success,
            "Recursion should succeed on iteration {}",
            i
        );
        assert!(
            !result.terminates,
            "BREAK: Recursion should not terminate unexpectedly on iteration {}",
            i
        );
    }
}

// ============================================================================
// PATTERN 33: CANCEL PROCESS INSTANCE (Critical 20% - Critical Cancellation)
// ============================================================================

#[tokio::test]
async fn test_pattern_33_cancel_process_break_finding() {
    // Break-finding: Test cancel process for breaks

    let (pattern_id, executor) = advanced_control::create_pattern_33();
    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    // Test 1: Verify cancellation
    let result = executor.execute(&context);
    assert!(result.success, "Should execute successfully");
    assert!(
        !result.cancel_activities.is_empty(),
        "BREAK: Cancel process should cancel activities"
    );
    assert_eq!(
        result.cancel_activities[0], "process_instance",
        "BREAK: Should cancel process instance"
    );

    // Test 2: CRITICAL - Verify termination
    assert!(
        result.terminates,
        "BREAK: Cancel process MUST terminate workflow"
    );

    // Test 3: Verify state indicates cancellation
    assert!(
        result.next_state.is_some(),
        "BREAK: Cancel process should set next state"
    );
    assert!(
        result.next_state.as_ref().unwrap().contains("cancelled"),
        "BREAK: State should indicate cancellation"
    );
}

#[tokio::test]
async fn test_pattern_33_cancel_process_state_consistency() {
    // Break-finding: Test state consistency for cancel process

    let (pattern_id, executor) = advanced_control::create_pattern_33();

    // Test with different contexts
    for i in 0..100 {
        let mut context = PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: format!("scope_{}", i),
        };
        context.variables.insert("test".to_string(), i.to_string());

        let result = executor.execute(&context);

        // CRITICAL: Must always terminate
        assert!(
            result.terminates,
            "BREAK: Cancel process MUST always terminate on iteration {}",
            i
        );

        // CRITICAL: Must always cancel
        assert!(
            !result.cancel_activities.is_empty(),
            "BREAK: Cancel process MUST always cancel on iteration {}",
            i
        );
    }
}

#[tokio::test]
async fn test_pattern_33_vs_32_cancel_difference() {
    // Break-finding: Verify pattern 33 (cancel process) vs 32 (cancel activity) difference

    let (pattern_id_32, executor_32) = advanced_control::create_pattern_32();
    let (pattern_id_33, executor_33) = advanced_control::create_pattern_33();

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result_32 = executor_32.execute(&context);
    let result_33 = executor_33.execute(&context);

    // CRITICAL BREAK: Pattern 32 should NOT terminate
    assert!(
        !result_32.terminates,
        "BREAK: Cancel activity (32) should NOT terminate workflow"
    );

    // CRITICAL BREAK: Pattern 33 MUST terminate
    assert!(
        result_33.terminates,
        "BREAK: Cancel process (33) MUST terminate workflow"
    );

    // Both should cancel activities
    assert!(
        !result_32.cancel_activities.is_empty(),
        "BREAK: Pattern 32 should cancel activities"
    );
    assert!(
        !result_33.cancel_activities.is_empty(),
        "BREAK: Pattern 33 should cancel activities"
    );
}

// ============================================================================
// PATTERN 38 & 39: THREADING (Critical 20% - Parallelism)
// ============================================================================

#[tokio::test]
async fn test_pattern_38_multiple_threads_break_finding() {
    // Break-finding: Test multiple threads pattern for breaks

    let (pattern_id, executor) = advanced_control::create_pattern_38();
    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    // Test 1: Verify threads are scheduled
    let result = executor.execute(&context);
    assert!(result.success, "Should execute successfully");
    assert!(
        !result.next_activities.is_empty(),
        "BREAK: Multiple threads should schedule activities"
    );
    assert!(
        result.next_activities.len() >= 2,
        "BREAK: Should schedule at least 2 threads"
    );

    // Test 2: Verify thread names
    assert!(
        result.next_activities.contains(&"thread_1".to_string()),
        "BREAK: Should schedule thread_1"
    );
    assert!(
        result.next_activities.contains(&"thread_2".to_string()),
        "BREAK: Should schedule thread_2"
    );

    // Test 3: Verify no cancellation
    assert_eq!(
        result.cancel_activities.len(),
        0,
        "BREAK: Multiple threads should not cancel activities"
    );

    // Test 4: Verify no termination
    assert!(
        !result.terminates,
        "BREAK: Multiple threads should not terminate"
    );
}

#[tokio::test]
async fn test_pattern_39_thread_merge_break_finding() {
    // Break-finding: Test thread merge pattern for breaks

    let (pattern_id, executor) = advanced_control::create_pattern_39();
    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    // Test 1: Verify merge state
    let result = executor.execute(&context);
    assert!(result.success, "Should execute successfully");
    assert!(
        result.next_state.is_some(),
        "BREAK: Thread merge should set next state"
    );
    assert!(
        result.next_state.as_ref().unwrap().contains("merged"),
        "BREAK: State should indicate merge"
    );

    // Test 2: Verify no activities scheduled (merge completes)
    assert_eq!(
        result.next_activities.len(),
        0,
        "BREAK: Thread merge should not schedule new activities"
    );

    // Test 3: Verify no cancellation
    assert_eq!(
        result.cancel_activities.len(),
        0,
        "BREAK: Thread merge should not cancel activities"
    );
}

#[tokio::test]
async fn test_pattern_38_39_threading_sequence_break() {
    // Break-finding: Test threading sequence for breaks

    let (pattern_id_38, executor_38) = advanced_control::create_pattern_38();
    let (pattern_id_39, executor_39) = advanced_control::create_pattern_39();

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    // Execute pattern 38 (spawn threads)
    let result_38 = executor_38.execute(&context);
    assert!(result_38.success, "Pattern 38 should succeed");
    assert!(
        !result_38.next_activities.is_empty(),
        "BREAK: Pattern 38 should schedule threads"
    );

    // Execute pattern 39 (merge threads)
    let result_39 = executor_39.execute(&context);
    assert!(result_39.success, "Pattern 39 should succeed");
    assert_eq!(
        result_39.next_activities.len(),
        0,
        "BREAK: Pattern 39 should not schedule activities after merge"
    );

    // Verify state transition
    assert!(
        result_38.next_state.as_ref().unwrap().contains("executing"),
        "BREAK: Pattern 38 state should indicate execution"
    );
    assert!(
        result_39.next_state.as_ref().unwrap().contains("merged"),
        "BREAK: Pattern 39 state should indicate merge"
    );
}

#[tokio::test]
async fn test_pattern_38_thread_count_consistency() {
    // Break-finding: Test thread count consistency

    let (pattern_id, executor) = advanced_control::create_pattern_38();
    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    // Execute multiple times - thread count should be consistent
    let mut thread_counts = Vec::new();
    for _ in 0..100 {
        let result = executor.execute(&context);
        assert!(result.success, "Should succeed");
        thread_counts.push(result.next_activities.len());
    }

    // All executions should schedule same number of threads
    let first_count = thread_counts[0];
    for (i, count) in thread_counts.iter().enumerate() {
        assert_eq!(
            *count, first_count,
            "BREAK: Thread count should be consistent, iteration {}",
            i
        );
    }
}

// ============================================================================
// COMPREHENSIVE BREAK-FINDING: ALL CRITICAL PATTERNS
// ============================================================================

#[tokio::test]
async fn test_critical_patterns_state_isolation() {
    // Break-finding: Test state isolation between critical patterns

    let patterns = vec![
        advanced_control::create_pattern_26(),
        advanced_control::create_pattern_27(),
        advanced_control::create_pattern_28(),
        advanced_control::create_pattern_29(),
        advanced_control::create_pattern_33(),
        advanced_control::create_pattern_38(),
        advanced_control::create_pattern_39(),
    ];

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    // Execute all patterns - each should maintain its own state
    for (pattern_id, executor) in &patterns {
        let result = executor.execute(&context);
        assert!(result.success, "Pattern {} should succeed", pattern_id.0);

        // Verify each pattern has distinct behavior
        match pattern_id.0 {
            26 => {
                assert_eq!(
                    result.cancel_activities.len(),
                    0,
                    "BREAK: Pattern 26 should not cancel"
                );
            }
            27 => {
                assert!(
                    !result.cancel_activities.is_empty(),
                    "BREAK: Pattern 27 should cancel"
                );
            }
            28 | 29 => {
                assert!(
                    !result.terminates,
                    "BREAK: Patterns 28-29 should not terminate"
                );
            }
            33 => {
                assert!(result.terminates, "BREAK: Pattern 33 MUST terminate");
            }
            38 => {
                assert!(
                    !result.next_activities.is_empty(),
                    "BREAK: Pattern 38 should schedule threads"
                );
            }
            39 => {
                assert_eq!(
                    result.next_activities.len(),
                    0,
                    "BREAK: Pattern 39 should not schedule activities"
                );
            }
            _ => {}
        }
    }
}

#[tokio::test]
async fn test_critical_patterns_boundary_conditions() {
    // Break-finding: Test boundary conditions for all critical patterns

    let critical_patterns = vec![26, 27, 28, 29, 33, 38, 39];

    for pattern_id_num in critical_patterns {
        let pattern_id = PatternId::new(pattern_id_num)
            .expect(&format!("Invalid pattern ID: {}", pattern_id_num));

        // Test with empty context
        let empty_context = PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: String::new(),
        };

        // Get executor based on pattern ID
        let executor = match pattern_id_num {
            26 => advanced_control::create_pattern_26().1,
            27 => advanced_control::create_pattern_27().1,
            28 => advanced_control::create_pattern_28().1,
            29 => advanced_control::create_pattern_29().1,
            33 => advanced_control::create_pattern_33().1,
            38 => advanced_control::create_pattern_38().1,
            39 => advanced_control::create_pattern_39().1,
            _ => panic!("Unexpected pattern ID"),
        };

        let result = executor.execute(&empty_context);
        assert!(
            result.success,
            "BREAK: Pattern {} should handle empty context",
            pattern_id_num
        );

        // Test with very large context
        let mut large_context = PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: String::new(),
        };
        for i in 0..10000 {
            large_context
                .variables
                .insert(format!("var_{}", i), format!("value_{}", i));
        }

        let result = executor.execute(&large_context);
        assert!(
            result.success,
            "BREAK: Pattern {} should handle large context",
            pattern_id_num
        );
    }
}

#[tokio::test]
async fn test_critical_patterns_termination_consistency() {
    // Break-finding: Test termination consistency - CRITICAL BREAK CHECK

    // Patterns that MUST terminate
    let terminating_patterns = vec![33, 34, 35];

    // Patterns that MUST NOT terminate
    let non_terminating_patterns = vec![26, 27, 28, 29, 30, 31, 32, 36, 37, 38, 39];

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    // Test terminating patterns
    for pattern_id_num in terminating_patterns {
        let executor = match pattern_id_num {
            33 => advanced_control::create_pattern_33().1,
            34 => advanced_control::create_pattern_34().1,
            35 => advanced_control::create_pattern_35().1,
            _ => panic!("Unexpected pattern ID"),
        };

        let result = executor.execute(&context);
        assert!(
            result.terminates,
            "CRITICAL BREAK: Pattern {} MUST terminate",
            pattern_id_num
        );
    }

    // Test non-terminating patterns
    for pattern_id_num in non_terminating_patterns {
        let executor = match pattern_id_num {
            26 => advanced_control::create_pattern_26().1,
            27 => advanced_control::create_pattern_27().1,
            28 => advanced_control::create_pattern_28().1,
            29 => advanced_control::create_pattern_29().1,
            30 => advanced_control::create_pattern_30().1,
            31 => advanced_control::create_pattern_31().1,
            32 => advanced_control::create_pattern_32().1,
            36 => advanced_control::create_pattern_36().1,
            37 => advanced_control::create_pattern_37().1,
            38 => advanced_control::create_pattern_38().1,
            39 => advanced_control::create_pattern_39().1,
            _ => panic!("Unexpected pattern ID"),
        };

        let result = executor.execute(&context);
        assert!(
            !result.terminates,
            "CRITICAL BREAK: Pattern {} MUST NOT terminate",
            pattern_id_num
        );
    }
}
