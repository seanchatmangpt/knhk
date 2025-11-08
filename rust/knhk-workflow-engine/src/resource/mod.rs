//! Resource management module
//!
//! Provides resource allocation, pooling, and lifecycle management.

pub mod allocation;
mod pool;

pub use allocation::{
    AllocationPolicy, AllocationRequest, AllocationResult, Capability, Resource, ResourceAllocator,
    ResourceId, Role,
};
pub use pool::*;
