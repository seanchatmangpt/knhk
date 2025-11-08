let composite = CompositePattern::Choice(vec![
    (condition1, Box::new(pattern1)),
    (condition2, Box::new(pattern2)),
]);