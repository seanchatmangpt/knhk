//! YAWL Resource Allocation
//!
//! Implements YAWL resource allocation policies with TRIZ Principle 40: Composite Materials
//!
//! Based on: org.yawlfoundation.yawl.resourcing

use crate::error::{WorkflowError, WorkflowResult};
use crate::resource::allocation::types::ResourceId;
use crate::resourcing::three_phase::AllocationContext;

/// Allocation policy (TRIZ Principle 40: Composite Materials)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationPolicy {
    /// Round robin allocation
    RoundRobin,
    /// Random allocation
    Random,
    /// Shortest queue allocation
    ShortestQueue,
    /// Least busy allocation
    LeastBusy,
    /// Fastest completion history
    FastestCompletion,
}

/// Allocation result
#[derive(Debug, Clone)]
pub struct AllocationResult {
    /// Selected resource ID
    pub resource_id: ResourceId,
    /// Allocation method used
    pub method: AllocationPolicy,
}

/// Resource allocator
pub struct ResourceAllocator {
    policy: AllocationPolicy,
}

impl ResourceAllocator {
    pub fn new(policy: AllocationPolicy) -> Self {
        Self { policy }
    }

    pub async fn allocate(
        &self,
        eligible_resources: &[ResourceId],
        _context: &AllocationContext,
    ) -> WorkflowResult<AllocationResult> {
        if eligible_resources.is_empty() {
            return Err(WorkflowError::ResourceUnavailable(
                "No eligible resources".to_string(),
            ));
        }

        let selected = match self.policy {
            AllocationPolicy::RoundRobin => {
                // Simple round-robin (would need state in real implementation)
                eligible_resources[0].clone()
            }
            AllocationPolicy::Random => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..eligible_resources.len());
                eligible_resources[index].clone()
            }
            AllocationPolicy::ShortestQueue => {
                // Would need queue length data
                eligible_resources[0].clone()
            }
            AllocationPolicy::LeastBusy => {
                // Would need busy status data
                eligible_resources[0].clone()
            }
            AllocationPolicy::FastestCompletion => {
                // Would need completion history
                eligible_resources[0].clone()
            }
        };

        Ok(AllocationResult {
            resource_id: selected,
            method: self.policy,
        })
    }
}

