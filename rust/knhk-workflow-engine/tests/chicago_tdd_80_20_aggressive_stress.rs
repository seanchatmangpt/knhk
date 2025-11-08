//! 80/20 Aggressive Stress Tests - Push Critical Patterns to Breaking Point
//!
//! Tests designed to find breaks under extreme conditions:
//! - Memory pressure
//! - State corruption
//! - Race conditions
//! - Resource exhaustion
//! - Invalid transitions

use knhk_workflow_engine::case::CaseId;
use knhk_workflow_engine::parser::WorkflowSpecId;
use knhk_workflow_engine::patterns::{
    advanced_control, PatternExecutionContext, PatternExecutionResult,
};
use std::collections::{HashMap, HashSet};

// ============================================================================
// AGGRESSIVE STRESS: PATTERN 26 & 27 (DISCRIMINATORS)
// ============================================================================

#[tokio::test]
async fn test_pattern_26_27_extreme_stress() {
    // Push discriminators to breaking point

    let (_, executor_26) = advanced_control::create_pattern_26();
    let (_, executor_27) = advanced_control::create_pattern_27();

    // Execute 10,000 times rapidly
    for i in 0..10_000 {
        let context = PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: format!("stress_{}", i),
        };

        let result_26 = executor_26.execute(&context);
        let result_27 = executor_27.execute(&context);

        // CRITICAL: Pattern 26 should never cancel
        assert_eq!(
            result_26.cancel_activities.len(),
            0,
            "BREAK: Pattern 26 cancelled activities on iteration {}",
            i
        );

        // CRITICAL: Pattern 27 should always cancel
        assert!(
            !result_27.cancel_activities.is_empty(),
            "BREAK: Pattern 27 failed to cancel on iteration {}",
            i
        );

        // CRITICAL: Neither should terminate
        assert!(
            !result_26.terminates,
            "BREAK: Pattern 26 terminated on iteration {}",
            i
        );
        assert!(
            !result_27.terminates,
            "BREAK: Pattern 27 terminated on iteration {}",
            i
        );
    }
}

// ============================================================================
// AGGRESSIVE STRESS: PATTERN 28 & 29 (LOOPS/RECURSION)
// ============================================================================

#[tokio::test]
async fn test_pattern_28_29_infinite_loop_protection() {
    // Test for infinite loop breaks

    let (_, executor_28) = advanced_control::create_pattern_28();
    let (_, executor_29) = advanced_control::create_pattern_29();

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    // Execute in tight loop - should not hang
    let start = std::time::Instant::now();
    for _ in 0..100_000 {
        let result_28 = executor_28.execute(&context);
        let result_29 = executor_29.execute(&context);

        assert!(result_28.success, "Pattern 28 should succeed");
        assert!(result_29.success, "Pattern 29 should succeed");

        // CRITICAL: Should never terminate (loops continue)
        assert!(!result_28.terminates, "BREAK: Pattern 28 terminated");
        assert!(!result_29.terminates, "BREAK: Pattern 29 terminated");
    }
    let duration = start.elapsed();

    // Should complete in reasonable time (< 1 second for 100k iterations)
    assert!(
        duration.as_secs() < 1,
        "BREAK: Patterns took too long: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_pattern_28_29_memory_pressure() {
    // Test with memory pressure (large contexts)

    let (_, executor_28) = advanced_control::create_pattern_28();
    let (_, executor_29) = advanced_control::create_pattern_29();

    // Create contexts with increasing variable counts
    for var_count in [100, 1000, 10000, 100000] {
        let mut context = PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: String::new(),
        };

        for i in 0..var_count {
            context
                .variables
                .insert(format!("var_{}", i), format!("value_{}", i));
        }

        let result_28 = executor_28.execute(&context);
        let result_29 = executor_29.execute(&context);

        assert!(
            result_28.success,
            "BREAK: Pattern 28 failed with {} variables",
            var_count
        );
        assert!(
            result_29.success,
            "BREAK: Pattern 29 failed with {} variables",
            var_count
        );
    }
}

// ============================================================================
// AGGRESSIVE STRESS: PATTERN 33 (CANCEL PROCESS)
// ============================================================================

#[tokio::test]
async fn test_pattern_33_termination_guarantee() {
    // CRITICAL: Pattern 33 MUST always terminate - test extensively

    let (_, executor) = advanced_control::create_pattern_33();

    // Test with 10,000 different contexts
    for i in 0..10_000 {
        let mut context = PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: format!("test_{}", i),
        };

        // Add varying variables
        for j in 0..(i % 100) {
            context
                .variables
                .insert(format!("var_{}", j), format!("value_{}", j));
        }

        let result = executor.execute(&context);

        // CRITICAL BREAK CHECK: Must always terminate
        assert!(
            result.terminates,
            "CRITICAL BREAK: Pattern 33 did not terminate on iteration {}",
            i
        );

        // CRITICAL BREAK CHECK: Must always cancel
        assert!(
            !result.cancel_activities.is_empty(),
            "CRITICAL BREAK: Pattern 33 did not cancel on iteration {}",
            i
        );

        // CRITICAL BREAK CHECK: Must always succeed
        assert!(
            result.success,
            "CRITICAL BREAK: Pattern 33 failed on iteration {}",
            i
        );
    }
}

#[tokio::test]
async fn test_pattern_33_vs_others_termination_isolation() {
    // CRITICAL: Verify pattern 33 termination doesn't affect other patterns

    let (_, executor_26) = advanced_control::create_pattern_26();
    let (_, executor_27) = advanced_control::create_pattern_27();
    let (_, executor_33) = advanced_control::create_pattern_33();
    let (_, executor_38) = advanced_control::create_pattern_38();

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    // Execute pattern 33 (terminates)
    let result_33 = executor_33.execute(&context);
    assert!(result_33.terminates, "Pattern 33 must terminate");

    // Execute other patterns - should NOT be affected
    let result_26 = executor_26.execute(&context);
    let result_27 = executor_27.execute(&context);
    let result_38 = executor_38.execute(&context);

    assert!(
        !result_26.terminates,
        "BREAK: Pattern 26 affected by pattern 33"
    );
    assert!(
        !result_27.terminates,
        "BREAK: Pattern 27 affected by pattern 33"
    );
    assert!(
        !result_38.terminates,
        "BREAK: Pattern 38 affected by pattern 33"
    );
}

// ============================================================================
// AGGRESSIVE STRESS: PATTERN 38 & 39 (THREADING)
// ============================================================================

#[tokio::test]
async fn test_pattern_38_thread_scheduling_consistency() {
    // CRITICAL: Thread scheduling must be consistent

    let (_, executor) = advanced_control::create_pattern_38();

    let mut thread_sets = Vec::new();

    // Execute 1000 times
    for _ in 0..1000 {
        let context = PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: String::new(),
        };

        let result = executor.execute(&context);
        assert!(result.success, "Should succeed");

        // CRITICAL: Must schedule threads
        assert!(
            !result.next_activities.is_empty(),
            "BREAK: Pattern 38 did not schedule threads"
        );

        // Collect thread sets
        let mut threads: Vec<String> = result.next_activities.clone();
        threads.sort();
        thread_sets.push(threads);
    }

    // CRITICAL: All thread sets should be identical
    let first_set = &thread_sets[0];
    for (i, thread_set) in thread_sets.iter().enumerate() {
        assert_eq!(
            thread_set, first_set,
            "BREAK: Thread scheduling inconsistent on iteration {}",
            i
        );
    }

    // CRITICAL: Should schedule exactly 2 threads
    assert_eq!(
        first_set.len(),
        2,
        "BREAK: Should schedule exactly 2 threads"
    );
    assert!(
        first_set.contains(&"thread_1".to_string()),
        "BREAK: Should schedule thread_1"
    );
    assert!(
        first_set.contains(&"thread_2".to_string()),
        "BREAK: Should schedule thread_2"
    );
}

#[tokio::test]
async fn test_pattern_38_39_threading_sequence_stress() {
    // Stress test threading sequence

    let (_, executor_38) = advanced_control::create_pattern_38();
    let (_, executor_39) = advanced_control::create_pattern_39();

    // Execute spawn-merge sequence 10,000 times
    for i in 0..10_000 {
        let context = PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: HashSet::new(),
            scope_id: format!("thread_{}", i),
        };

        // Spawn threads
        let result_38 = executor_38.execute(&context);
        assert!(
            result_38.success,
            "BREAK: Pattern 38 failed on iteration {}",
            i
        );
        assert!(
            !result_38.next_activities.is_empty(),
            "BREAK: Pattern 38 did not spawn threads on iteration {}",
            i
        );

        // Merge threads
        let result_39 = executor_39.execute(&context);
        assert!(
            result_39.success,
            "BREAK: Pattern 39 failed on iteration {}",
            i
        );
        assert_eq!(
            result_39.next_activities.len(),
            0,
            "BREAK: Pattern 39 scheduled activities after merge on iteration {}",
            i
        );

        // CRITICAL: Neither should terminate
        assert!(
            !result_38.terminates,
            "BREAK: Pattern 38 terminated on iteration {}",
            i
        );
        assert!(
            !result_39.terminates,
            "BREAK: Pattern 39 terminated on iteration {}",
            i
        );
    }
}

// ============================================================================
// COMPREHENSIVE BREAK-FINDING: ALL CRITICAL PATTERNS
// ============================================================================

#[tokio::test]
async fn test_all_critical_patterns_concurrent_execution() {
    // Simulate concurrent execution of all critical patterns

    let executors = vec![
        ("26", advanced_control::create_pattern_26().1),
        ("27", advanced_control::create_pattern_27().1),
        ("28", advanced_control::create_pattern_28().1),
        ("29", advanced_control::create_pattern_29().1),
        ("33", advanced_control::create_pattern_33().1),
        ("38", advanced_control::create_pattern_38().1),
        ("39", advanced_control::create_pattern_39().1),
    ];

    // Execute each pattern 1000 times in sequence (simulating concurrent)
    for (name, executor) in &executors {
        for i in 0..1000 {
            let context = PatternExecutionContext {
                case_id: CaseId::new(),
                workflow_id: WorkflowSpecId::new(),
                variables: HashMap::new(),
                arrived_from: HashSet::new(),
                scope_id: format!("{}_{}", name, i),
            };

            let result = executor.execute(&context);

            assert!(
                result.success,
                "BREAK: Pattern {} failed on iteration {}",
                name, i
            );

            // Pattern-specific break checks
            match *name {
                "26" => {
                    assert_eq!(
                        result.cancel_activities.len(),
                        0,
                        "BREAK: Pattern 26 cancelled on iteration {}",
                        i
                    );
                    assert!(
                        !result.terminates,
                        "BREAK: Pattern 26 terminated on iteration {}",
                        i
                    );
                }
                "27" => {
                    assert!(
                        !result.cancel_activities.is_empty(),
                        "BREAK: Pattern 27 did not cancel on iteration {}",
                        i
                    );
                    assert!(
                        !result.terminates,
                        "BREAK: Pattern 27 terminated on iteration {}",
                        i
                    );
                }
                "28" | "29" => {
                    assert!(
                        !result.terminates,
                        "BREAK: Pattern {} terminated on iteration {}",
                        name, i
                    );
                }
                "33" => {
                    assert!(
                        result.terminates,
                        "CRITICAL BREAK: Pattern 33 did not terminate on iteration {}",
                        i
                    );
                    assert!(
                        !result.cancel_activities.is_empty(),
                        "BREAK: Pattern 33 did not cancel on iteration {}",
                        i
                    );
                }
                "38" => {
                    assert!(
                        !result.next_activities.is_empty(),
                        "BREAK: Pattern 38 did not schedule threads on iteration {}",
                        i
                    );
                    assert!(
                        !result.terminates,
                        "BREAK: Pattern 38 terminated on iteration {}",
                        i
                    );
                }
                "39" => {
                    assert_eq!(
                        result.next_activities.len(),
                        0,
                        "BREAK: Pattern 39 scheduled activities on iteration {}",
                        i
                    );
                    assert!(
                        !result.terminates,
                        "BREAK: Pattern 39 terminated on iteration {}",
                        i
                    );
                }
                _ => {}
            }
        }
    }
}

#[tokio::test]
async fn test_critical_patterns_state_corruption_detection() {
    // Test for state corruption across pattern executions

    let patterns = vec![
        ("26", advanced_control::create_pattern_26().1),
        ("27", advanced_control::create_pattern_27().1),
        ("28", advanced_control::create_pattern_28().1),
        ("29", advanced_control::create_pattern_29().1),
        ("33", advanced_control::create_pattern_33().1),
        ("38", advanced_control::create_pattern_38().1),
        ("39", advanced_control::create_pattern_39().1),
    ];

    // Execute patterns in various orders
    let execution_orders = vec![
        vec![0, 1, 2, 3, 4, 5, 6],
        vec![6, 5, 4, 3, 2, 1, 0],
        vec![4, 0, 5, 1, 6, 2, 3],
        vec![3, 3, 3, 0, 0, 0, 4],
    ];

    for order in execution_orders {
        for pattern_idx in order {
            let (name, executor) = &patterns[pattern_idx];
            let context = PatternExecutionContext {
                case_id: CaseId::new(),
                workflow_id: WorkflowSpecId::new(),
                variables: HashMap::new(),
                arrived_from: HashSet::new(),
                scope_id: format!("order_{}", pattern_idx),
            };

            let result = executor.execute(&context);

            // CRITICAL: All patterns should succeed regardless of execution order
            assert!(
                result.success,
                "BREAK: Pattern {} failed in execution order",
                name
            );

            // CRITICAL: Pattern 33 must always terminate
            if *name == "33" {
                assert!(
                    result.terminates,
                    "CRITICAL BREAK: Pattern 33 did not terminate"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_critical_patterns_edge_case_combinations() {
    // Test edge case combinations

    let (_, executor_26) = advanced_control::create_pattern_26();
    let (_, executor_27) = advanced_control::create_pattern_27();
    let (_, executor_33) = advanced_control::create_pattern_33();
    let (_, executor_38) = advanced_control::create_pattern_38();
    let (_, executor_39) = advanced_control::create_pattern_39();

    // Edge case 1: Empty context
    let empty_context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: String::new(),
    };

    let result_26 = executor_26.execute(&empty_context);
    let result_27 = executor_27.execute(&empty_context);
    let result_33 = executor_33.execute(&empty_context);
    let result_38 = executor_38.execute(&empty_context);
    let result_39 = executor_39.execute(&empty_context);

    assert!(
        result_26.success,
        "BREAK: Pattern 26 failed with empty context"
    );
    assert!(
        result_27.success,
        "BREAK: Pattern 27 failed with empty context"
    );
    assert!(
        result_33.success,
        "BREAK: Pattern 33 failed with empty context"
    );
    assert!(
        result_38.success,
        "BREAK: Pattern 38 failed with empty context"
    );
    assert!(
        result_39.success,
        "BREAK: Pattern 39 failed with empty context"
    );

    // Edge case 2: Very large scope_id
    let large_scope_context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: HashSet::new(),
        scope_id: "x".repeat(10000),
    };

    let result_26 = executor_26.execute(&large_scope_context);
    let result_27 = executor_27.execute(&large_scope_context);
    let result_33 = executor_33.execute(&large_scope_context);

    assert!(
        result_26.success,
        "BREAK: Pattern 26 failed with large scope_id"
    );
    assert!(
        result_27.success,
        "BREAK: Pattern 27 failed with large scope_id"
    );
    assert!(
        result_33.success,
        "BREAK: Pattern 33 failed with large scope_id"
    );

    // CRITICAL: Pattern 33 must still terminate
    assert!(
        result_33.terminates,
        "CRITICAL BREAK: Pattern 33 did not terminate with large scope_id"
    );
}
