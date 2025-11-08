// rust/knhk-patterns/tests/chicago_tdd_new_patterns.rs
// Chicago TDD tests for new workflow patterns (9, 11, 20, 21)
// Tests behavior (what patterns do), not implementation (how they do it)

use knhk_patterns::*;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

// ============================================================================
// Test Data Types
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
struct TestData {
    value: i32,
    id: String,
}

impl TestData {
    fn new(value: i32, id: &str) -> Self {
        Self {
            value,
            id: id.to_string(),
        }
    }
}

// ============================================================================
// Pattern 9: Discriminator Tests (First-Wins)
// ============================================================================

#[test]
fn test_discriminator_returns_first_successful_result() {
    // Arrange: Fast and slow branches
    let fast = Arc::new(|mut data: TestData| {
        std::thread::sleep(Duration::from_millis(10));
        data.id = "fast".to_string();
        Ok(data)
    });

    let slow = Arc::new(|mut data: TestData| {
        std::thread::sleep(Duration::from_millis(100));
        data.id = "slow".to_string();
        Ok(data)
    });

    let pattern = DiscriminatorPattern::new(vec![fast, slow]).unwrap();

    // Act
    let input = TestData::new(42, "input");
    let results = pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "fast"); // Fast branch wins
    assert_eq!(results[0].value, 42);
}

#[test]
fn test_discriminator_handles_failing_branches() {
    // Arrange: One failing, one succeeding branch
    let failing = Arc::new(|_data: TestData| {
        Err(PatternError::ExecutionFailed(
            "Intentional failure".to_string(),
        ))
    });

    let succeeding = Arc::new(|mut data: TestData| {
        std::thread::sleep(Duration::from_millis(20));
        data.id = "success".to_string();
        Ok(data)
    });

    let pattern = DiscriminatorPattern::new(vec![failing, succeeding]).unwrap();

    // Act
    let input = TestData::new(10, "input");
    let results = pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "success");
}

#[test]
fn test_discriminator_fails_when_all_branches_fail() {
    // Arrange: All failing branches
    let fail1 =
        Arc::new(|_data: TestData| Err(PatternError::ExecutionFailed("Fail 1".to_string())));

    let fail2 =
        Arc::new(|_data: TestData| Err(PatternError::ExecutionFailed("Fail 2".to_string())));

    let pattern = DiscriminatorPattern::new(vec![fail1, fail2]).unwrap();

    // Act
    let input = TestData::new(5, "input");
    let result = pattern.execute(input);

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_discriminator_race_condition_atomic() {
    // Arrange: Multiple concurrent branches competing
    let counter = Arc::new(AtomicUsize::new(0));

    let branches: Vec<_> = (0..5)
        .map(|i| {
            let counter = counter.clone();
            Arc::new(move |mut data: TestData| {
                // Increment counter atomically
                counter.fetch_add(1, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(5));
                data.id = format!("branch_{}", i);
                Ok(data)
            }) as Arc<dyn Fn(TestData) -> Result<TestData, PatternError> + Send + Sync>
        })
        .collect();

    let pattern = DiscriminatorPattern::new(branches).unwrap();

    // Act
    let input = TestData::new(99, "input");
    let results = pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 1); // Only one result
                                  // All branches attempted (raciness may mean some didn't finish, but at least one did)
    assert!(counter.load(Ordering::SeqCst) >= 1);
}

// ============================================================================
// Pattern 11: Implicit Termination Tests
// ============================================================================

#[test]
fn test_implicit_termination_waits_for_all_branches() {
    // Arrange: Multiple branches with different durations
    let completed = Arc::new(AtomicUsize::new(0));

    let branches: Vec<_> = (0..3)
        .map(|i| {
            let completed = completed.clone();
            Arc::new(move |mut data: TestData| {
                std::thread::sleep(Duration::from_millis(10 * (i + 1)));
                completed.fetch_add(1, Ordering::SeqCst);
                data.value += i as i32;
                Ok(data)
            }) as Arc<dyn Fn(TestData) -> Result<TestData, PatternError> + Send + Sync>
        })
        .collect();

    let pattern = ImplicitTerminationPattern::new(branches).unwrap();

    // Act
    let start = Instant::now();
    let input = TestData::new(10, "input");
    let results = pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 3); // All branches completed
    assert_eq!(completed.load(Ordering::SeqCst), 3); // All incremented
    assert!(start.elapsed() >= Duration::from_millis(30)); // Waited for slowest
}

#[test]
fn test_implicit_termination_collects_all_results() {
    // Arrange: Branches that modify data differently
    let branch1 = Arc::new(|mut data: TestData| {
        data.value *= 2;
        Ok(data)
    });

    let branch2 = Arc::new(|mut data: TestData| {
        data.value *= 3;
        Ok(data)
    });

    let branch3 = Arc::new(|mut data: TestData| {
        data.value *= 5;
        Ok(data)
    });

    let pattern = ImplicitTerminationPattern::new(vec![branch1, branch2, branch3]).unwrap();

    // Act
    let input = TestData::new(10, "input");
    let results = pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 3);
    let values: Vec<i32> = results.iter().map(|r| r.value).collect();
    assert!(values.contains(&20)); // 10 * 2
    assert!(values.contains(&30)); // 10 * 3
    assert!(values.contains(&50)); // 10 * 5
}

#[test]
fn test_implicit_termination_handles_partial_failures() {
    // Arrange: Some branches fail, some succeed
    let success1 = Arc::new(|data: TestData| Ok(data));

    let failure =
        Arc::new(|_data: TestData| Err(PatternError::ExecutionFailed("Intentional".to_string())));

    let success2 = Arc::new(|mut data: TestData| {
        data.value += 5;
        Ok(data)
    });

    let pattern = ImplicitTerminationPattern::new(vec![success1, failure, success2]).unwrap();

    // Act
    let input = TestData::new(10, "input");
    let results = pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 2); // Only successful branches
}

// ============================================================================
// Pattern 20: Timeout Tests
// ============================================================================

#[test]
fn test_timeout_succeeds_within_limit() {
    // Arrange: Fast branch
    let branch = Arc::new(|mut data: TestData| {
        std::thread::sleep(Duration::from_millis(10));
        data.value *= 2;
        Ok(data)
    });

    let pattern = TimeoutPattern::new(branch, 100).unwrap();

    // Act
    let input = TestData::new(5, "input");
    let results = pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value, 10);
}

#[test]
fn test_timeout_triggers_on_slow_branch() {
    // Arrange: Slow branch that exceeds timeout
    let slow_branch = Arc::new(|mut data: TestData| {
        std::thread::sleep(Duration::from_millis(200));
        data.value *= 2;
        Ok(data)
    });

    let pattern = TimeoutPattern::new(slow_branch, 50).unwrap();

    // Act
    let input = TestData::new(5, "input");
    let result = pattern.execute(input);

    // Assert
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Timeout"));
    }
}

#[test]
fn test_timeout_uses_fallback_on_timeout() {
    // Arrange: Slow primary, fast fallback
    let slow_primary = Arc::new(|mut data: TestData| {
        std::thread::sleep(Duration::from_millis(200));
        data.value = 999;
        Ok(data)
    });

    let fast_fallback = Arc::new(|mut data: TestData| {
        data.value = 42;
        data.id = "fallback".to_string();
        Ok(data)
    });

    let pattern = TimeoutPattern::with_fallback(slow_primary, 50, Some(fast_fallback)).unwrap();

    // Act
    let input = TestData::new(5, "input");
    let results = pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value, 42);
    assert_eq!(results[0].id, "fallback");
}

#[test]
fn test_timeout_uses_fallback_on_branch_failure() {
    // Arrange: Failing primary, succeeding fallback
    let failing_primary = Arc::new(|_data: TestData| {
        Err(PatternError::ExecutionFailed("Primary failed".to_string()))
    });

    let fallback = Arc::new(|mut data: TestData| {
        data.value = 100;
        Ok(data)
    });

    let pattern = TimeoutPattern::with_fallback(failing_primary, 100, Some(fallback)).unwrap();

    // Act
    let input = TestData::new(5, "input");
    let results = pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value, 100);
}

#[test]
fn test_timeout_zero_validation() {
    // Arrange & Act: Try to create timeout with 0ms
    let branch = Arc::new(|data: TestData| Ok(data));
    let result = TimeoutPattern::new(branch, 0);

    // Assert
    assert!(result.is_err());
}

// ============================================================================
// Pattern 21: Cancellation Tests
// ============================================================================

#[test]
fn test_cancellation_executes_when_not_cancelled() {
    // Arrange: Never cancel
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let cancel_fn = {
        let flag = cancel_flag.clone();
        Arc::new(move || flag.load(Ordering::SeqCst))
    };

    let branch = Arc::new(|mut data: TestData| {
        data.value *= 2;
        Ok(data)
    });

    let pattern = CancellationPattern::new(branch, cancel_fn).unwrap();

    // Act
    let input = TestData::new(5, "input");
    let results = pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value, 10);
}

#[test]
fn test_cancellation_prevents_execution_when_cancelled_before() {
    // Arrange: Already cancelled
    let cancel_flag = Arc::new(AtomicBool::new(true)); // Already cancelled
    let cancel_fn = {
        let flag = cancel_flag.clone();
        Arc::new(move || flag.load(Ordering::SeqCst))
    };

    let branch = Arc::new(|mut data: TestData| {
        data.value = 999; // Should never execute
        Ok(data)
    });

    let pattern = CancellationPattern::new(branch, cancel_fn).unwrap();

    // Act
    let input = TestData::new(5, "input");
    let result = pattern.execute(input);

    // Assert
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("cancelled"));
    }
}

#[test]
fn test_cancellation_detects_cancellation_after_execution() {
    // Arrange: Cancel after execution
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let cancel_fn = {
        let flag = cancel_flag.clone();
        Arc::new(move || flag.load(Ordering::SeqCst))
    };

    let flag_for_branch = cancel_flag.clone();
    let branch = Arc::new(move |mut data: TestData| {
        data.value *= 2;
        // Cancel after execution
        flag_for_branch.store(true, Ordering::SeqCst);
        Ok(data)
    });

    let pattern = CancellationPattern::new(branch, cancel_fn).unwrap();

    // Act
    let input = TestData::new(5, "input");
    let result = pattern.execute(input);

    // Assert
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("cancelled after execution"));
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_timeout_with_discriminator_integration() {
    // Arrange: Timeout wrapping a discriminator (race with timeout)
    let discriminator_branches = vec![
        Arc::new(|mut data: TestData| {
            std::thread::sleep(Duration::from_millis(20));
            data.id = "branch1".to_string();
            Ok(data)
        }) as Arc<dyn Fn(TestData) -> Result<TestData, PatternError> + Send + Sync>,
        Arc::new(|mut data: TestData| {
            std::thread::sleep(Duration::from_millis(10));
            data.id = "branch2".to_string();
            Ok(data)
        }),
    ];

    let discriminator = Arc::new(DiscriminatorPattern::new(discriminator_branches).unwrap());

    let timeout_branch =
        Arc::new(move |input: TestData| discriminator.execute(input).map(|mut v| v.pop().unwrap()));

    let timeout_pattern = TimeoutPattern::new(timeout_branch, 100).unwrap();

    // Act
    let input = TestData::new(42, "input");
    let results = timeout_pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "branch2"); // Faster branch wins
}

#[test]
fn test_cancellable_implicit_termination() {
    // Arrange: Implicit termination that can be cancelled
    let cancel_flag = Arc::new(AtomicBool::new(false));

    let cancel_fn = {
        let flag = cancel_flag.clone();
        Arc::new(move || flag.load(Ordering::SeqCst))
    };

    // Branches for implicit termination
    let flag1 = cancel_flag.clone();
    let branch1 = Arc::new(move |data: TestData| {
        std::thread::sleep(Duration::from_millis(10));
        flag1.store(true, Ordering::SeqCst); // Cancel partway through
        Ok(data)
    });

    let branch2 = Arc::new(|data: TestData| {
        std::thread::sleep(Duration::from_millis(20));
        Ok(data)
    });

    let implicit_term = Arc::new(ImplicitTerminationPattern::new(vec![branch1, branch2]).unwrap());

    let cancellable_branch =
        Arc::new(move |input: TestData| implicit_term.execute(input).map(|mut v| v.pop().unwrap()));

    let pattern = CancellationPattern::new(cancellable_branch, cancel_fn).unwrap();

    // Act
    let input = TestData::new(1, "input");
    let result = pattern.execute(input);

    // Assert: Should be cancelled after execution
    assert!(result.is_err());
}
