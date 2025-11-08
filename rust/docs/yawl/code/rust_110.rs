use knhk_patterns::hook_patterns::HookRetryPattern;

let pattern = HookRetryPattern::new(
    predicate,
    |receipt: &Receipt| receipt.ticks == 0, // Should retry if failed
    3, // max attempts
)?;
let results = pattern.execute_hooks(&context)?;