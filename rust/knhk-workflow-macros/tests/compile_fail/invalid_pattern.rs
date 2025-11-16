//! Test that invalid patterns are detected at compile time

use knhk_workflow_macros::workflow;

workflow! {
    name: InvalidPatternWorkflow,
    patterns: [NonExistentPattern],  // This pattern doesn't exist!

    states: {
        Initial -> Complete,
    },

    constraints: {
        max_duration: 30_000,
    },
}

fn main() {}
