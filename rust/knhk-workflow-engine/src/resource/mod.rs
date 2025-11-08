//! Resource management module
//!
//! Provides resource allocation, task assignment, and resource management
//! capabilities matching YAWL's resource allocation features.

pub mod allocation;

pub use allocation::{
    AllocationPolicy, AllocationRequest, AllocationResult, Capability, Resource, ResourceAllocator,
    ResourceId, Role,
};
