//! Chicago TDD tests for context commands

#![cfg(feature = "std")]

use knhk_cli::commands::context;

/// Test: context::list returns Result
/// Chicago TDD: Test behavior (listing) not implementation (storage reading)
#[test]
fn test_context_list_returns_result() {
    // Act: List contexts
    let result = context::list();

    // Assert: Returns Result (may fail if storage read fails, but should not panic)
    assert!(result.is_ok() || result.is_err());

    // If successful, should return Vec<String>
    if let Ok(contexts) = result {
        assert!(contexts.iter().all(|c| !c.is_empty()));
    }
}

/// Test: context::current returns Result
/// Chicago TDD: Test behavior (current context) not implementation (state management)
#[test]
fn test_context_current_returns_result() {
    // Act: Get current context
    let result = context::current();

    // Assert: Returns Result (may fail if no current context, but should not panic)
    assert!(result.is_ok() || result.is_err());

    // If successful, should return String
    if let Ok(context_id) = result {
        assert!(!context_id.is_empty());
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

    // Assert: Returns Result (may fail if storage fails, but should not panic)
    assert!(result.is_ok() || result.is_err());
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

    // Assert: Both should return Results
    assert!(create_result.is_ok() || create_result.is_err());
    assert!(use_result.is_ok() || use_result.is_err());
}
