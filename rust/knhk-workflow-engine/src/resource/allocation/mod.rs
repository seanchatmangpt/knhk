//! Resource Allocation System
//!
//! Implements YAWL-style resource allocation policies:
//! - Four-eyes principle (dual approval)
//! - Chained execution (sequential assignment)
//! - Round-robin allocation
//! - Shortest queue allocation
//! - Role-based allocation
//! - Capability-based allocation

pub mod allocator;
pub mod policies;
pub mod types;

pub use allocator::ResourceAllocator;
pub use types::{
    AllocationPolicy, AllocationRequest, AllocationResult, Capability, Resource, ResourceId, Role,
};
