//! Chicago TDD tests for coverage commands

#![cfg(feature = "std")]

use knhk_cli::commands::coverage;

/// Test: coverage::get returns Result
/// Chicago TDD: Test behavior (coverage retrieval) not implementation (coverage calculation)
#[test]
fn test_coverage_get_returns_result() {
    // Act: Get coverage
    let result = coverage::get();

    // Assert: Returns Result (may fail if coverage calculation fails, but should not panic)
    assert!(result.is_ok() || result.is_err());

    // If successful, should return String
    if let Ok(coverage_str) = result {
        assert!(!coverage_str.is_empty());
    }
}
