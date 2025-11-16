//! Test that deadlock cycles are detected at compile time

use knhk_workflow_macros::workflow;

workflow! {
    name: DeadlockWorkflow,
    patterns: [Sequence],

    states: {
        A -> B,
        B -> C,
        C -> A,  // Cycle: A -> B -> C -> A
    },

    constraints: {
        max_duration: 30_000,
    },
}

fn main() {}
