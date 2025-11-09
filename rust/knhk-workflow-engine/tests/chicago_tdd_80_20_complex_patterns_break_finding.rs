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

chicago_async_test!(test_pattern_26_blocking_discriminator_edge_cases, {
    // Break-finding: Test edge cases for blocking discriminator

    let (pattern_id, executor) = advanced_control::create_pattern_26();

    // Test 1: Empty context - should default to waiting (no branches arrived)
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
    assert!(
        result.next_state.as_ref().unwrap().contains("waiting"),
        "BREAK: Should be waiting when no branches arrived"
    );
    assert_eq!(
        result.next_activities.len(),
        0,
        "BREAK: Should not schedule activities when waiting"
    );

    // Test 2: All branches arrived - should continue
    let mut variables = HashMap::new();
    variables.insert("expected_branches".to_string(), "3".to_string());
    let mut arrived_from = HashSet::new();
    arrived_from.insert("branch_1".to_string());
    arrived_from.insert("branch_2".to_string());
    arrived_from.insert("branch_3".to_string());

    let all_arrived_context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from,
        scope_id: String::new(),
    };

    let result2 = executor.execute(&all_arrived_context);
    assert!(
        result2.next_state.as_ref().unwrap().contains("all-arrived"),
        "BREAK: Should indicate all arrived when all branches present"
    );
    assert!(
        result2.next_activities.contains(&"continue".to_string()),
        "BREAK: Should schedule continue when all arrived"
    );

    // Test 3: Context with many variables
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

    let result3 = executor.execute(&large_context);
    assert!(result3.success, "Should handle large context");

    // Test 4: Verify state consistency
    let result1 = executor.execute(&empty_context);
    let result2 = executor.execute(&empty_context);
    assert_eq!(
        result1.next_state, result2.next_state,
        "Same context should produce same state"
    );
}

chicago_async_test!(test_pattern_27_cancelling_discriminator_break_finding, {
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

chicago_async_test!(test_pattern_26_27_discriminator_state_corruption, {
    // Break-finding: Test for state corruption between discriminator patterns

    let (pattern_id_26, executor_26) = advanced_control::create_pattern_26();
    let (pattern_id_27, executor_27) = advanced_control::create_pattern_27();

    // Test Pattern 26 with proper context
    let mut variables_26 = HashMap::new();
    variables_26.insert("expected_branches".to_string(), "2".to_string());
    let mut arrived_from_26 = HashSet::new();
    arrived_from_26.insert("branch_1".to_string());

    let context_26 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: variables_26,
        arrived_from: arrived_from_26,
        scope_id: String::new(),
    };

    // Execute pattern 26 multiple times
    for _ in 0..100 {
        let result = executor_26.execute(&context_26);
        assert!(result.success, "Pattern 26 should always succeed");
        assert_eq!(
            result.cancel_activities.len(),
            0,
            "BREAK: Blocking discriminator should not cancel activities"
        );
    }

    // Test Pattern 27 with proper context
    let mut variables_27 = HashMap::new();
    variables_27.insert("all_branches".to_string(), "branch_0,branch_1".to_string());
    let mut arrived_from_27 = HashSet::new();
    arrived_from_27.insert("branch_0".to_string());

    let context_27 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: variables_27,
        arrived_from: arrived_from_27,
        scope_id: String::new(),
    };

    // Execute pattern 27 multiple times
    for _ in 0..100 {
        let result = executor_27.execute(&context_27);
        assert!(result.success, "Pattern 27 should always succeed");
        assert!(
            !result.cancel_activities.is_empty(),
            "BREAK: Cancelling discriminator should cancel activities"
        );
    }

    // Verify pattern 26 still works after pattern 27
    let result = executor_26.execute(&context_26);
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

chicago_async_test!(test_pattern_28_structured_loop_break_finding, {
    // Break-finding: Test structured loop for breaks

    let (pattern_id, executor) = advanced_control::create_pattern_28();

    // Test 1: Loop should continue when continue=true
    let mut variables = HashMap::new();
    variables.insert("continue".to_string(), "true".to_string());
    variables.insert("iteration".to_string(), "0".to_string());

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
        "BREAK: Structured loop should set next state"
    );
    assert!(
        result.next_state.as_ref().unwrap().contains("iterating"),
        "BREAK: State should indicate iteration when continue=true"
    );
    assert!(
        result.next_activities.contains(&"loop_body".to_string()),
        "BREAK: Should schedule loop_body when continuing"
    );
    assert_eq!(
        result.variables.get("iteration"),
        Some(&"1".to_string()),
        "BREAK: Should increment iteration"
    );

    // Test 2: Loop should exit when continue=false
    let mut variables2 = HashMap::new();
    variables2.insert("continue".to_string(), "false".to_string());
    variables2.insert("iteration".to_string(), "5".to_string());

    let context2 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: variables2,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result2 = executor.execute(&context2);
    assert!(
        result2.next_state.as_ref().unwrap().contains("exited"),
        "BREAK: State should indicate exit when continue=false"
    );
    assert!(
        result2.next_activities.contains(&"loop_exit".to_string()),
        "BREAK: Should schedule loop_exit when exiting"
    );

    // Test 3: Loop should exit when max_iterations reached
    let mut variables3 = HashMap::new();
    variables3.insert("continue".to_string(), "true".to_string());
    variables3.insert("iteration".to_string(), "999".to_string());
    variables3.insert("max_iterations".to_string(), "1000".to_string());

    let context3 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: variables3,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result3 = executor.execute(&context3);
    assert!(
        result3.next_state.as_ref().unwrap().contains("exited"),
        "BREAK: Should exit when max_iterations reached"
    );

    // Test 4: Verify no cancellation (loops don't cancel)
    assert_eq!(
        result.cancel_activities.len(),
        0,
        "BREAK: Structured loop should not cancel activities"
    );

    // Test 5: Verify no termination (loops continue)
    assert!(
        !result.terminates,
        "BREAK: Structured loop should not terminate"
    );
}

chicago_async_test!(test_pattern_29_recursion_break_finding, {
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

chicago_async_test!(test_pattern_28_29_loop_recursion_infinite_break, {
    // Break-finding: Test for infinite loop/recursion breaks

    let (pattern_id_28, executor_28) = advanced_control::create_pattern_28();
    let (pattern_id_29, executor_29) = advanced_control::create_pattern_29();

    // Test Pattern 28: Loop with max_iterations protection
    let mut variables_28 = HashMap::new();
    variables_28.insert("continue".to_string(), "true".to_string());
    variables_28.insert("iteration".to_string(), "0".to_string());
    variables_28.insert("max_iterations".to_string(), "1000".to_string());

    let context_28 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: variables_28,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    // Execute loop many times - should not hang and should respect max_iterations
    for i in 0..1000 {
        let mut ctx = context_28.clone();
        ctx.variables.insert("iteration".to_string(), i.to_string());
        let result = executor_28.execute(&ctx);
        assert!(result.success, "Loop should succeed on iteration {}", i);
        assert!(
            !result.terminates,
            "BREAK: Loop should not terminate unexpectedly on iteration {}",
            i
        );

        // Should exit when max_iterations reached
        if i >= 999 {
            assert!(
                result.next_state.as_ref().unwrap().contains("exited"),
                "BREAK: Should exit at max_iterations"
            );
        }
    }

    // Test Pattern 29: Recursion with max_depth protection
    let mut variables_29 = HashMap::new();
    variables_29.insert("depth".to_string(), "0".to_string());
    variables_29.insert("max_depth".to_string(), "100".to_string());
    variables_29.insert("sub_case_done".to_string(), "false".to_string());

    let context_29 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: variables_29,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    // Execute recursion many times - should not hang and should respect max_depth
    for i in 0..100 {
        let mut ctx = context_29.clone();
        ctx.variables.insert("depth".to_string(), i.to_string());
        let result = executor_29.execute(&ctx);
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

        // Should complete when max_depth reached
        if i >= 99 {
            assert!(
                result.next_state.as_ref().unwrap().contains("completed"),
                "BREAK: Should complete at max_depth"
            );
        }
    }
}

// ============================================================================
// PATTERN 33: CANCEL PROCESS INSTANCE (Critical 20% - Critical Cancellation)
// ============================================================================

chicago_async_test!(test_pattern_33_cancel_process_break_finding, {
    // Break-finding: Test cancel process for breaks

    let (pattern_id, executor) = advanced_control::create_pattern_33();
    let case_id = CaseId::new();

    // Test 1: Cancel process with case_id
    let context = PatternExecutionContext {
        case_id,
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should execute successfully");
    assert!(
        !result.cancel_activities.is_empty(),
        "BREAK: Cancel process should cancel activities"
    );
    assert!(
        result.cancel_activities[0].contains("process:"),
        "BREAK: Should cancel process (format: process:case_id)"
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

    // Test 4: Cancel process with scope_id
    let context2 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: "scope_123".to_string(),
    };
    let result2 = executor.execute(&context2);
    assert!(
        result2.cancel_activities[0].contains("process:scope_123"),
        "BREAK: Should use scope_id when provided"
    );
use chicago_tdd_tools::{chicago_async_test, assert_ok, assert_err, assert_eq_msg};
}

chicago_async_test!(test_pattern_33_cancel_process_state_consistency, {
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

chicago_async_test!(test_pattern_33_vs_32_cancel_difference, {
    // Break-finding: Verify pattern 33 (cancel process) vs 32 (cancel activity) difference

    let (pattern_id_32, executor_32) = advanced_control::create_pattern_32();
    let (pattern_id_33, executor_33) = advanced_control::create_pattern_33();

    // Test Pattern 32: Cancel activity with activity_ids
    let mut variables_32 = HashMap::new();
    variables_32.insert(
        "activity_ids".to_string(),
        "activity_1,activity_2".to_string(),
    );

    let context_32 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: variables_32,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result_32 = executor_32.execute(&context_32);

    // Test Pattern 33: Cancel process
    let context_33 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result_33 = executor_33.execute(&context_33);

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

    // Pattern 32 should cancel specific activities
    assert!(
        result_32
            .cancel_activities
            .contains(&"activity_1".to_string()),
        "BREAK: Pattern 32 should cancel specified activities"
    );

    // Pattern 33 should cancel process
    assert!(
        result_33.cancel_activities[0].contains("process:"),
        "BREAK: Pattern 33 should cancel process"
    );
}

// ============================================================================
// PATTERN 38 & 39: THREADING (Critical 20% - Parallelism)
// ============================================================================

chicago_async_test!(test_pattern_38_multiple_threads_break_finding, {
    // Break-finding: Test multiple threads pattern for breaks

    let (pattern_id, executor) = advanced_control::create_pattern_38();

    // Test 1: Default thread count (should be 2)
    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result = executor.execute(&context);
    assert!(result.success, "Should execute successfully");
    assert!(
        !result.next_activities.is_empty(),
        "BREAK: Multiple threads should schedule activities"
    );
    assert_eq!(
        result.next_activities.len(),
        2,
        "BREAK: Should schedule exactly 2 threads by default"
    );

    // Test 2: Custom thread count from variables
    let mut variables = HashMap::new();
    variables.insert("thread_count".to_string(), "5".to_string());

    let context2 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result2 = executor.execute(&context2);
    assert_eq!(
        result2.next_activities.len(),
        5,
        "BREAK: Should schedule 5 threads when thread_count=5"
    );
    assert!(
        result2.next_activities.contains(&"thread_1".to_string()),
        "BREAK: Should schedule thread_1"
    );
    assert!(
        result2.next_activities.contains(&"thread_5".to_string()),
        "BREAK: Should schedule thread_5"
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

chicago_async_test!(test_pattern_39_thread_merge_break_finding, {
    // Break-finding: Test thread merge pattern for breaks

    let (pattern_id, executor) = advanced_control::create_pattern_39();

    // Test 1: All threads arrived - should merge
    let mut variables = HashMap::new();
    variables.insert("expected_threads".to_string(), "2".to_string());
    let mut arrived_from = HashSet::new();
    arrived_from.insert("thread_1".to_string());
    arrived_from.insert("thread_2".to_string());

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
        result.next_state.is_some(),
        "BREAK: Thread merge should set next state"
    );
    assert!(
        result.next_state.as_ref().unwrap().contains("merged"),
        "BREAK: State should indicate merge when all threads arrived"
    );
    assert!(
        result.next_activities.contains(&"continue".to_string()),
        "BREAK: Should schedule continue when all threads merged"
    );

    // Test 2: Not all threads arrived - should wait
    let mut variables2 = HashMap::new();
    variables2.insert("expected_threads".to_string(), "3".to_string());
    let mut arrived_from2 = HashSet::new();
    arrived_from2.insert("thread_1".to_string());

    let context2 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: variables2,
        arrived_from: arrived_from2,
        scope_id: String::new(),
    };

    let result2 = executor.execute(&context2);
    assert!(
        result2.next_state.as_ref().unwrap().contains("waiting"),
        "BREAK: State should indicate waiting when threads not all arrived"
    );
    assert_eq!(
        result2.next_activities.len(),
        0,
        "BREAK: Should not schedule activities when waiting"
    );

    // Test 3: Verify no cancellation
    assert_eq!(
        result.cancel_activities.len(),
        0,
        "BREAK: Thread merge should not cancel activities"
    );
}

chicago_async_test!(test_pattern_38_39_threading_sequence_break, {
    // Break-finding: Test threading sequence for breaks

    let (pattern_id_38, executor_38) = advanced_control::create_pattern_38();
    let (pattern_id_39, executor_39) = advanced_control::create_pattern_39();

    // Execute pattern 38 (spawn threads)
    let mut variables = HashMap::new();
    variables.insert("thread_count".to_string(), "3".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result_38 = executor_38.execute(&context);
    assert!(result_38.success, "Pattern 38 should succeed");
    assert!(
        !result_38.next_activities.is_empty(),
        "BREAK: Pattern 38 should schedule threads"
    );
    assert_eq!(
        result_38.next_activities.len(),
        3,
        "BREAK: Pattern 38 should schedule 3 threads"
    );

    // Execute pattern 39 (merge threads) - simulate all threads arrived
    let mut variables2 = HashMap::new();
    variables2.insert("expected_threads".to_string(), "3".to_string());
    let mut arrived_from = HashSet::new();
    arrived_from.insert("thread_1".to_string());
    arrived_from.insert("thread_2".to_string());
    arrived_from.insert("thread_3".to_string());

    let context2 = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: variables2,
        arrived_from,
        scope_id: String::new(),
    };

    let result_39 = executor_39.execute(&context2);
    assert!(result_39.success, "Pattern 39 should succeed");
    assert!(
        result_39.next_activities.contains(&"continue".to_string()),
        "BREAK: Pattern 39 should schedule continue when all threads merged"
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

chicago_async_test!(test_pattern_38_thread_count_consistency, {
    // Break-finding: Test thread count consistency

    let (pattern_id, executor) = advanced_control::create_pattern_38();

    // Test with fixed thread_count
    let mut variables = HashMap::new();
    variables.insert("thread_count".to_string(), "3".to_string());

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
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
    assert_eq!(first_count, 3, "BREAK: Should schedule 3 threads");
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

chicago_async_test!(test_critical_patterns_state_isolation, {
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

chicago_async_test!(test_critical_patterns_boundary_conditions, {
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

chicago_async_test!(test_critical_patterns_termination_consistency, {
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
