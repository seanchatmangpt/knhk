//! Resource allocator implementation

use super::policies::{
    allocate_capability_based, allocate_chained, allocate_four_eyes, allocate_role_based,
    allocate_round_robin, allocate_shortest_queue, PolicyContext,
};
use super::types::{AllocationPolicy, AllocationRequest, AllocationResult, Resource, ResourceId};
use crate::error::{WorkflowError, WorkflowResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Resource allocation manager
pub struct ResourceAllocator {
    /// Policy context
    ctx: PolicyContext,
}

impl ResourceAllocator {
    /// Create new resource allocator
    pub fn new() -> Self {
        Self {
            ctx: PolicyContext {
                resources: Arc::new(RwLock::new(HashMap::new())),
                round_robin_state: Arc::new(RwLock::new(HashMap::new())),
                chained_state: Arc::new(RwLock::new(HashMap::new())),
            },
        }
    }

    /// Register a resource
    pub async fn register_resource(&self, resource: Resource) -> WorkflowResult<()> {
        let mut resources = self.ctx.resources.write().await;
        resources.insert(resource.id, resource);
        Ok(())
    }

    /// Get resource
    pub async fn get_resource(&self, resource_id: ResourceId) -> WorkflowResult<Resource> {
        let resources = self.ctx.resources.read().await;
        resources.get(&resource_id).cloned().ok_or_else(|| {
            WorkflowError::ResourceUnavailable(format!("Resource {} not found", resource_id.0))
        })
    }

    /// Allocate resources for a task
    pub async fn allocate(&self, request: AllocationRequest) -> WorkflowResult<AllocationResult> {
        match request.policy {
            AllocationPolicy::FourEyes => allocate_four_eyes(&self.ctx, &request).await,
            AllocationPolicy::Chained => allocate_chained(&self.ctx, &request).await,
            AllocationPolicy::RoundRobin => allocate_round_robin(&self.ctx, &request).await,
            AllocationPolicy::ShortestQueue => allocate_shortest_queue(&self.ctx, &request).await,
            AllocationPolicy::RoleBased => allocate_role_based(&self.ctx, &request).await,
            AllocationPolicy::CapabilityBased => {
                allocate_capability_based(&self.ctx, &request).await
            }
            AllocationPolicy::Manual => Err(WorkflowError::ResourceUnavailable(
                "Manual allocation required".to_string(),
            )),
        }
    }

    /// Update resource workload
    pub async fn update_workload(&self, resource_id: ResourceId, delta: i32) -> WorkflowResult<()> {
        let mut resources = self.ctx.resources.write().await;
        if let Some(resource) = resources.get_mut(&resource_id) {
            if delta > 0 {
                resource.workload = resource.workload.saturating_add(delta as u32);
                resource.queue_length = resource.queue_length.saturating_add(delta as u32);
            } else {
                resource.workload = resource.workload.saturating_sub((-delta) as u32);
                resource.queue_length = resource.queue_length.saturating_sub((-delta) as u32);
            }
        }
        Ok(())
    }

    /// Set resource availability
    pub async fn set_availability(
        &self,
        resource_id: ResourceId,
        available: bool,
    ) -> WorkflowResult<()> {
        let mut resources = self.ctx.resources.write().await;
        if let Some(resource) = resources.get_mut(&resource_id) {
            resource.available = available;
        }
        Ok(())
    }
}

impl Default for ResourceAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_four_eyes_allocation() {
        let allocator = ResourceAllocator::new();

        // Register two resources
        let resource1 = Resource {
            id: ResourceId::new(),
            name: "Resource 1".to_string(),
            roles: vec![crate::resource::allocation::types::Role {
                id: "approver".to_string(),
                name: "Approver".to_string(),
                capabilities: vec![],
            }],
            capabilities: vec![],
            workload: 0,
            queue_length: 0,
            available: true,
        };

        let resource2 = Resource {
            id: ResourceId::new(),
            name: "Resource 2".to_string(),
            roles: vec![crate::resource::allocation::types::Role {
                id: "approver".to_string(),
                name: "Approver".to_string(),
                capabilities: vec![],
            }],
            capabilities: vec![],
            workload: 0,
            queue_length: 0,
            available: true,
        };

        allocator
            .register_resource(resource1)
            .await
            .expect("Failed to register resource1");
        allocator
            .register_resource(resource2)
            .await
            .expect("Failed to register resource2");

        // Allocate with four-eyes policy
        let request = AllocationRequest {
            task_id: "task1".to_string(),
            spec_id: crate::parser::WorkflowSpecId::new(),
            required_roles: vec!["approver".to_string()],
            required_capabilities: vec![],
            policy: AllocationPolicy::FourEyes,
            priority: 100,
        };

        let result = allocator
            .allocate(request)
            .await
            .expect("Failed to allocate resources");
        assert_eq!(result.resource_ids.len(), 2);
        assert_eq!(result.policy, AllocationPolicy::FourEyes);
    }
}
