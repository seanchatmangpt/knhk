//! Basic Control Flow Patterns (1-5)

use crate::patterns::adapter::PatternAdapter;
use crate::patterns::{PatternExecutor, PatternId};
use knhk_patterns::{
    ExclusiveChoicePattern, ParallelSplitPattern, SequencePattern, SimpleMergePattern,
    SynchronizationPattern,
};
use serde_json::Value;
use std::sync::Arc;

/// Pattern 1: Sequence
pub fn create_sequence_pattern() -> (PatternId, Box<dyn PatternExecutor>) {
    // Create a simple sequence pattern that passes data through
    let branches: Vec<
        Arc<dyn Fn(Value) -> Result<Value, knhk_patterns::PatternError> + Send + Sync>,
    > = vec![Arc::new(|v: Value| {
        // Identity function - just pass through
        Ok(v)
    })];

    let pattern = SequencePattern::new(branches)
        .map(|p| Arc::new(p) as Arc<dyn knhk_patterns::Pattern<Value>>)
        .unwrap_or_else(|_| {
            // Fallback: create a no-op pattern
            #[allow(clippy::expect_used)] // Fallback pattern creation should never fail
            Arc::new(SequencePattern::new(vec![]).expect("Empty SequencePattern should never fail"))
        });

    let adapter = PatternAdapter::new(pattern, PatternId(1));
    (PatternId(1), Box::new(adapter))
}

/// Pattern 2: Parallel Split (AND-split)
pub fn create_parallel_split_pattern() -> (PatternId, Box<dyn PatternExecutor>) {
    let branches: Vec<
        Arc<dyn Fn(Value) -> Result<Value, knhk_patterns::PatternError> + Send + Sync>,
    > = vec![
        Arc::new(|v: Value| Ok(v.clone())),
        Arc::new(|v: Value| Ok(v.clone())),
    ];

    let pattern = ParallelSplitPattern::new(branches)
        .map(|p| Arc::new(p) as Arc<dyn knhk_patterns::Pattern<Value>>)
        .unwrap_or_else(|_| {
            Arc::new(
                #[allow(clippy::expect_used)] // Fallback pattern creation should never fail
                ParallelSplitPattern::new(vec![])
                    .expect("Empty ParallelSplitPattern should never fail"),
            )
        });

    let adapter = PatternAdapter::new(pattern, PatternId(2));
    (PatternId(2), Box::new(adapter))
}

/// Pattern 3: Synchronization (AND-join)
pub fn create_synchronization_pattern() -> (PatternId, Box<dyn PatternExecutor>) {
    let pattern = Arc::new(SynchronizationPattern::new()) as Arc<dyn knhk_patterns::Pattern<Value>>;

    let adapter = PatternAdapter::new(pattern, PatternId(3));
    (PatternId(3), Box::new(adapter))
}

/// Pattern 4: Exclusive Choice (XOR-split)
pub fn create_exclusive_choice_pattern() -> (PatternId, Box<dyn PatternExecutor>) {
    let choices: Vec<(
        Arc<dyn Fn(&Value) -> bool + Send + Sync>,
        Arc<dyn Fn(Value) -> Result<Value, knhk_patterns::PatternError> + Send + Sync>,
    )> = vec![(Arc::new(|_| true), Arc::new(|v: Value| Ok(v.clone())))];

    let pattern = ExclusiveChoicePattern::new(choices)
        .map(|p| Arc::new(p) as Arc<dyn knhk_patterns::Pattern<Value>>)
        .unwrap_or_else(|_| {
            Arc::new(
                #[allow(clippy::expect_used)] // Fallback pattern creation should never fail
                ExclusiveChoicePattern::new(vec![])
                    .expect("Empty ExclusiveChoicePattern should never fail"),
            )
        });

    let adapter = PatternAdapter::new(pattern, PatternId(4));
    (PatternId(4), Box::new(adapter))
}

/// Pattern 5: Simple Merge (XOR-join)
pub fn create_simple_merge_pattern() -> (PatternId, Box<dyn PatternExecutor>) {
    let pattern = Arc::new(SimpleMergePattern::new()) as Arc<dyn knhk_patterns::Pattern<Value>>;

    let adapter = PatternAdapter::new(pattern, PatternId(5));
    (PatternId(5), Box::new(adapter))
}
