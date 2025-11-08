let branch = Arc::new(|mut data: i32| {
    data += 1;
    Ok(data)
});

let should_continue = Arc::new(|data: &i32| *data < 10);

let pattern = ArbitraryCyclesPattern::new(branch, should_continue, 100)?;
let results = pattern.execute(5)?;
assert_eq!(results[0], 10); // Stopped when value == 10