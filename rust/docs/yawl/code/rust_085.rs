use knhk_patterns::*;
use std::sync::Arc;

// Pattern 1: Sequence
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

// Pattern 2: Parallel Split
let branches = vec![
    Arc::new(|mut data: i32| { data *= 2; Ok(data) }),
    Arc::new(|mut data: i32| { data *= 3; Ok(data) }),
];

let pattern = ParallelSplitPattern::new(branches)?;
let results = pattern.execute(10)?;
assert_eq!(results.len(), 2); // [20, 30]

// Pattern 4: Exclusive Choice
let choices = vec![
    (
        Arc::new(|data: &i32| *data < 0) as ConditionFn<i32>,
        Arc::new(|mut data: i32| { data = -1; Ok(data) }) as BranchFn<i32>,
    ),
    (
        Arc::new(|data: &i32| *data >= 0),
        Arc::new(|mut data: i32| { data = 100; Ok(data) }),
    ),
];

let pattern = ExclusiveChoicePattern::new(choices)?;
let results = pattern.execute(5)?;
assert_eq!(results[0], 100);