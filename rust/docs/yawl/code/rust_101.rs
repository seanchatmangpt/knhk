let composite = CompositePattern::Retry {
    pattern: Box::new(pattern),
    should_continue: Arc::new(|data| data.value < 10),
    max_attempts: 100,
};