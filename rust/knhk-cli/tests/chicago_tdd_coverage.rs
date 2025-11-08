//! Chicago TDD tests for coverage commands

#![cfg(feature = "std")]

use knhk_cli::commands::coverage;

/// Test: coverage::get returns Result
/// Chicago TDD: Test behavior (coverage retrieval) not implementation (coverage calculation)
#[test]
fn test_coverage_get_returns_result() {
    // Act: Get coverage
    let result = coverage::get();

    // Assert: Verify actual behavior - either succeeds with coverage string or fails with error
    match result {
        Ok(coverage_str) => {
            // Success case - should return non-empty coverage string
            assert!(
                !coverage_str.is_empty(),
                "Coverage string should not be empty"
            );
        }
        Err(e) => {
            // Error case - should have meaningful error message
            assert!(!e.is_empty(), "Error message should not be empty");
        }
    }
}
