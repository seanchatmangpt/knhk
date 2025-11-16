//! Phase 0: Async/Await Mastery - Concurrency Primitives
//!
//! This module provides the foundational async/await infrastructure for the KNHK workflow engine:
//! - Structured concurrency with Nurseries
//! - Cancellation tokens and scopes
//! - Work-stealing executor for high-performance task scheduling
//! - Pin/Unpin utilities for safe async patterns
//!
//! # Performance Targets
//! - Task spawn latency: <100ns (P99)
//! - CPU utilization: >95% for CPU-bound workloads
//! - Scalability: Linear scaling to core count
//!
//! # Feature Gates
//! Enable with: `#[cfg(feature = "async-v2")]`

#[cfg(feature = "async-v2")]
pub mod nursery;

#[cfg(feature = "async-v2")]
pub mod cancel_token;

#[cfg(feature = "async-v2")]
pub mod work_stealing;

#[cfg(feature = "async-v2")]
pub mod pin_utils;

#[cfg(feature = "async-v2")]
pub use nursery::{Nursery, NurseryScope};

#[cfg(feature = "async-v2")]
pub use cancel_token::{CancelToken, CancelScope};

#[cfg(feature = "async-v2")]
pub use work_stealing::{WorkStealingExecutor, WorkerConfig};

#[cfg(feature = "async-v2")]
pub use pin_utils::PinExt;

/// Re-export common async traits
pub use async_trait::async_trait;
pub use std::future::Future;
pub use std::pin::Pin;
pub use tokio::task::JoinHandle;

/// Concurrency module result type
pub type ConcurrencyResult<T> = Result<T, ConcurrencyError>;

/// Concurrency-specific errors
#[derive(Debug, thiserror::Error)]
pub enum ConcurrencyError {
    #[error("Task cancelled")]
    Cancelled,

    #[error("Nursery task failed: {0}")]
    TaskFailed(String),

    #[error("Work stealing scheduler error: {0}")]
    SchedulerError(String),

    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}
