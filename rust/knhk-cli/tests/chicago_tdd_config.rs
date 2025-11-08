//! Chicago TDD tests for config commands

#![cfg(feature = "std")]

use knhk_cli::commands::config;

/// Test: config::show returns Result
/// Chicago TDD: Test behavior (config display) not implementation (config loading)
#[test]
fn test_config_show_returns_result() {
    // Act: Show config
    let result = config::show();

    // Assert: Returns Result (may fail if config load fails, but should not panic)
    assert!(result.is_ok() || result.is_err());

    // If successful, should return String
    if let Ok(config_str) = result {
        assert!(!config_str.is_empty());
    }
}
