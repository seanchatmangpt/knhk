use super::{Pattern, PatternError, PatternExecutionContext, PatternExecutionResult, PatternId};
use std::collections::HashMap;
use std::sync::Arc;

pub struct PatternRegistry {
    execs: HashMap<u8, Arc<dyn Pattern + Send + Sync>>,
}

impl PatternRegistry {
    pub fn new() -> Self { Self { execs: HashMap::new() } }
    pub fn register(&mut self, p: Arc<dyn Pattern + Send + Sync>) {
        self.execs.insert(p.id(), p);
    }
    pub fn execute(&self, id: &PatternId, ctx: &PatternExecutionContext)
        -> Option<Result<PatternExecutionResult, PatternError>>
    {
        self.execs.get(&id.0).map(|p| { p.validate_ingress(ctx)?; p.execute(ctx) })
    }
    pub fn count(&self) -> usize { self.execs.len() }
}

pub trait RegisterAllExt { fn register_all_patterns(&mut self); }

impl RegisterAllExt for PatternRegistry {
    fn register_all_patterns(&mut self) {
        use crate::exec::*;
        macro_rules! reg { ($t:ty) => { self.register(Arc::new(<$t>::default())); }; }
        // Basic 1..5
        reg!(P01Sequence); reg!(P02ParallelSplit); reg!(P03Synchronization);
        reg!(P04ExclusiveChoice); reg!(P05SimpleMerge);
        // Advanced 6..11
        reg!(P06MultiChoice); reg!(P07StructuredSyncMerge); reg!(P08MultiMerge);
        reg!(P09Discriminator); reg!(P10ArbitraryCycles); reg!(P11ImplicitTermination);
        // MI 12..15
        reg!(P12MI_NoSync); reg!(P13MI_WithSync); reg!(P14MI_DesignTime); reg!(P15MI_Runtime);
        // State 16..18
        reg!(P16DeferredChoice); reg!(P17InterleavedParallel); reg!(P18Milestone);
        // Cancellation 19..25 (skip 23 naming collision → we use same slot mapping)
        reg!(P19CancelActivity); reg!(P20CancelCase); reg!(P21CancelRegion);
        reg!(P22MICancel); reg!(P24MIForceComplete); reg!(P25MICancelConditional);
        // Discriminators/loops 26..29
        reg!(P26BlockingDiscriminator); reg!(P27CancellingDiscriminator);
        reg!(P28StructuredLoop); reg!(P29Recursion);
        // Triggers 30..31
        reg!(P30TransientTrigger); reg!(P31PersistentTrigger);
        // Joins 33..37
        reg!(P33StaticPartialJoin); reg!(P34DynamicPartialJoin);
        reg!(P35GeneralizedAndJoin); reg!(P36LocalSyncMerge); reg!(P37GeneralSyncMerge);
        // Threads 38..39
        reg!(P38ThreadMerge); reg!(P39ThreadSplit);
        // Termination 40..43
        reg!(P40ExplicitTermination); reg!(P41ImplicitTermination2);
        reg!(P42MultiEndTermination); reg!(P43TerminationWithCancellation);
    }
}
src/exec/mod.rs (minimal viable pattern implementations)
use crate::{Pattern, PatternExecutionContext, PatternExecutionResult, PatternError};

// Helper macro for boilerplate executors
macro_rules! patt {
    ($name:ident, $id:expr, $body:expr) => {
        #[derive(Default)]
        pub struct $name;
        impl Pattern for $name {
            fn id(&self) -> u8 { $id }
            fn execute(&self, ctx: &PatternExecutionContext)
                -> Result<PatternExecutionResult, PatternError>
            { $body(ctx) }
        }
    };
}

patt!(P01Sequence, 1, |ctx| Ok(PatternExecutionResult::ok(vec!["next".into()])));
patt!(P02ParallelSplit, 2, |_ctx| Ok(PatternExecutionResult::ok(vec!["a".into(),"b".into()])));
patt!(P03Synchronization, 3, |_ctx| Ok(PatternExecutionResult::ok(vec!["after_sync".into()])));
patt!(P04ExclusiveChoice, 4, |_ctx| Ok(PatternExecutionResult::ok(vec!["chosen".into()])));
patt!(P05SimpleMerge, 5, |_ctx| Ok(PatternExecutionResult::ok(vec!["merged".into()])));
patt!(P06MultiChoice, 6, |_ctx| Ok(PatternExecutionResult::ok(vec!["x".into(),"y".into()])));
patt!(P07StructuredSyncMerge, 7, |_ctx| Ok(PatternExecutionResult::ok(vec!["after_or_join".into()])));
patt!(P08MultiMerge, 8, |_ctx| Ok(PatternExecutionResult::ok(vec!["pass".into()])));
patt!(P09Discriminator, 9, |_ctx| Ok(PatternExecutionResult::ok(vec!["first_wins".into()])));
patt!(P10ArbitraryCycles, 10, |_ctx| Ok(PatternExecutionResult::ok(vec!["loop_next".into()])));
patt!(P11ImplicitTermination, 11, |_ctx| Ok(PatternExecutionResult::ok(vec![])));
patt!(P12MI_NoSync, 12, |_ctx| Ok(PatternExecutionResult::ok(vec!["mi1".into(),"mi2".into()])));
patt!(P13MI_WithSync, 13, |_ctx| Ok(PatternExecutionResult::ok(vec!["mi_join".into()])));
patt!(P14MI_DesignTime, 14, |_ctx| Ok(PatternExecutionResult::ok(vec!["mi_d".into()])));
patt!(P15MI_Runtime, 15, |_ctx| Ok(PatternExecutionResult::ok(vec!["mi_r".into()])));
patt!(P16DeferredChoice, 16, |_ctx| Ok(PatternExecutionResult::ok(vec!["event_path".into()])));
patt!(P17InterleavedParallel, 17, |_ctx| Ok(PatternExecutionResult::ok(vec!["interleave_next".into()])));
patt!(P18Milestone, 18, |_ctx| Ok(PatternExecutionResult::ok(vec!["milestone_pass".into()])));
patt!(P19CancelActivity, 19, |_ctx| Ok(PatternExecutionResult::ok(vec![])));
patt!(P20CancelCase, 20, |_ctx| Ok(PatternExecutionResult::ok(vec![])));
patt!(P21CancelRegion, 21, |_ctx| Ok(PatternExecutionResult::ok(vec![])));
patt!(P22MICancel, 22, |_ctx| Ok(PatternExecutionResult::ok(vec![])));
patt!(P24MIForceComplete, 24, |_ctx| Ok(PatternExecutionResult::ok(vec![])));
patt!(P25MICancelConditional, 25, |_ctx| Ok(PatternExecutionResult::ok(vec![])));
patt!(P26BlockingDiscriminator, 26, |_ctx| Ok(PatternExecutionResult::ok(vec!["after_blocking".into()])));
patt!(P27CancellingDiscriminator, 27, |_ctx| Ok(PatternExecutionResult::ok(vec!["after_cancel".into()])));
patt!(P28StructuredLoop, 28, |_ctx| Ok(PatternExecutionResult::ok(vec!["loop_next".into()])));
patt!(P29Recursion, 29, |_ctx| Ok(PatternExecutionResult::ok(vec!["recurse".into()])));
patt!(P30TransientTrigger, 30, |_ctx| Ok(PatternExecutionResult::ok(vec!["onetime".into()])));
patt!(P31PersistentTrigger, 31, |_ctx| Ok(PatternExecutionResult::ok(vec!["persist".into()])));
patt!(P33StaticPartialJoin, 33, |_ctx| Ok(PatternExecutionResult::ok(vec!["partial".into()])));
patt!(P34DynamicPartialJoin, 34, |_ctx| Ok(PatternExecutionResult::ok(vec!["partial_dyn".into()])));
patt!(P35GeneralizedAndJoin, 35, |_ctx| Ok(PatternExecutionResult::ok(vec!["gand".into()])));
patt!(P36LocalSyncMerge, 36, |_ctx| Ok(PatternExecutionResult::ok(vec!["lsm".into()])));
patt!(P37GeneralSyncMerge, 37, |_ctx| Ok(PatternExecutionResult::ok(vec!["gsm".into()])));
patt!(P38ThreadMerge, 38, |_ctx| Ok(PatternExecutionResult::ok(vec!["tmerge".into()])));
patt!(P39ThreadSplit, 39, |_ctx| Ok(PatternExecutionResult::ok(vec!["tsplit_a".into(),"tsplit_b".into()])));
patt!(P40ExplicitTermination, 40, |_ctx| Ok(PatternExecutionResult::ok(vec![])));
patt!(P41ImplicitTermination2, 41, |_ctx| Ok(PatternExecutionResult::ok(vec![])));
patt!(P42MultiEndTermination, 42, |_ctx| Ok(PatternExecutionResult::ok(vec![])));
patt!(P43TerminationWithCancellation, 43, |_ctx| Ok(PatternExecutionResult::ok(vec![])));
crates/state