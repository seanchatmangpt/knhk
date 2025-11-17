//! YAWL Resource Management Implementation
//!
//! Implements YAWL resource allocation with TRIZ hyper-advanced patterns:
//! - TRIZ Principle 40: Composite Materials - Multiple allocation strategies
//! - TRIZ Principle 35: Parameter Changes - Dynamic allocation parameters
//!
//! Based on: org.yawlfoundation.yawl.resourcing

pub mod allocation;
pub mod constraints;
pub mod filters;
pub mod resource_pool_impl;
pub mod three_phase;

pub use allocation::{AllocationPolicy, AllocationResult, ResourceAllocator};
pub use constraints::{
    Constraint, ConstraintResult, ConstraintType, FourEyesPrinciple, SeparationOfDuties,
};
pub use filters::{FilterResult, FilterType, ResourceFilter};
pub use resource_pool_impl::ResourcePoolWrapper;
pub use three_phase::{AllocationPhase, ResourcePool as ResourcePoolTrait, ThreePhaseAllocator};
