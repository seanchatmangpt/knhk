use knhk_patterns::composition::PatternBuilder;

let workflow = PatternBuilder::new()
    .then(Arc::new(|mut data: i32| { data += 1; Ok(data) }))
    .parallel(vec![
        Arc::new(|mut data: i32| { data *= 2; Ok(data) }),
        Arc::new(|mut data: i32| { data *= 3; Ok(data) }),
    ])
    .retry(
        Arc::new(|mut data: i32| { data += 1; Ok(data) }),
        Arc::new(|data: &i32| *data < 10),
        100,
    )
    .build();

let results = workflow.execute(5)?;