let branches = vec![
    Arc::new(|mut data: i32| { data *= 2; Ok(data) }),
    Arc::new(|mut data: i32| { data *= 3; Ok(data) }),
    Arc::new(|mut data: i32| { data *= 5; Ok(data) }),
];

let pattern = ParallelSplitPattern::new(branches)?;
let results = pattern.execute(10)?;
assert_eq!(results.len(), 3); // [20, 30, 50]