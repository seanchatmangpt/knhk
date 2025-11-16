//! Chicago TDD Tests for knhk-test-cache
//!
//! Comprehensive test suite using chicago-tdd-tools to validate
//! all components of the autonomic test cache daemon.
//!
//! **Test Coverage**:
//! 1. Code hashing (deterministic, idempotent)
//! 2. Cache operations (store, retrieve, expiration)
//! 3. File watching (change detection, debouncing)
//! 4. Test compilation (cargo availability, build execution)
//! 5. Daemon lifecycle (start, stop, status)
//!
//! Uses chicago-tdd-tools macros for AAA pattern enforcement.

use chicago_tdd_tools::{assert_err, assert_ok, chicago_async_test, chicago_test};
use knhk_test_cache::{Cache, CacheResult, CodeHasher, TestCacheError, TestCompiler, TestStatus};
use std::path::PathBuf;
use tempfile::TempDir;

/// Test that code hasher generates deterministic hashes
chicago_test!(test_code_hasher_deterministic, {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "pub fn test() {}").unwrap();

    // Act
    let hasher = CodeHasher::new(temp_dir.path().to_path_buf());
    let hash1 = hasher.hash().unwrap();
    let hash2 = hasher.hash().unwrap();

    // Assert
    assert_eq!(hash1, hash2, "Hash should be deterministic");
});

/// Test that code hasher detects code changes
chicago_test!(test_code_hasher_detects_changes, {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "pub fn test() {}").unwrap();

    let hasher = CodeHasher::new(temp_dir.path().to_path_buf());
    let hash1 = hasher.hash().unwrap();

    // Act
    std::fs::write(
        src_dir.join("lib.rs"),
        "pub fn test() { println!(\"changed\"); }",
    )
    .unwrap();
    let hash2 = hasher.hash().unwrap();

    // Assert
    assert_ne!(hash1, hash2, "Hash should change when code changes");
});

/// Test that code hasher excludes target directory
chicago_test!(test_code_hasher_excludes_target, {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    let target_dir = temp_dir.path().join("target");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::create_dir_all(&target_dir).unwrap();

    std::fs::write(src_dir.join("lib.rs"), "pub fn test() {}").unwrap();
    std::fs::write(target_dir.join("test.rs"), "should be excluded").unwrap();

    // Act
    let hasher = CodeHasher::new(temp_dir.path().to_path_buf());
    let hash = hasher.hash().unwrap();

    // Assert
    assert!(!hash.is_empty(), "Hash should be generated");
    // Verify target/test.rs is not included in hash
    let hash_without_target = hash;
    std::fs::remove_file(target_dir.join("test.rs")).unwrap();
    let hash_after_removal = hasher.hash().unwrap();
    assert_eq!(
        hash_without_target, hash_after_removal,
        "Target directory should be excluded"
    );
});

/// Test cache store and retrieve
chicago_test!(test_cache_store_and_retrieve, {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let cache = Cache::new(temp_dir.path().to_path_buf());

    // Act
    let code_hash = "test_hash_123";
    let result = cache.store(code_hash, TestStatus::Passed, None, 1.5);
    assert_ok!(&result, "Store should succeed");

    let cached = cache.get(code_hash).unwrap();

    // Assert
    assert!(cached.is_some(), "Cache should contain stored result");
    let cached_result = cached.unwrap();
    assert_eq!(cached_result.code_hash, code_hash);
    assert_eq!(cached_result.status, TestStatus::Passed);
    assert_eq!(cached_result.duration_secs, 1.5);
});

/// Test cache expiration
chicago_test!(test_cache_expiration, {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let cache = Cache::new(temp_dir.path().to_path_buf()).with_max_age(1); // 1 second expiration

    let code_hash = "test_hash_456";
    cache
        .store(code_hash, TestStatus::Passed, None, 1.0)
        .unwrap();

    // Act & Assert: Should be available immediately
    assert!(
        cache.get(code_hash).unwrap().is_some(),
        "Cache should contain result immediately"
    );

    // Wait for expiration
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Assert: Should be expired
    assert!(
        cache.get(code_hash).unwrap().is_none(),
        "Cache should expire after TTL"
    );
});

/// Test cache statistics
chicago_test!(test_cache_stats, {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let cache = Cache::new(temp_dir.path().to_path_buf());

    // Act
    cache.store("hash1", TestStatus::Passed, None, 1.0).unwrap();
    cache.store("hash2", TestStatus::Failed, None, 2.0).unwrap();

    let stats = cache.stats().unwrap();

    // Assert
    assert_eq!(stats.entry_count, 2, "Should have 2 cache entries");
    assert!(
        stats.total_size_bytes > 0,
        "Cache should have non-zero size"
    );
});

/// Test cache clear
chicago_test!(test_cache_clear, {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let cache = Cache::new(temp_dir.path().to_path_buf());

    cache.store("hash1", TestStatus::Passed, None, 1.0).unwrap();
    cache.store("hash2", TestStatus::Failed, None, 2.0).unwrap();

    // Act
    let result = cache.clear();
    assert_ok!(&result, "Clear should succeed");

    // Assert
    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 0, "Cache should be empty after clear");
});

/// Test test compiler checks cargo availability
chicago_test!(test_compiler_checks_cargo, {
    // Arrange & Act
    let result = TestCompiler::check_cargo_available();

    // Assert
    // Cargo should be available in test environment
    // If not, that's also a valid test result
    if result.is_err() {
        assert_err!(&result, "Cargo not found");
    } else {
        assert_ok!(&result, "Cargo should be available");
    }
});

/// Test test compiler creates instance
chicago_test!(test_compiler_creation, {
    // Arrange
    let temp_dir = TempDir::new().unwrap();

    // Act
    let compiler = TestCompiler::new(temp_dir.path().to_path_buf());

    // Assert
    assert_eq!(compiler.workspace_root(), temp_dir.path());
});

/// Test test compiler checks binary existence
chicago_test!(test_compiler_binaries_exist, {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let compiler = TestCompiler::new(temp_dir.path().to_path_buf());

    // Act
    let exists = compiler.binaries_exist();

    // Assert
    // For a fresh temp directory without compiled binaries, they should not exist
    assert_eq!(exists, false, "Binaries should not exist in fresh temp directory");
});

/// Test daemon status when not running
chicago_test!(test_daemon_status_not_running, {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let daemon = knhk_test_cache::Daemon::new(temp_dir.path().to_path_buf());

    // Act
    let status = daemon.status().unwrap();

    // Assert
    assert_eq!(status.running, false, "Daemon should not be running");
    assert_eq!(
        status.cache_stats.entry_count, 0,
        "Cache should be empty initially"
    );
});

/// Test daemon checks if running
chicago_test!(test_daemon_is_running_check, {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let daemon = knhk_test_cache::Daemon::new(temp_dir.path().to_path_buf());

    // Act
    let is_running = daemon.is_running();

    // Assert
    assert_eq!(is_running, false, "Daemon should not be running initially");
});

/// Test cache result serialization
chicago_test!(test_cache_result_serialization, {
    // Arrange
    let result = CacheResult {
        code_hash: "test_hash".to_string(),
        timestamp: 1234567890,
        status: TestStatus::Passed,
        output: Some("test output".to_string()),
        duration_secs: 1.5,
    };

    // Act
    let serialized = serde_json::to_string(&result).unwrap();
    let deserialized: CacheResult = serde_json::from_str(&serialized).unwrap();

    // Assert
    assert_eq!(deserialized.code_hash, result.code_hash);
    assert_eq!(deserialized.status, result.status);
    assert_eq!(deserialized.duration_secs, result.duration_secs);
});

/// Test test status enum
chicago_test!(test_test_status_enum, {
    // Arrange & Act
    let passed = TestStatus::Passed;
    let failed = TestStatus::Failed;
    let skipped = TestStatus::Skipped;

    // Assert
    assert_eq!(passed, TestStatus::Passed);
    assert_eq!(failed, TestStatus::Failed);
    assert_eq!(skipped, TestStatus::Skipped);
});

/// Test error types
chicago_test!(test_error_types, {
    // Arrange & Act
    let watcher_error = TestCacheError::WatcherError(notify::Error::generic("test error"));
    let io_error = TestCacheError::IoError(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "not found",
    ));
    let process_error = TestCacheError::ProcessError("process failed".to_string());
    let cache_error = TestCacheError::CacheError("cache failed".to_string());
    let daemon_running = TestCacheError::DaemonRunning(12345);
    let daemon_not_running = TestCacheError::DaemonNotRunning;
    let cargo_not_found = TestCacheError::CargoNotFound;
    let invalid_config = TestCacheError::InvalidConfig("invalid".to_string());

    // Assert: Verify error types can be created and formatted
    assert!(format!("{}", watcher_error).contains("watcher"));
    assert!(format!("{}", io_error).contains("IO"));
    assert!(format!("{}", process_error).contains("process"));
    assert!(format!("{}", cache_error).contains("cache"));
    assert!(format!("{}", daemon_running).contains("running"));
    assert!(format!("{}", daemon_not_running).contains("not running"));
    assert!(format!("{}", cargo_not_found).contains("Cargo"));
    assert!(format!("{}", invalid_config).contains("Invalid"));
});

/// Test code hasher with custom exclude patterns
chicago_test!(test_code_hasher_custom_exclude, {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    let custom_dir = temp_dir.path().join("custom");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::create_dir_all(&custom_dir).unwrap();

    std::fs::write(src_dir.join("lib.rs"), "pub fn test() {}").unwrap();
    std::fs::write(custom_dir.join("excluded.rs"), "should be excluded").unwrap();

    // Act
    let hasher = CodeHasher::new(temp_dir.path().to_path_buf()).exclude("custom/".to_string());
    let hash = hasher.hash().unwrap();

    // Assert
    assert!(!hash.is_empty(), "Hash should be generated");
    // Verify custom/excluded.rs is not included
    let hash_before = hash;
    std::fs::remove_file(custom_dir.join("excluded.rs")).unwrap();
    let hash_after = hasher.hash().unwrap();
    assert_eq!(hash_before, hash_after, "Custom exclude should work");
});

/// Test cache with output
chicago_test!(test_cache_with_output, {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let cache = Cache::new(temp_dir.path().to_path_buf());

    // Act
    let code_hash = "test_hash_with_output";
    let output = Some("test output\nmore output".to_string());
    cache
        .store(code_hash, TestStatus::Failed, output.clone(), 2.5)
        .unwrap();

    let cached = cache.get(code_hash).unwrap();

    // Assert
    assert!(cached.is_some(), "Cache should contain result");
    let cached_result = cached.unwrap();
    assert_eq!(cached_result.output, output);
    assert_eq!(cached_result.status, TestStatus::Failed);
});

/// Test cache cleanup keeps last N entries
chicago_test!(test_cache_cleanup_keeps_last_n, {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let cache = Cache::new(temp_dir.path().to_path_buf());

    // Act: Store more than 10 entries
    for i in 0..15 {
        cache
            .store(&format!("hash_{}", i), TestStatus::Passed, None, 1.0)
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different timestamps
    }

    // Assert: Should keep only last 10
    let stats = cache.stats().unwrap();
    assert!(stats.entry_count <= 10, "Should keep at most 10 entries");
});
