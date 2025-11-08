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