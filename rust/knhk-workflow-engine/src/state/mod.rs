//! State module - Improved state management
//!
//! Provides:
//! - State manager with event sourcing
//! - State caching
//! - State snapshots

pub mod manager;
#[cfg(feature = "storage")]
mod store;

pub use manager::{StateEvent, StateManager};
#[cfg(feature = "storage")]
pub use store::StateStore;
