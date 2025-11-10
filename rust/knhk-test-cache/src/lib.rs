//! KNHK Test Cache Daemon
//!
//! Autonomic file watcher that keeps test binaries pre-compiled and ready.
//! Monitors code changes and automatically rebuilds test binaries in background.
//!
//! **DFLSS Alignment**:
//! - **80/20 Focus**: Critical path optimization (test binary pre-compilation)
//! - **Production-Ready**: No placeholders, real implementations
//! - **Autonomic**: Self-governing system that maintains invariants continuously
//! - **Performance**: ≤5 second SLA for test execution
//!
//! **Architecture**:
//! - File watching: Monitors `.rs` files for changes
//! - Code hashing: SHA-256 hash of all source files
//! - Test binary pre-compilation: `cargo build --tests` in background
//! - Result caching: Cache test results by code hash
//! - Daemon management: PID file, graceful shutdown
//!
//! **Usage**:
//! ```bash
//! # Start daemon
//! knhk-test-cache start
//!
//! # Stop daemon
//! knhk-test-cache stop
//!
//! # Check status
//! knhk-test-cache status
//!
//! # Force rebuild
//! knhk-test-cache rebuild
//! ```

pub mod cache;
pub mod compiler;
pub mod daemon;
pub mod hasher;
pub mod watcher;

pub use cache::{Cache, CacheResult, CacheStats, TestStatus};
pub use compiler::TestCompiler;
pub use daemon::{Daemon, DaemonStatus};
pub use hasher::CodeHasher;
pub use watcher::FileWatcher;

/// Error types for test cache daemon
#[derive(thiserror::Error, Debug)]
pub enum TestCacheError {
    #[error("File watcher error: {0}")]
    WatcherError(#[from] notify::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Process execution error: {0}")]
    ProcessError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Daemon already running (PID: {0})")]
    DaemonRunning(u32),

    #[error("Daemon not running")]
    DaemonNotRunning,

    #[error("Cargo not found in PATH")]
    CargoNotFound,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Result type for test cache operations
pub type TestCacheResult<T> = Result<T, TestCacheError>;
