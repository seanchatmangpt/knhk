//! Bounds enforcement tests
//!
//! These tests enforce the Chatman Constant across all critical operations.

mod hot_path;
mod warm_path;
mod cold_path;
mod regression;

pub use hot_path::*;
pub use warm_path::*;
pub use cold_path::*;
pub use regression::*;
