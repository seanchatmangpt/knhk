//! Storage layer for workflow engine
//!
//! Provides persistent storage and memory-mapped file access for workflows.

#[cfg(feature = "memory-v2")]
pub mod mmap;

#[cfg(feature = "memory-v2")]
pub use mmap::{MmapWorkflowStore, MmapWorkflowReader};
