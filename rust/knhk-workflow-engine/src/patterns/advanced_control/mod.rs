//! Advanced Control Flow Patterns (26-39)

pub mod cancellation;
pub mod control;
pub mod discriminators;
pub mod loops;
pub mod triggers;

pub use cancellation::{
    AbortProcessInstancePattern, CancelActivityInstancePattern, CancelProcessInstancePattern,
    StopProcessInstancePattern,
};
pub use control::{
    ActivityInstanceMultipleThreadsPattern, DisableActivityPattern, SkipActivityPattern,
    ThreadMergePattern,
};
pub use discriminators::{BlockingDiscriminatorPattern, CancellingDiscriminatorPattern};
pub use loops::{RecursionPattern, StructuredLoopPattern};
pub use triggers::{PersistentTriggerPattern, TransientTriggerPattern};

use crate::patterns::{PatternExecutor, PatternId};

/// Pattern 26: Blocking Discriminator
pub fn create_pattern_26() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(26), Box::new(BlockingDiscriminatorPattern))
}

/// Pattern 27: Cancelling Discriminator
pub fn create_pattern_27() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(27), Box::new(CancellingDiscriminatorPattern))
}

/// Pattern 28: Structured Loop
pub fn create_pattern_28() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(28), Box::new(StructuredLoopPattern))
}

/// Pattern 29: Recursion
pub fn create_pattern_29() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(29), Box::new(RecursionPattern))
}

/// Pattern 30: Transient Trigger
pub fn create_pattern_30() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(30), Box::new(TransientTriggerPattern))
}

/// Pattern 31: Persistent Trigger
pub fn create_pattern_31() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(31), Box::new(PersistentTriggerPattern))
}

/// Pattern 32: Cancel Activity Instance
pub fn create_pattern_32() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(32), Box::new(CancelActivityInstancePattern))
}

/// Pattern 33: Cancel Process Instance
pub fn create_pattern_33() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(33), Box::new(CancelProcessInstancePattern))
}

/// Pattern 34: Stop Process Instance
pub fn create_pattern_34() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(34), Box::new(StopProcessInstancePattern))
}

/// Pattern 35: Abort Process Instance
pub fn create_pattern_35() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(35), Box::new(AbortProcessInstancePattern))
}

/// Pattern 36: Disable Activity
pub fn create_pattern_36() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(36), Box::new(DisableActivityPattern))
}

/// Pattern 37: Skip Activity
pub fn create_pattern_37() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(37), Box::new(SkipActivityPattern))
}

/// Pattern 38: Activity Instance in Multiple Threads
pub fn create_pattern_38() -> (PatternId, Box<dyn PatternExecutor>) {
    (
        PatternId(38),
        Box::new(ActivityInstanceMultipleThreadsPattern),
    )
}

/// Pattern 39: Thread Merge
pub fn create_pattern_39() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(39), Box::new(ThreadMergePattern))
}
