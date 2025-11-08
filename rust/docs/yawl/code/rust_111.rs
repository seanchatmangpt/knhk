use knhk_patterns::hook_patterns::*;

// Parallel validation followed by conditional routing
let parallel_pattern = HookParallelPattern::new(vec![pred1, pred2])?;
let choice_pattern = HookChoicePattern::new(choices)?;

// Execute in sequence
let results1 = parallel_pattern.execute_hooks(&context)?;
let results2 = choice_pattern.execute_hooks(&context)?;