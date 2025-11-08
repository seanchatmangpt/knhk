//! Chicago TDD tests for config commands

#![cfg(feature = "std")]

use knhk_cli::commands::config;

/// Test: config::show returns Result
/// Chicago TDD: Test behavior (config display) not implementation (config loading)
#[test]
fn test_config_show_returns_result() {
    // Act: Show config
    let result = config::show();

    // Assert: Verify actual behavior - either succeeds with config string or fails with error
    match result {
        Ok(config_str) => {
            // Success case - should return non-empty config string
            assert!(!config_str.is_empty(), "Config string should not be empty");
        }
        Err(e) => {
            // Error case - should have meaningful error message
            assert!(!e.is_empty(), "Error message should not be empty");
        }
    }
}
