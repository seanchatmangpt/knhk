use knhk_patterns::hook_patterns::HookChoicePattern;

let choices = vec![
    (|receipt: &Receipt| receipt.ticks > 4, predicate1),
    (|receipt: &Receipt| receipt.ticks <= 4, predicate2),
];

let pattern = HookChoicePattern::new(choices)?;
let results = pattern.execute_hooks(&context)?;