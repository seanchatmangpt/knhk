use knhk_patterns::hook_patterns::*;
use knhk_etl::hook_registry::HookRegistry;

let registry = HookRegistry::new();
// Register hooks for predicates
registry.register_hook(pred1, KernelType::ValidateSp, guard1, invariants1)?;
registry.register_hook(pred2, KernelType::ValidateSp, guard2, invariants2)?;

// Execute hooks in parallel
let pattern = HookParallelPattern::new(vec![pred1, pred2])?;
let context = HookExecutionContext {
    hook_registry: registry,
    predicate_runs: runs,
    soa_arrays: soa,
    tick_budget: 8,
};
let results = pattern.execute_hooks(&context)?;

// Execute hooks conditionally
let choices = vec![
    (|receipt: &Receipt| receipt.ticks > 4, pred1),
    (|_| true, pred2),
];
let pattern = HookChoicePattern::new(choices)?;
let results = pattern.execute_hooks(&context)?;

// Execute hooks with retry
let pattern = HookRetryPattern::new(
    pred1,
    |receipt: &Receipt| receipt.ticks == 0,
    3, // max attempts
)?;
let results = pattern.execute_hooks(&context)?;