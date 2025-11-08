//! Assertion Helpers
//!
//! Provides assertion utilities following Chicago TDD principles.

/// Assert that a result is successful
pub fn assert_success<T, E: std::fmt::Debug>(result: &Result<T, E>) {
    assert!(
        result.is_ok(),
        "Expected success, but got error: {:?}",
        result.as_ref().err()
    );
}

/// Assert that a result is an error
pub fn assert_error<T: std::fmt::Debug, E>(result: &Result<T, E>) {
    assert!(
        result.is_err(),
        "Expected error, but got success: {:?}",
        result.as_ref().ok()
    );
}

/// Assert that two values are equal with a custom message
pub fn assert_eq_with_msg<T: std::fmt::Debug + PartialEq>(actual: &T, expected: &T, msg: &str) {
    assert_eq!(
        actual, expected,
        "{}: expected {:?}, got {:?}",
        msg, expected, actual
    );
}

/// Assert that a value is within a range
pub fn assert_in_range<T: PartialOrd + std::fmt::Debug>(value: &T, min: &T, max: &T, msg: &str) {
    assert!(
        value >= min && value <= max,
        "{}: value {:?} not in range [{:?}, {:?}]",
        msg,
        value,
        min,
        max
    );
}
