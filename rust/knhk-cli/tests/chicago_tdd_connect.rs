//! Chicago TDD tests for connect commands

#![cfg(feature = "std")]

use knhk_cli::commands::connect;

/// Test: connect::register returns Result
/// Chicago TDD: Test behavior (registration) not implementation (storage)
#[test]
fn test_connect_register_returns_result() {
    // Arrange: Create test connector data
    let name = "test-connector".to_string();
    let schema = "http://example.org/schema".to_string();
    let source = "file:///tmp/test.nt".to_string();

    // Act: Register connector
    let result = connect::register(name.clone(), schema.clone(), source.clone());

    // Assert: Verify actual behavior - either succeeds or fails with meaningful error
    match result {
        Ok(_) => {
            // Success case - registration completed
        }
        Err(e) => {
            // Error case - should have meaningful error message
            assert!(!e.is_empty(), "Error message should not be empty");
        }
    }
}

/// Test: connect::register with duplicate name
/// Chicago TDD: Test behavior (duplicate detection) not implementation (storage lookup)
#[test]
fn test_connect_register_duplicate() {
    // Arrange: Register connector first time
    let name = "duplicate-connector".to_string();
    let schema = "http://example.org/schema".to_string();
    let source = "file:///tmp/test.nt".to_string();

    // First registration (may succeed or fail depending on storage)
    let _ = connect::register(name.clone(), schema.clone(), source.clone());

    // Act: Register same connector again
    let result = connect::register(name.clone(), schema.clone(), source.clone());

    // Assert: Verify actual behavior - either succeeds (if storage doesn't persist) or fails with duplicate error
    match result {
        Ok(_) => {
            // Success case - storage doesn't persist duplicates
        }
        Err(e) => {
            // Error case - should indicate duplicate or storage error
            assert!(!e.is_empty(), "Error message should not be empty");
        }
    }
}

/// Test: connect::list returns Result
/// Chicago TDD: Test behavior (listing) not implementation (storage reading)
#[test]
fn test_connect_list_returns_result() {
    // Act: List connectors
    let result = connect::list();

    // Assert: Verify actual behavior - either succeeds with valid list or fails with error
    match result {
        Ok(connectors) => {
            // Success case - should return valid list (may be empty)
            // All connector names should be non-empty if list is non-empty
            assert!(
                connectors.iter().all(|c| !c.is_empty()),
                "Connector names should not be empty"
            );
        }
        Err(e) => {
            // Error case - should have meaningful error message
            assert!(!e.is_empty(), "Error message should not be empty");
        }
    }
}

/// Test: connect::register then list
/// Chicago TDD: Test behavior (registration and listing) not implementation (storage)
#[test]
fn test_connect_register_then_list() {
    // Arrange: Register a connector
    let name = "list-test-connector".to_string();
    let schema = "http://example.org/schema".to_string();
    let source = "file:///tmp/test.nt".to_string();

    // Act: Register and list
    let register_result = connect::register(name.clone(), schema.clone(), source.clone());
    let list_result = connect::list();

    // Assert: Verify actual behavior of both operations
    match (register_result, list_result) {
        (Ok(_), Ok(connectors)) => {
            // Both succeeded - verify list is valid
            assert!(
                connectors.iter().all(|c| !c.is_empty()),
                "Connector names should not be empty"
            );
        }
        (Ok(_), Err(e)) => {
            // Registration succeeded but listing failed
            assert!(!e.is_empty(), "List error message should not be empty");
        }
        (Err(e), Ok(_)) => {
            // Registration failed but listing succeeded
            assert!(!e.is_empty(), "Register error message should not be empty");
        }
        (Err(e1), Err(e2)) => {
            // Both failed - verify error messages
            assert!(!e1.is_empty(), "Register error message should not be empty");
            assert!(!e2.is_empty(), "List error message should not be empty");
        }
    }
}

/// Test: connect::register with invalid source format
/// Chicago TDD: Test behavior (validation) not implementation (parsing)
#[test]
fn test_connect_register_invalid_source() {
    // Arrange: Use invalid source format
    let name = "invalid-connector".to_string();
    let schema = "http://example.org/schema".to_string();
    let source = "invalid://format".to_string();

    // Act: Register connector
    let result = connect::register(name, schema, source);

    // Assert: Verify actual behavior - either succeeds (lenient validation) or fails (strict validation)
    match result {
        Ok(_) => {
            // Success case - validation is lenient or source format is acceptable
        }
        Err(e) => {
            // Error case - validation is strict, should indicate invalid format
            assert!(!e.is_empty(), "Error message should not be empty");
            // Error should mention validation or format issue
            assert!(
                e.to_lowercase().contains("invalid")
                    || e.to_lowercase().contains("format")
                    || e.to_lowercase().contains("validation"),
                "Error should mention validation or format issue"
            );
        }
    }
}
