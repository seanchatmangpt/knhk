//! Test Fixtures
//!
//! Provides reusable test fixtures with automatic cleanup and state management.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

/// Test fixture error
#[derive(Error, Debug)]
pub enum FixtureError {
    /// Failed to create fixture
    #[error("Failed to create fixture: {0}")]
    CreationFailed(String),
    /// Fixture operation failed
    #[error("Fixture operation failed: {0}")]
    OperationFailed(String),
}

/// Result type for fixture operations
pub type FixtureResult<T> = Result<T, FixtureError>;

/// Generic test fixture
///
/// Provides a base fixture structure that can be extended for specific use cases.
pub struct TestFixture {
    /// Unique test counter for isolation
    test_counter: u64,
    /// Test metadata
    metadata: HashMap<String, String>,
}

impl TestFixture {
    /// Create a new test fixture with unique identifier
    pub fn new() -> FixtureResult<Self> {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);

        Ok(Self {
            test_counter: counter,
            metadata: HashMap::new(),
        })
    }

    /// Get test counter
    pub fn test_counter(&self) -> u64 {
        self.test_counter
    }

    /// Set metadata
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Cleanup fixture resources
    pub fn cleanup(&self) -> FixtureResult<()> {
        // Override in specific implementations
        Ok(())
    }
}

impl Default for TestFixture {
    fn default() -> Self {
        // Default implementation should not fail - use unwrap_or_else with panic
        Self::new().unwrap_or_else(|e| panic!("Failed to create default fixture: {}", e))
    }
}
