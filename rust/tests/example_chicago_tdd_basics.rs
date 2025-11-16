//! Comprehensive examples of chicago-tdd-tools v1.3.0 basic capabilities
//! Demonstrates: test!, async_test!, fixture_test!, assertions, builders

use chicago_tdd_tools::prelude::*;

// ============================================================================
// 1. BASIC SYNCHRONOUS TESTS
// ============================================================================

test!(test_addition_simple, {
    // Arrange
    let x = 5;
    let y = 3;

    // Act
    let result = x + y;

    // Assert
    assert_eq!(result, 8);
});

test!(test_multiple_assertions, {
    // Arrange
    let numbers = vec![1, 2, 3, 4, 5];

    // Act
    let sum: i32 = numbers.iter().sum();
    let product: i32 = numbers.iter().product();
    let count = numbers.len();

    // Assert
    assert_eq!(sum, 15);
    assert_eq!(product, 120);
    assert_eq!(count, 5);
    assert!(count > 0);
});

// ============================================================================
// 2. RESULT TYPE ASSERTIONS
// ============================================================================

test!(test_result_assertions_ok, {
    // Arrange
    let success: Result<i32, String> = Ok(42);
    let failure: Result<i32, String> = Err("error message".to_string());

    // Act - Success case
    let value = success.unwrap_or(0);

    // Assert
    assert_ok!(success);
    assert_eq!(value, 42);
    assert_err!(failure);
});

test!(test_result_detailed_assertions, {
    // Arrange
    let result: Result<Vec<i32>, String> = Ok(vec![1, 2, 3]);

    // Act
    let vec_ref = result.as_ref();

    // Assert
    assert_ok!(result);
    assert!(vec_ref.is_ok());
    assert!(vec_ref.as_ref().unwrap().len() == 3);
});

// ============================================================================
// 3. RANGE & COMPARISON ASSERTIONS
// ============================================================================

test!(test_range_assertions, {
    // Arrange
    let value = 50;

    // Act & Assert
    assert_in_range!(value, 0, 100);
    assert_in_range!(value, 40, 60);
    assert!((0..100).contains(&value));
});

test!(test_comparison_chains, {
    // Arrange
    let a = 10;
    let b = 20;
    let c = 30;

    // Act & Assert
    assert!(a < b);
    assert!(b < c);
    assert!(a < b && b < c);
});

// ============================================================================
// 4. BOOLEAN & OPTION ASSERTIONS
// ============================================================================

test!(test_option_assertions, {
    // Arrange
    let some_value: Option<i32> = Some(42);
    let none_value: Option<i32> = None;

    // Act & Assert
    assert!(some_value.is_some());
    assert!(none_value.is_none());
    assert_eq!(some_value.unwrap_or(0), 42);
    assert_eq!(none_value.unwrap_or(0), 0);
});

test!(test_boolean_conditions, {
    // Arrange
    let condition_a = true;
    let condition_b = false;

    // Act & Assert
    assert!(condition_a);
    assert!(!condition_b);
});

// ============================================================================
// 5. ARRAY & COLLECTION ASSERTIONS
// ============================================================================

test!(test_collection_assertions, {
    // Arrange
    let vec = vec![1, 2, 3, 4, 5];
    let empty_vec: Vec<i32> = vec![];

    // Act
    let contains_three = vec.contains(&3);
    let is_sorted = vec.windows(2).all(|w| w[0] <= w[1]);

    // Assert
    assert!(contains_three);
    assert!(is_sorted);
    assert!(!empty_vec.is_empty() || empty_vec.len() == 0);
    assert_eq!(vec.len(), 5);
});

test!(test_string_assertions, {
    // Arrange
    let text = "Hello, World!";

    // Act & Assert
    assert!(text.contains("World"));
    assert!(text.starts_with("Hello"));
    assert!(text.ends_with("!"));
    assert_eq!(text.len(), 13);
});

// ============================================================================
// 6. CUSTOM ASSERTION MESSAGES
// ============================================================================

test!(test_assertions_with_messages, {
    // Arrange
    let result = 2 + 2;

    // Act & Assert with custom message
    assert_eq_msg!(result, 4, "Math is broken: expected 4, got {}", result);
    assert!(result > 0, "Result must be positive");
});

// ============================================================================
// 7. PATTERN MATCHING & STRUCT ASSERTIONS
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct TestUser {
    id: u32,
    name: String,
    email: String,
}

test!(test_struct_assertions, {
    // Arrange
    let user = TestUser {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Act
    let other_user = TestUser {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Assert
    assert_eq!(user, other_user);
    assert_eq!(user.id, 1);
    assert!(user.name.contains("Alice"));
});

// ============================================================================
// 8. ASYNC TESTS
// ============================================================================

async_test!(test_async_basic, {
    // Arrange
    let future = async { 5 + 3 };

    // Act
    let result = future.await;

    // Assert
    assert_eq!(result, 8);
});

async_test!(test_async_with_sleep, {
    // Arrange
    let start = std::time::Instant::now();

    // Act
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    let elapsed = start.elapsed();

    // Assert
    assert!(elapsed.as_millis() >= 10);
});

async_test!(test_async_multiple_operations, {
    // Arrange
    let data = vec![1, 2, 3, 4, 5];

    // Act
    let processed: Vec<i32> = futures::future::join_all(
        data.iter().map(|x| async move { x * 2 })
    )
    .await;

    // Assert
    assert_eq!(processed, vec![2, 4, 6, 8, 10]);
});

// ============================================================================
// 9. TIMEOUT & DEADLINE ASSERTIONS
// ============================================================================

async_test!(test_async_with_timeout, {
    // Arrange - 1 second timeout (default for async_test!)
    let future = async {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        42
    };

    // Act
    let result = future.await;

    // Assert
    assert_eq!(result, 42);
});

// ============================================================================
// 10. ERROR HANDLING IN TESTS
// ============================================================================

test!(test_error_handling, {
    // Arrange
    let result: Result<i32, String> = Err("Something went wrong".to_string());

    // Act & Assert
    match result {
        Ok(_) => panic!("Expected an error"),
        Err(e) => assert!(e.contains("wrong")),
    }
});

test!(test_panic_handling, {
    // This test demonstrates structured panic handling
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        assert_eq!(1, 1); // This doesn't panic
        1 + 1
    }));

    assert!(result.is_ok());
});

// ============================================================================
// 11. CLEANUP & RESOURCE MANAGEMENT
// ============================================================================

test!(test_resource_cleanup, {
    // Arrange - using RAII pattern
    let _file_path = "/tmp/test_file.txt";

    // Act
    let content = "test data";

    // Assert
    assert!(!content.is_empty());

    // Cleanup happens automatically when test ends (RAII)
});

// ============================================================================
// 12. PARAMETERIZED-STYLE TESTING
// ============================================================================

test!(test_multiple_cases, {
    // Arrange - test multiple cases in one test
    let test_cases = vec![
        (2, 3, 5),
        (10, 20, 30),
        (100, 200, 300),
        (0, 0, 0),
    ];

    // Act & Assert
    for (a, b, expected) in test_cases {
        let result = a + b;
        assert_eq_msg!(result, expected, "Failed for {} + {}", a, b);
    }
});

// ============================================================================
// 13. TEST ISOLATION & INDEPENDENCE
// ============================================================================

test!(test_isolation_one, {
    // Each test is independent - no shared state
    let value = 42;
    assert_eq!(value, 42);
});

test!(test_isolation_two, {
    // This test doesn't affect test_isolation_one
    let value = 100;
    assert_eq!(value, 100);
});

// ============================================================================
// 14. EXPECTED FAILURES (SHOULD PANIC)
// ============================================================================

test!(test_that_should_panic_is_avoided, {
    // Chicago TDD philosophy: tests should pass, not panic
    // If testing panic behavior, use catch_unwind
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        panic!("This is expected");
    }));

    assert!(result.is_err());
});

// ============================================================================
// 15. BENCHMARKING WITHIN TESTS
// ============================================================================

test!(test_with_timing, {
    // Arrange
    let start = std::time::Instant::now();
    let iterations = 1000;

    // Act
    let mut sum = 0;
    for i in 0..iterations {
        sum += i;
    }

    // Measure
    let elapsed = start.elapsed();

    // Assert - check both correctness and performance
    assert!(sum > 0);
    assert!(elapsed.as_millis() < 1000); // Should be fast
});
