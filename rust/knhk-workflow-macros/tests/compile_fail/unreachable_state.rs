//! Test that unreachable states are detected at compile time

use knhk_workflow_macros::workflow;

workflow! {
    name: UnreachableWorkflow,
    patterns: [Sequence],

    states: {
        Initial -> StateA,
        StateA -> Complete,
        // StateB is unreachable!
        StateB -> Complete,
    },

    constraints: {
        max_duration: 30_000,
    },
}

fn main() {}
