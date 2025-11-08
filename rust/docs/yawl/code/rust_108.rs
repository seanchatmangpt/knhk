use knhk_patterns::hook_patterns::HookParallelPattern;

let predicates = vec![predicate1, predicate2, predicate3];
let pattern = HookParallelPattern::new(predicates)?;
let results = pattern.execute_hooks(&context)?;