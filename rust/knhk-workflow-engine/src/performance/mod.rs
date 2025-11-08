#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Performance optimization for Fortune 500-level workflow engine

pub mod batching;
pub mod cache;
pub mod pooling;
pub mod routing;

pub use batching::{BatchConfig, BatchProcessor};
pub use cache::{CacheConfig, WorkflowCache};
pub use pooling::{ConnectionPool, PoolConfig};
pub use routing::{PathRouter, RoutingDecision};
