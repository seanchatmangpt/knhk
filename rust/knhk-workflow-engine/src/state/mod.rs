//! State module - Improved state management
//!
//! Provides:
//! - State manager with event sourcing
//! - State caching
//! - State snapshots

pub mod manager;
mod store;

pub use manager::{StateEvent, StateManager};
pub use store::StateStore;
