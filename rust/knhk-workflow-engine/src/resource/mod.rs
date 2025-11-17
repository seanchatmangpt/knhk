//! Resource management module
//!
//! Provides resource allocation, pooling, and lifecycle management.
//!
//! # YAWL Resource Management
//!
//! - `yawl_resource.rs`: YAWL 3-phase allocation system with TRIZ enhancements
//! - `compliance.rs`: Compliance constraints (SOD, 4-eyes) with hyper-advanced patterns

pub mod allocation;
pub mod compliance;
mod pool;
pub mod query;
pub mod three_phase;
pub mod yawl_resource;

pub use allocation::{
    AllocationPolicy, AllocationRequest, AllocationResult, Capability, Resource, ResourceAllocator,
    ResourceId, Role,
};
pub use compliance::{
    ApproverHistory, ComplianceManager, ConstraintType, ConstraintViolation, FourEyes,
    ResourceHistory, SeparationOfDuties,
};
pub use pool::*;
pub use query::{
    CompositeQuery, FilterQuery, OptimizedQuery, QueryBuilder, QueryCompositeOperator,
    ResourceQuery,
};
pub use three_phase::{
    AllocatePhase, AllocationStrategy, CompositeOperator, OfferCriteria, OfferPhase,
    ResourceMetadata, StartPhase, ThreePhaseAllocation, ThreePhaseAllocator,
};
pub use yawl_resource::{
    AllocationAlgorithm, AllocationContext, AllocationResult as YawlAllocationResult,
    FilterContext, LaunchMode, ResourceFilter, YawlResourceManager,
};
