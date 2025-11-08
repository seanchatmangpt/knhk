// ✅ Good: Single path selection
let pattern = ExclusiveChoicePattern::new(vec![
    (is_error, error_handler),
    (is_success, success_handler),
])?;