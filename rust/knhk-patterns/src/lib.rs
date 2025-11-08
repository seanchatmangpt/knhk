// rust/knhk-patterns/src/lib.rs
// Van der Aalst workflow patterns for KNHK pipeline orchestration

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod ffi;
pub mod patterns;
pub mod composition;
pub mod pipeline_ext;
pub mod hook_patterns;

pub use ffi::PatternType;
pub use patterns::{
    Pattern, PatternResult, PatternError,
    SequencePattern, ParallelSplitPattern, SynchronizationPattern,
    ExclusiveChoicePattern, SimpleMergePattern, MultiChoicePattern,
    ArbitraryCyclesPattern, DeferredChoicePattern,
};

pub use composition::{CompositePattern, PatternBuilder};
pub use pipeline_ext::PipelinePatternExt;
pub use hook_patterns::{
    HookSequencePattern, HookParallelPattern, HookChoicePattern, HookRetryPattern,
    HookCondition, HookRetryCondition,
    create_hook_context, create_hook_context_from_components,
};
