#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Distributed state management for multi-region deployment

pub mod balancer;
pub mod distributed;
pub mod leader;
pub mod sync;

pub use balancer::{LoadBalanceStrategy, LoadBalancer};
pub use distributed::{DistributedStateStore, ReplicationConfig};
pub use leader::{LeaderElection, LeaderState};
pub use sync::{StateSync, SyncStrategy};
