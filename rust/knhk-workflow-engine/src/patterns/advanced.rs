//! Advanced Branching Patterns (6-11)

use crate::patterns::adapter::PatternAdapter;
use crate::patterns::{PatternExecutor, PatternId};
use knhk_patterns::{
    ArbitraryCyclesPattern, DiscriminatorPattern, ImplicitTerminationPattern, MultiChoicePattern,
};
use serde_json::Value;
use std::sync::Arc;

/// Pattern 6: Multi-Choice (OR-split)
pub fn create_multi_choice_pattern() -> (PatternId, Box<dyn PatternExecutor>) {
    let choices: Vec<(
        Arc<dyn Fn(&Value) -> bool + Send + Sync>,
        Arc<dyn Fn(Value) -> Result<Value, knhk_patterns::PatternError> + Send + Sync>,
    )> = vec![(Arc::new(|_| true), Arc::new(|v: Value| Ok(v.clone())))];

    let pattern = MultiChoicePattern::new(choices)
        .map(|p| Arc::new(p) as Arc<dyn knhk_patterns::Pattern<Value>>)
        .unwrap_or_else(|_| {
            Arc::new(
                #[allow(clippy::expect_used)] // Fallback pattern creation should never fail
                MultiChoicePattern::new(vec![])
                    .expect("Empty MultiChoicePattern should never fail"),
            )
        });

    let adapter = PatternAdapter::new(pattern, "pattern:6:multi-choice".to_string());
    ("pattern:6:multi-choice".to_string(), Box::new(adapter))
}

/// Pattern 7: Structured Synchronizing Merge
pub fn create_structured_synchronizing_merge_pattern() -> (PatternId, Box<dyn PatternExecutor>) {
    // Similar to synchronization but for OR-join
    // For now, use synchronization pattern as base
    let pattern = Arc::new(knhk_patterns::SynchronizationPattern::new())
        as Arc<dyn knhk_patterns::Pattern<Value>>;

    let adapter = PatternAdapter::new(
        pattern,
        "pattern:7:structured-synchronizing-merge".to_string(),
    );
    (
        "pattern:7:structured-synchronizing-merge".to_string(),
        Box::new(adapter),
    )
}

/// Pattern 8: Multi-Merge
pub fn create_multi_merge_pattern() -> (PatternId, Box<dyn PatternExecutor>) {
    // Multi-merge: merge all incoming branches without synchronization
    // Similar to simple merge but handles multiple branches
    let pattern = Arc::new(knhk_patterns::SimpleMergePattern::new())
        as Arc<dyn knhk_patterns::Pattern<Value>>;

    let adapter = PatternAdapter::new(pattern, "pattern:8:multi-merge".to_string());
    ("pattern:8:multi-merge".to_string(), Box::new(adapter))
}

/// Pattern 9: Discriminator
pub fn create_discriminator_pattern() -> (PatternId, Box<dyn PatternExecutor>) {
    let branches: Vec<
        Arc<dyn Fn(Value) -> Result<Value, knhk_patterns::PatternError> + Send + Sync>,
    > = vec![Arc::new(|v: Value| Ok(v.clone()))];

    let pattern = DiscriminatorPattern::new(branches)
        .map(|p| Arc::new(p) as Arc<dyn knhk_patterns::Pattern<Value>>)
        .unwrap_or_else(|_| {
            Arc::new(
                #[allow(clippy::expect_used)] // Fallback pattern creation should never fail
                DiscriminatorPattern::new(vec![Arc::new(|v: Value| Ok(v))])
                    .expect("DiscriminatorPattern with valid branches should never fail"),
            )
        });

    let adapter = PatternAdapter::new(pattern, "pattern:9:discriminator".to_string());
    ("pattern:9:discriminator".to_string(), Box::new(adapter))
}

/// Pattern 10: Arbitrary Cycles
pub fn create_arbitrary_cycles_pattern() -> (PatternId, Box<dyn PatternExecutor>) {
    let max_iterations = 100;
    let pattern = ArbitraryCyclesPattern::new(
        Arc::new(|v: Value| Ok(v.clone())),
        Arc::new(|_| false),
        max_iterations,
    )
    .map(|p| Arc::new(p) as Arc<dyn knhk_patterns::Pattern<Value>>)
    .unwrap_or_else(|_| {
        Arc::new(
            ArbitraryCyclesPattern::new(Arc::new(|v: Value| Ok(v)), Arc::new(|_| false), 100)
                .expect("ArbitraryCyclesPattern with valid cycle count should never fail"),
        )
    });

    let adapter = PatternAdapter::new(pattern, "pattern:10:arbitrary-cycles".to_string());
    ("pattern:10:arbitrary-cycles".to_string(), Box::new(adapter))
}

/// Pattern 11: Implicit Termination
pub fn create_implicit_termination_pattern() -> (PatternId, Box<dyn PatternExecutor>) {
    let branches: Vec<
        Arc<dyn Fn(Value) -> Result<Value, knhk_patterns::PatternError> + Send + Sync>,
    > = vec![Arc::new(|v: Value| Ok(v.clone()))];

    let pattern = ImplicitTerminationPattern::new(branches)
        .map(|p| Arc::new(p) as Arc<dyn knhk_patterns::Pattern<Value>>)
        .unwrap_or_else(|_| {
            Arc::new(
                ImplicitTerminationPattern::new(vec![Arc::new(|v: Value| Ok(v))])
                    .expect("ArbitraryCyclesPattern with valid cycle count should never fail"),
            )
        });

    let adapter = PatternAdapter::new(pattern, "pattern:11:implicit-termination".to_string());
    (
        "pattern:11:implicit-termination".to_string(),
        Box::new(adapter),
    )
}
