//! Integration tests for workflow macros

use knhk_workflow_macros::workflow;

#[test]
fn test_simple_workflow() {
    workflow! {
        name: SimpleWorkflow,
        patterns: [Sequence],

        states: {
            Initial -> Processing,
            Processing -> Complete,
        },

        constraints: {
            max_duration: 30_000,
            max_concurrency: 10,
        },
    }

    // Create workflow instance
    let wf = SimpleWorkflow::new("test-1".to_string());
    assert_eq!(SimpleWorkflow::CALCULATED_COMPLEXITY, 1);
    assert!(!SimpleWorkflow::HAS_DEADLOCK);
}

#[test]
fn test_branching_workflow() {
    workflow! {
        name: BranchingWorkflow,
        patterns: [ExclusiveChoice, SimpleMerge],

        states: {
            Initial -> Decision,
            Decision -> [PathA, PathB],
            PathA -> Complete,
            PathB -> Complete,
        },

        constraints: {
            max_duration: 30_000,
        },
    }

    let wf = BranchingWorkflow::new("test-2".to_string());
    assert!(BranchingWorkflow::CALCULATED_COMPLEXITY > 1);
}

#[test]
fn test_parallel_workflow() {
    workflow! {
        name: ParallelWorkflow,
        patterns: [ParallelSplit, Synchronization],

        states: {
            Initial -> Fork,
            Fork -> [TaskA, TaskB, TaskC],
            [TaskA, TaskB, TaskC] -> Join,
            Join -> Complete,
        },

        constraints: {
            max_duration: 60_000,
            max_concurrency: 3,
        },
    }

    let wf = ParallelWorkflow::new("test-3".to_string());
    assert!(!ParallelWorkflow::HAS_DEADLOCK);
}

#[test]
fn test_user_registration_workflow() {
    workflow! {
        name: UserRegistration,
        patterns: [Sequence, ExclusiveChoice, ParallelSplit, Synchronization],

        states: {
            Initial -> ValidateEmail,
            ValidateEmail -> [CreateAccount, RejectInvalid],
            CreateAccount -> [SendWelcome, CreateProfile],
            [SendWelcome, CreateProfile] -> Complete,
            RejectInvalid -> Complete,
        },

        constraints: {
            max_duration: 30_000,
            max_concurrency: 100,
        },
    }

    // Test workflow creation
    let wf = UserRegistration::new("user-123".to_string());

    // Test compile-time constants
    assert!(UserRegistration::CALCULATED_COMPLEXITY > 0);
    assert!(!UserRegistration::HAS_DEADLOCK);
}
