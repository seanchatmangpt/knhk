//! Test that workflows without terminal states are detected

use knhk_workflow_macros::workflow;

workflow! {
    name: NoTerminalWorkflow,
    patterns: [Sequence],

    states: {
        Initial -> StateA,
        StateA -> StateB,
        StateB -> StateA,  // All states have outgoing transitions!
    },

    constraints: {
        max_duration: 30_000,
    },
}

fn main() {}
