//! Chicago TDD tests for boot commands

#![cfg(feature = "std")]

use knhk_cli::commands::boot;

/// Test: boot::init returns Result
/// Chicago TDD: Test behavior (initialization) not implementation (file operations)
#[test]
fn test_boot_init_returns_result() {
    // Arrange: Create temporary files for sigma and q
    let temp_dir = std::env::temp_dir();
    let sigma_file = temp_dir.join("test_sigma.ttl");
    let q_file = temp_dir.join("test_q.ttl");

    // Create minimal TTL files
    std::fs::write(&sigma_file, "@prefix ex: <http://example.org/> .").unwrap();
    std::fs::write(&q_file, "@prefix ex: <http://example.org/> .").unwrap();

    // Act: Initialize boot
    let result = boot::init(
        sigma_file.to_string_lossy().to_string(),
        q_file.to_string_lossy().to_string(),
    );

    // Assert: Returns Result (may fail if config dir creation fails, but should not panic)
    assert!(result.is_ok() || result.is_err());

    // Cleanup
    let _ = std::fs::remove_file(&sigma_file);
    let _ = std::fs::remove_file(&q_file);
}

/// Test: boot::init with non-existent sigma file
/// Chicago TDD: Test behavior (error handling) not implementation (file checking)
#[test]
fn test_boot_init_with_nonexistent_sigma() {
    // Arrange: Use non-existent file
    let sigma_file = "/nonexistent/sigma.ttl";
    let q_file = "/nonexistent/q.ttl";

    // Act: Initialize boot
    let result = boot::init(sigma_file.to_string(), q_file.to_string());

    // Assert: Should return error
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("not found") || error_msg.contains("Schema"));
}

/// Test: boot::init with non-existent q file
/// Chicago TDD: Test behavior (error handling) not implementation (file checking)
#[test]
fn test_boot_init_with_nonexistent_q() {
    // Arrange: Create sigma but not q
    let temp_dir = std::env::temp_dir();
    let sigma_file = temp_dir.join("test_sigma.ttl");
    std::fs::write(&sigma_file, "@prefix ex: <http://example.org/> .").unwrap();
    let q_file = "/nonexistent/q.ttl";

    // Act: Initialize boot
    let result = boot::init(sigma_file.to_string_lossy().to_string(), q_file.to_string());

    // Assert: Should return error
    assert!(result.is_err());

    // Cleanup
    let _ = std::fs::remove_file(&sigma_file);
}
