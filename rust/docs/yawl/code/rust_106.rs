// ✅ Good: Retry with backoff
let pattern = ArbitraryCyclesPattern::new(
    processor,
    Arc::new(|result| result.should_retry()),
    3, // max attempts
)?;