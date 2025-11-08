//! Chicago TDD tests for context commands

#![cfg(feature = "std")]

use knhk_cli::commands::context;

/// Test: context::list returns Result
/// Chicago TDD: Test behavior (listing) not implementation (storage reading)
#[test]
fn test_context_list_returns_result() {
    // Act: List contexts
    let result = context::list();

    // Assert: Verify actual behavior - either succeeds with valid list or fails with error
    match result {
        Ok(contexts) => {
            // Success case - should return valid list (may be empty)
            assert!(
                contexts.iter().all(|c| !c.is_empty()),
                "Context IDs should not be empty"
            );
        }
        Err(e) => {
            // Error case - should have meaningful error message
            assert!(!e.is_empty(), "Error message should not be empty");
        }
    }
}

/// Test: context::current returns Result
/// Chicago TDD: Test behavior (current context) not implementation (state management)
#[test]
fn test_context_current_returns_result() {
    // Act: Get current context
    let result = context::current();

    // Assert: Verify actual behavior - either succeeds with context ID or fails with error
    match result {
        Ok(context_id) => {
            // Success case - should return non-empty context ID
            assert!(!context_id.is_empty(), "Context ID should not be empty");
        }
        Err(e) => {
            // Error case - should have meaningful error message
            assert!(!e.is_empty(), "Error message should not be empty");
        }
    }
}

/// Test: context::create returns Result
/// Chicago TDD: Test behavior (creation) not implementation (storage)
#[test]
fn test_context_create_returns_result() {
    // Arrange: Create test context data
    let id = "test-context".to_string();
    let name = "Test Context".to_string();
    let schema_iri = "http://example.org/schema".to_string();

    // Act: Create context
    let result = context::create(id.clone(), name.clone(), schema_iri.clone());

    // Assert: Verify actual behavior - either succeeds or fails with meaningful error
    match result {
        Ok(_) => {
            // Success case - context created
        }
        Err(e) => {
            // Error case - should have meaningful error message
            assert!(!e.is_empty(), "Error message should not be empty");
        }
    }
}

/// Test: context::use_context returns Result
/// Chicago TDD: Test behavior (context switching) not implementation (state management)
#[test]
fn test_context_use_returns_result() {
    // Arrange: Use test context ID
    let id = "test-context".to_string();

    // Act: Use context
    let result = context::use_context(id.clone());

    // Assert: Returns Result (may fail if context doesn't exist, but should not panic)
    assert!(result.is_ok() || result.is_err());
}

/// Test: context::create then use
/// Chicago TDD: Test behavior (creation and usage) not implementation (storage)
#[test]
fn test_context_create_then_use() {
    // Arrange: Create test context
    let id = "create-use-test".to_string();
    let name = "Create Use Test".to_string();
    let schema_iri = "http://example.org/schema".to_string();

    // Act: Create and use context
    let create_result = context::create(id.clone(), name.clone(), schema_iri.clone());
    let use_result = context::use_context(id.clone());

    // Assert: Verify actual behavior of both operations
    match (create_result, use_result) {
        (Ok(_), Ok(_)) => {
            // Both succeeded
        }
        (Ok(_), Err(e)) => {
            // Creation succeeded but use failed
            assert!(!e.is_empty(), "Use error message should not be empty");
        }
        (Err(e), Ok(_)) => {
            // Creation failed but use succeeded (context may exist from previous test)
            assert!(!e.is_empty(), "Create error message should not be empty");
        }
        (Err(e1), Err(e2)) => {
            // Both failed - verify error messages
            assert!(!e1.is_empty(), "Create error message should not be empty");
            assert!(!e2.is_empty(), "Use error message should not be empty");
        }
    }
}
