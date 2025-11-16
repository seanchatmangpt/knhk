//! Newtype pattern integration tests

use knhk_workflow_engine::types::newtypes::{
    BatchSize, PriorityLevel, RetryCount, TickCount, TimeoutMs,
};

#[test]
fn test_priority_level_constants() {
    assert_eq!(PriorityLevel::MIN.value(), 0);
    assert_eq!(PriorityLevel::MAX.value(), 255);
    assert_eq!(PriorityLevel::NORMAL.value(), 128);
    assert_eq!(PriorityLevel::HIGH.value(), 192);
    assert_eq!(PriorityLevel::CRITICAL.value(), 255);
}

#[test]
fn test_priority_level_checks() {
    assert!(!PriorityLevel::LOW.is_high());
    assert!(!PriorityLevel::NORMAL.is_high());
    assert!(PriorityLevel::HIGH.is_high());
    assert!(PriorityLevel::CRITICAL.is_critical());
}

#[test]
fn test_priority_level_ordering() {
    assert!(PriorityLevel::HIGH > PriorityLevel::NORMAL);
    assert!(PriorityLevel::CRITICAL > PriorityLevel::HIGH);
    assert!(PriorityLevel::LOW < PriorityLevel::NORMAL);
}

#[test]
fn test_timeout_ms_constants() {
    assert_eq!(TimeoutMs::DEFAULT.as_millis(), 30_000);
    assert_eq!(TimeoutMs::SHORT.as_millis(), 5_000);
    assert_eq!(TimeoutMs::LONG.as_millis(), 120_000);
    assert_eq!(TimeoutMs::MAX_TIMEOUT.as_millis(), 300_000);
}

#[test]
fn test_timeout_ms_clamping() {
    let timeout = TimeoutMs::new(500_000); // Exceeds max
    assert_eq!(timeout, TimeoutMs::MAX_TIMEOUT);

    let normal = TimeoutMs::new(60_000);
    assert_eq!(normal.as_millis(), 60_000);
}

#[test]
fn test_timeout_from_seconds() {
    let timeout = TimeoutMs::from_secs(60);
    assert_eq!(timeout.as_millis(), 60_000);
}

#[test]
fn test_timeout_expiration() {
    let timeout = TimeoutMs::new(1000); // 1 second

    let not_expired = std::time::Duration::from_millis(500);
    assert!(!timeout.is_expired(not_expired));

    let expired = std::time::Duration::from_millis(1500);
    assert!(timeout.is_expired(expired));
}

#[test]
fn test_retry_count_operations() {
    let retries = RetryCount::new(3);
    assert_eq!(retries.value(), 3);
    assert!(!retries.is_exhausted());

    let decremented = retries.decrement();
    assert_eq!(decremented.value(), 2);

    let incremented = retries.increment();
    assert_eq!(incremented.value(), 4);
}

#[test]
fn test_retry_count_exhaustion() {
    let retries = RetryCount::new(0);
    assert!(retries.is_exhausted());

    let non_exhausted = RetryCount::new(1);
    assert!(!non_exhausted.is_exhausted());
}

#[test]
fn test_retry_count_clamping() {
    let max_retries = RetryCount::new(100); // Exceeds max
    assert_eq!(max_retries, RetryCount::MAX_RETRIES);
}

#[test]
fn test_batch_size_validation() {
    let valid = BatchSize::new(100);
    assert!(valid.is_ok());
    assert_eq!(valid.unwrap().value(), 100);

    let too_large = BatchSize::new(20_000);
    assert!(too_large.is_ok());
    assert_eq!(too_large.unwrap(), BatchSize::MAX);

    let zero = BatchSize::new(0);
    assert!(zero.is_err());
}

#[test]
fn test_batch_size_unchecked() {
    let batch = BatchSize::new_unchecked(1000);
    assert_eq!(batch.value(), 1000);
    assert_eq!(batch.as_usize(), 1000);
}

#[test]
fn test_tick_count_chatman_constant() {
    let within = TickCount::new(5);
    assert!(within.is_within_budget());
    assert!(!within.exceeds_budget());

    let at_limit = TickCount::CHATMAN_CONSTANT;
    assert!(at_limit.is_within_budget());
    assert!(!at_limit.exceeds_budget());

    let exceeds = TickCount::new(10);
    assert!(!exceeds.is_within_budget());
    assert!(exceeds.exceeds_budget());
}

#[test]
fn test_tick_count_arithmetic() {
    let tick1 = TickCount::new(3);
    let tick2 = TickCount::new(2);

    let sum = tick1 + tick2;
    assert_eq!(sum.value(), 5);

    let diff = tick1 - tick2;
    assert_eq!(diff.value(), 1);
}

#[test]
fn test_tick_count_saturation() {
    let small = TickCount::new(2);
    let large = TickCount::new(5);

    let diff = small - large; // Would underflow, but saturates to 0
    assert_eq!(diff.value(), 0);
}

#[test]
fn test_newtype_zero_cost_size() {
    // All newtypes should be same size as their wrapped primitive
    assert_eq!(
        std::mem::size_of::<PriorityLevel>(),
        std::mem::size_of::<u8>()
    );
    assert_eq!(
        std::mem::size_of::<TimeoutMs>(),
        std::mem::size_of::<u64>()
    );
    assert_eq!(
        std::mem::size_of::<RetryCount>(),
        std::mem::size_of::<u32>()
    );
    assert_eq!(
        std::mem::size_of::<BatchSize>(),
        std::mem::size_of::<u32>()
    );
    assert_eq!(
        std::mem::size_of::<TickCount>(),
        std::mem::size_of::<u64>()
    );
}

#[test]
fn test_newtype_display() {
    assert_eq!(PriorityLevel::new(128).to_string(), "priority:128");
    assert_eq!(TimeoutMs::new(5000).to_string(), "5s");
    assert_eq!(TimeoutMs::new(500).to_string(), "500ms");
    assert_eq!(RetryCount::new(3).to_string(), "3 retries");
    assert_eq!(BatchSize::new_unchecked(100).to_string(), "batch:100");
    assert_eq!(TickCount::new(5).to_string(), "5 ticks");
}

#[test]
fn test_newtype_serialization() {
    let priority = PriorityLevel::HIGH;
    let json = serde_json::to_string(&priority).expect("should serialize");
    let deserialized: PriorityLevel =
        serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(priority, deserialized);
}

#[test]
fn test_newtype_defaults() {
    assert_eq!(PriorityLevel::default(), PriorityLevel::NORMAL);
    assert_eq!(TimeoutMs::default(), TimeoutMs::DEFAULT);
    assert_eq!(RetryCount::default(), RetryCount::DEFAULT);
    assert_eq!(BatchSize::default(), BatchSize::DEFAULT);
    assert_eq!(TickCount::default(), TickCount::ZERO);
}
