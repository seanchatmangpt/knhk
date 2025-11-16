//! Multiple Instance pattern support modules
//!
//! Provides instance tracking, synchronization, and executor integration
//! for Patterns 12-15 (Multiple Instance patterns).

pub mod instance_tracker;
pub mod sync_gate;
pub mod executor_integration;

pub use instance_tracker::{InstanceTracker, InstanceMetadata, InstanceStatus};
pub use sync_gate::{SyncGate, SyncGateStatus};
pub use executor_integration::{spawn_instances, InstanceExecutor};
