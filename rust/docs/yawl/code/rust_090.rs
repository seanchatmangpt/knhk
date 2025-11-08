use knhk_patterns::*;
use std::sync::Arc;

let branch1 = Arc::new(|mut data: i32| {
    data *= 2;
    Ok(data)
});

let branch2 = Arc::new(|mut data: i32| {
    data += 10;
    Ok(data)
});

let pattern = SequencePattern::new(vec![branch1, branch2])?;
let results = pattern.execute(5)?;
assert_eq!(results[0], 20); // (5 * 2) + 10