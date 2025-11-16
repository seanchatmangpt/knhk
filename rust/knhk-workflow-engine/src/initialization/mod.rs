//! Phase 2: Lazy Initialization Module
//!
//! Provides safe lazy initialization primitives using OnceLock.

#[cfg(feature = "memory-v2")]
pub mod once_lock;

#[cfg(feature = "memory-v2")]
pub use once_lock::{PatternRegistry, GlobalResourceRegistry};
