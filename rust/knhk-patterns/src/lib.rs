// rust/knhk-patterns/src/lib.rs
// Van der Aalst workflow patterns for KNHK pipeline orchestration

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod composition;
pub mod ffi;
pub mod hook_patterns;
pub mod hot_path;
pub mod patterns;
pub mod pipeline_ext;

#[cfg(feature = "unrdf")]
pub mod unrdf_patterns;

pub mod hybrid_patterns;

pub use ffi::PatternType;
pub use patterns::{
    ArbitraryCyclesPattern, BranchFn, CancellationPattern, ConditionFn, DeferredChoicePattern,
    DiscriminatorPattern, ExclusiveChoicePattern, ImplicitTerminationPattern, MultiChoicePattern,
    ParallelSplitPattern, Pattern, PatternError, PatternResult, SequencePattern,
    SimpleMergePattern, SynchronizationPattern, TimeoutPattern,
};

pub use composition::{CompositePattern, PatternBuilder};
pub use hook_patterns::{
    create_hook_context, create_hook_context_from_components, HookChoicePattern, HookCondition,
    HookParallelPattern, HookRetryCondition, HookRetryPattern, HookSequencePattern,
};
pub use pipeline_ext::PipelinePatternExt;

#[cfg(feature = "unrdf")]
pub use unrdf_patterns::{
    UnrdfChoicePattern, UnrdfHookCondition, UnrdfHookRetryCondition, UnrdfParallelPattern,
    UnrdfRetryPattern, UnrdfSequencePattern,
};

pub use hybrid_patterns::{
    HybridChoicePattern, HybridExecutionResult, HybridHookCondition, HybridParallelPattern,
    HybridSequencePattern,
};

// Hot path C kernel API (for maximum performance)
pub use hot_path::{
    cancellation_hot, discriminator_hot, discriminator_simd_hot, implicit_termination_hot,
    timeout_hot, HotPathError, HotPathResult, PatternContextBuilder,
};
