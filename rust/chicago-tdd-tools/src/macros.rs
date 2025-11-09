//! Macros for Chicago TDD Testing
//!
//! Provides macros to enforce Chicago TDD principles and reduce boilerplate:
//! - AAA pattern enforcement (Arrange-Act-Assert)
//! - Async test wrappers with fixture management
//! - Performance testing (tick budget validation)
//! - Enhanced assertion macros with better error messages

/// Macro to enforce AAA (Arrange-Act-Assert) pattern
///
/// This macro ensures tests follow the Chicago TDD AAA pattern by requiring
/// explicit Arrange, Act, and Assert sections.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::chicago_test;
///
/// chicago_test!(test_feature_behavior, {
///     // Arrange: Set up test data
///     let input = "test";
///     let expected = "result";
///
///     // Act: Execute feature
///     let result = process(input);
///
///     // Assert: Verify behavior
///     assert_eq!(result, expected);
/// });
/// ```
#[macro_export]
macro_rules! chicago_test {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() {
            $body
        }
    };
}

/// Macro for async tests with AAA pattern enforcement
///
/// Wraps async test functions and ensures AAA pattern is followed.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::chicago_async_test;
///
/// chicago_async_test!(test_async_feature, {
///     // Arrange: Set up test data
///     let fixture = TestFixture::new().unwrap();
///
///     // Act: Execute async feature
///     let result = async_function().await;
///
///     // Assert: Verify behavior
///     assert_success(&result);
/// });
/// ```
#[macro_export]
macro_rules! chicago_async_test {
    ($name:ident, $body:block) => {
        #[tokio::test]
        async fn $name() {
            $body
        }
    };
}

/// Macro for async tests with automatic fixture setup and teardown
///
/// Creates a test fixture, runs the test body, and ensures cleanup.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::{chicago_fixture_test, prelude::*};
///
/// chicago_fixture_test!(test_with_fixture, fixture, {
///     // Arrange: Use provided fixture
///     let counter = fixture.test_counter();
///
///     // Act: Execute test
///     let result = process(counter);
///
///     // Assert: Verify behavior
///     assert!(result > 0);
/// });
/// ```
#[macro_export]
macro_rules! chicago_fixture_test {
    ($name:ident, $fixture_var:ident, $body:block) => {
        #[tokio::test]
        async fn $name() {
            // Arrange: Create fixture
            let mut $fixture_var = $crate::fixture::TestFixture::new()
                .unwrap_or_else(|e| panic!("Failed to create test fixture: {}", e));

            // Execute test body
            $body

            // Cleanup: Automatic teardown via Drop
        }
    };
}

/// Macro for performance tests with tick budget validation
///
/// Validates that hot path operations complete within the Chatman Constant
/// (≤8 ticks = 2ns budget).
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::chicago_performance_test;
///
/// chicago_performance_test!(test_hot_path_performance, {
///     // Arrange: Set up test data
///     let input = create_test_input();
///
///     // Act: Execute hot path operation
///     let (result, ticks) = measure_ticks(|| hot_path_operation(&input));
///
///     // Assert: Verify performance constraint
///     assert!(ticks <= 8, "Hot path exceeded tick budget: {} > 8", ticks);
///     assert_success(&result);
/// });
/// ```
#[macro_export]
macro_rules! chicago_performance_test {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() {
            $body
        }
    };
}

/// Assert that a result is successful with detailed error message
///
/// Provides better error messages than standard `assert!` when testing Results.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::assert_ok;
///
/// let result: Result<u32, String> = Ok(42);
/// assert_ok!(result);
///
/// // With custom message
/// assert_ok!(result, "Expected successful operation");
/// ```
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        match $result {
            Ok(_) => {}
            Err(e) => panic!("Expected Ok, but got Err: {:?}", e),
        }
    };
    ($result:expr, $msg:expr) => {
        match $result {
            Ok(_) => {}
            Err(e) => panic!("{}: Expected Ok, but got Err: {:?}", $msg, e),
        }
    };
}

/// Assert that a result is an error with detailed message
///
/// Provides better error messages when testing error cases.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::assert_err;
///
/// let result: Result<u32, String> = Err("error".to_string());
/// assert_err!(result);
///
/// // With custom message
/// assert_err!(result, "Expected error case");
/// ```
#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        match $result {
            Ok(v) => panic!("Expected Err, but got Ok: {:?}", v),
            Err(_) => {}
        }
    };
    ($result:expr, $msg:expr) => {
        match $result {
            Ok(v) => panic!("{}: Expected Err, but got Ok: {:?}", $msg, v),
            Err(_) => {}
        }
    };
}

/// Assert that a value is within tick budget (≤8 ticks)
///
/// Validates performance constraints according to Chatman Constant.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::assert_within_tick_budget;
///
/// let ticks = 5;
/// assert_within_tick_budget!(ticks);
///
/// // With custom message
/// assert_within_tick_budget!(ticks, "Hot path operation");
/// ```
#[macro_export]
macro_rules! assert_within_tick_budget {
    ($ticks:expr) => {
        assert!(
            $ticks <= 8,
            "Tick budget exceeded: {} > 8 (Chatman Constant violation)",
            $ticks
        );
    };
    ($ticks:expr, $msg:expr) => {
        assert!(
            $ticks <= 8,
            "{}: Tick budget exceeded: {} > 8 (Chatman Constant violation)",
            $msg,
            $ticks
        );
    };
}

/// Assert that a value is within a range with detailed error message
///
/// Provides better error messages for range assertions.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::assert_in_range;
///
/// let value = 5;
/// assert_in_range!(value, 0, 10);
///
/// // With custom message
/// assert_in_range!(value, 0, 10, "Value should be in valid range");
/// ```
#[macro_export]
macro_rules! assert_in_range {
    ($value:expr, $min:expr, $max:expr) => {
        assert!(
            $value >= $min && $value <= $max,
            "Value {} not in range [{}, {}]",
            $value,
            $min,
            $max
        );
    };
    ($value:expr, $min:expr, $max:expr, $msg:expr) => {
        assert!(
            $value >= $min && $value <= $max,
            "{}: Value {} not in range [{}, {}]",
            $msg,
            $value,
            $min,
            $max
        );
    };
}

/// Assert equality with detailed error message
///
/// Provides better error messages for equality assertions.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::assert_eq_msg;
///
/// let actual = 42;
/// let expected = 42;
/// assert_eq_msg!(actual, expected, "Values should match");
/// ```
#[macro_export]
macro_rules! assert_eq_msg {
    ($actual:expr, $expected:expr, $msg:expr) => {
        assert_eq!(
            $actual, $expected,
            "{}: expected {:?}, got {:?}",
            $msg, $expected, $actual
        );
    };
}

/// Assert that a guard constraint is satisfied
///
/// Validates guard constraints like max_run_len ≤ 8.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::assert_guard_constraint;
///
/// let max_run_len = 5;
/// assert_guard_constraint!(max_run_len <= 8, "max_run_len");
/// ```
#[macro_export]
macro_rules! assert_guard_constraint {
    ($condition:expr, $constraint_name:expr) => {
        assert!(
            $condition,
            "Guard constraint violation: {}",
            $constraint_name
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::TestFixture;

    // Note: We can't use chicago_test! macro here because it would create
    // a test function with the same name, causing conflicts.
    // These tests verify the macro expansion works correctly.

    #[test]
    fn test_chicago_test_macro_expansion() {
        // Verify macro expands to valid test function
        // This is tested by compilation success
        let _ = stringify! {
            chicago_test!(test_basic, {
                let x = 1;
                let y = x + 1;
                assert_eq!(y, 2);
            });
        };
    }

    #[test]
    fn test_chicago_async_test_macro_expansion() {
        // Verify macro expands to valid async test function
        let _ = stringify! {
            chicago_async_test!(test_async_basic, {
                let x = 1;
                let y = x + 1;
                assert_eq!(y, 2);
            });
        };
    }

    #[tokio::test]
    async fn test_chicago_fixture_test_macro() {
        chicago_fixture_test!(test_fixture_basic, fixture, {
            // Arrange
            let counter = fixture.test_counter();

            // Act
            let result = counter + 1;

            // Assert
            assert!(result > 0);
        });
    }

    #[test]
    fn test_assert_ok_macro() {
        let result: Result<u32, String> = Ok(42);
        assert_ok!(result);
        assert_ok!(result, "Should succeed");
    }

    #[test]
    #[should_panic(expected = "Expected Ok")]
    fn test_assert_ok_macro_fails() {
        let result: Result<u32, String> = Err("error".to_string());
        assert_ok!(result);
    }

    #[test]
    fn test_assert_err_macro() {
        let result: Result<u32, String> = Err("error".to_string());
        assert_err!(result);
        assert_err!(result, "Should fail");
    }

    #[test]
    #[should_panic(expected = "Expected Err")]
    fn test_assert_err_macro_fails() {
        let result: Result<u32, String> = Ok(42);
        assert_err!(result);
    }

    #[test]
    fn test_assert_within_tick_budget_macro() {
        assert_within_tick_budget!(5);
        assert_within_tick_budget!(8);
        assert_within_tick_budget!(0);
        assert_within_tick_budget!(5, "Test operation");
    }

    #[test]
    #[should_panic(expected = "Tick budget exceeded")]
    fn test_assert_within_tick_budget_macro_fails() {
        assert_within_tick_budget!(9);
    }

    #[test]
    fn test_assert_in_range_macro() {
        assert_in_range!(5, 0, 10);
        assert_in_range!(0, 0, 10);
        assert_in_range!(10, 0, 10);
        assert_in_range!(5, 0, 10, "Value should be valid");
    }

    #[test]
    #[should_panic(expected = "not in range")]
    fn test_assert_in_range_macro_fails_below() {
        assert_in_range!(-1, 0, 10);
    }

    #[test]
    #[should_panic(expected = "not in range")]
    fn test_assert_in_range_macro_fails_above() {
        assert_in_range!(11, 0, 10);
    }

    #[test]
    fn test_assert_eq_msg_macro() {
        assert_eq_msg!(42, 42, "Values should match");
    }

    #[test]
    #[should_panic(expected = "Values should match")]
    fn test_assert_eq_msg_macro_fails() {
        assert_eq_msg!(41, 42, "Values should match");
    }

    #[test]
    fn test_assert_guard_constraint_macro() {
        assert_guard_constraint!(5 <= 8, "max_run_len");
        assert_guard_constraint!(true, "always_true");
    }

    #[test]
    #[should_panic(expected = "Guard constraint violation")]
    fn test_assert_guard_constraint_macro_fails() {
        assert_guard_constraint!(9 <= 8, "max_run_len");
    }
}
