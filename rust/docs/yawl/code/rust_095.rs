let choices = vec![
    (
        Arc::new(|data: &i32| *data > 0) as ConditionFn<i32>,
        Arc::new(|mut data: i32| { data *= 2; Ok(data) }) as BranchFn<i32>,
    ),
    (
        Arc::new(|data: &i32| *data < 10),
        Arc::new(|mut data: i32| { data += 5; Ok(data) }),
    ),
];

let pattern = MultiChoicePattern::new(choices)?;
let results = pattern.execute(5)?;
assert_eq!(results.len(), 2); // Both conditions match