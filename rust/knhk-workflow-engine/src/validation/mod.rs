//! Workflow validation module
//!
//! Provides validation capabilities including deadlock detection,
//! workflow structure validation, and pattern validation.

pub mod deadlock;

pub use deadlock::{DeadlockDetectionResult, DeadlockDetector};
