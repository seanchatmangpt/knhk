// rust/knhk-patterns/src/lib.rs
// Van der Aalst workflow patterns for KNHK pipeline orchestration

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod ffi;
pub mod patterns;
pub mod composition;
pub mod pipeline_ext;

pub use patterns::{
    Pattern, PatternType, PatternResult, PatternError,
    SequencePattern, ParallelSplitPattern, SynchronizationPattern,
    ExclusiveChoicePattern, SimpleMergePattern, MultiChoicePattern,
    ArbitraryCyclesPattern, DeferredChoicePattern,
};

pub use composition::{CompositePattern, PatternBuilder};
pub use pipeline_ext::PipelinePatternExt;
