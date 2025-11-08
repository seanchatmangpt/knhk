//! State module - Improved state management
//!
//! Provides:
//! - State manager with event sourcing
//! - State caching
//! - State snapshots

mod store;
pub mod manager;

pub use manager::{StateEvent, StateManager};
pub use store::StateStore;

