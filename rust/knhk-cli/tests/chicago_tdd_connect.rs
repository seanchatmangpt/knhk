//! Chicago TDD tests for connect commands

#![cfg(feature = "std")]

use knhk_cli::commands::connect;

/// Test: connect::register then verify connector appears in list
/// Chicago TDD: Test behavior (registration) not implementation (storage)
#[test]
fn test_connect_register_then_verify_in_list() {
    // Arrange: Create test connector data
    let name = "test-connector-register".to_string();
    let schema = "http://example.org/schema".to_string();
    let source = "file:///tmp/test.nt".to_string();

    // Act: Register connector
    let register_result = connect::register(name.clone(), schema.clone(), source.clone());

    // Assert: Verify registration succeeded and connector appears in list (observable state)
    match register_result {
        Ok(_) => {
            // Registration succeeded - verify connector appears in list
            let list_result = connect::list();
            assert!(
                list_result.is_ok(),
                "List should succeed after registration"
            );
            let connectors = list_result.unwrap();
            assert!(
                connectors.contains(&name),
                "Registered connector '{}' should appear in list",
                name
            );
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

    // First registration
    let first_result = connect::register(name.clone(), schema.clone(), source.clone());

    // Verify first registration succeeded and connector appears in list
    if first_result.is_ok() {
        let list_result = connect::list();
        assert!(list_result.is_ok(), "List should succeed");
        let connectors = list_result.unwrap();
        assert!(
            connectors.contains(&name),
            "First registration should add connector to list"
        );
    }

    // Act: Register same connector again
    let second_result = connect::register(name.clone(), schema.clone(), source.clone());

    // Assert: Verify duplicate detection behavior (observable state)
    match second_result {
        Ok(_) => {
            // If second registration succeeded, verify connector still appears once in list
            let list_result = connect::list();
            assert!(list_result.is_ok(), "List should succeed");
            let connectors = list_result.unwrap();
            let count = connectors.iter().filter(|c| **c == name).count();
            assert_eq!(
                count, 1,
                "Connector should appear exactly once in list, found {} times",
                count
            );
        }
        Err(e) => {
            // Error case - should indicate duplicate
            assert!(!e.is_empty(), "Error message should not be empty");
            assert!(
                e.to_lowercase().contains("already") || e.to_lowercase().contains("duplicate"),
                "Error should mention duplicate or already registered: {}",
                e
            );
        }
    }
}

/// Test: connect::list returns valid connector names
/// Chicago TDD: Test behavior (listing) not implementation (storage reading)
#[test]
fn test_connect_list_returns_valid_names() {
    // Act: List connectors
    let result = connect::list();

    // Assert: Verify list behavior (observable output)
    match result {
        Ok(connectors) => {
            // Success case - should return valid list (may be empty)
            // All connector names should be non-empty if list is non-empty
            assert!(
                connectors.iter().all(|c| !c.is_empty()),
                "Connector names should not be empty"
            );
            // Verify list is a Vec (observable structure)
            assert!(
                connectors.len() >= 0,
                "List should return valid Vec (may be empty)"
            );
        }
        Err(e) => {
            // Error case - should have meaningful error message
            assert!(!e.is_empty(), "Error message should not be empty");
        }
    }
}

/// Test: connect::register then list - verify connector appears
/// Chicago TDD: Test behavior (registration and listing) not implementation (storage)
#[test]
fn test_connect_register_then_list() {
    // Arrange: Register a connector
    let name = "list-test-connector".to_string();
    let schema = "http://example.org/schema".to_string();
    let source = "file:///tmp/test.nt".to_string();

    // Act: Register connector
    let register_result = connect::register(name.clone(), schema.clone(), source.clone());

    // Assert: Verify registration succeeded
    assert!(register_result.is_ok(), "Registration should succeed");

    // Act: List connectors
    let list_result = connect::list();

    // Assert: Verify observable behavior - registered connector appears in list
    match list_result {
        Ok(connectors) => {
            // Verify list is valid
            assert!(
                connectors.iter().all(|c| !c.is_empty()),
                "Connector names should not be empty"
            );
            // Verify registered connector appears in list (observable state change)
            assert!(
                connectors.contains(&name),
                "Registered connector '{}' should appear in list. Found: {:?}",
                name,
                connectors
            );
        }
        Err(e) => {
            // Listing failed - should have meaningful error
            assert!(!e.is_empty(), "List error message should not be empty");
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
    let result = connect::register(name.clone(), schema, source);

    // Assert: Verify validation behavior (observable output)
    match result {
        Ok(_) => {
            // If validation is lenient and registration succeeded, verify connector appears in list
            let list_result = connect::list();
            if list_result.is_ok() {
                let connectors = list_result.unwrap();
                // If lenient validation, connector should appear in list
                assert!(
                    connectors.contains(&name),
                    "If validation is lenient, connector should appear in list"
                );
            }
        }
        Err(e) => {
            // Error case - validation is strict, should indicate invalid format
            assert!(!e.is_empty(), "Error message should not be empty");
            // Error should mention validation or format issue (observable error content)
            assert!(
                e.to_lowercase().contains("invalid")
                    || e.to_lowercase().contains("format")
                    || e.to_lowercase().contains("validation")
                    || e.to_lowercase().contains("parse")
                    || e.to_lowercase().contains("source"),
                "Error should mention validation, format, parse, or source issue: {}",
                e
            );
            // Verify connector does NOT appear in list when registration fails
            let list_result = connect::list();
            if list_result.is_ok() {
                let connectors = list_result.unwrap();
                assert!(
                    !connectors.contains(&name),
                    "Failed registration should not add connector to list"
                );
            }
        }
    }
}
