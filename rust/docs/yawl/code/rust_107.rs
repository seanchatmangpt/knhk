use knhk_patterns::hook_patterns::HookSequencePattern;
use knhk_etl::hook_registry::HookRegistry;

let registry = HookRegistry::new();
let predicates = vec![predicate1, predicate2, predicate3];

let pattern = HookSequencePattern::new(predicates)?;
let context = HookExecutionContext {
    hook_registry: registry,
    predicate_runs: runs,
    soa_arrays: soa,
    tick_budget: 8,
};
let results = pattern.execute_hooks(&context)?;