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

    /// Pre-bind resources for a workflow specification (hot-path allocation)
    ///
    /// This method performs declarative resource allocation before case activation,
    /// enabling hot-path allocation for enterprise YAWL workflows. It analyzes
    /// the workflow specification and pre-allocates resources based on roles,
    /// capabilities, and organizational ontology.
    pub async fn allocate_prebound(
        &self,
        spec: &crate::parser::WorkflowSpec,
    ) -> WorkflowResult<HashMap<String, ResourceId>> {
        let mut prebound = HashMap::new();
        let resources = self.ctx.resources.read().await;

        // Analyze each task in the workflow specification
        for (task_id, task) in &spec.tasks {
            // Build allocation request from task requirements
            let request = AllocationRequest {
                task_id: task_id.clone(),
                spec_id: spec.id,
                required_roles: task.required_roles.clone(),
                required_capabilities: task.required_capabilities.clone(),
                policy: task
                    .allocation_policy
                    .unwrap_or(AllocationPolicy::RoundRobin),
                priority: task.priority.map(|p| p.min(255) as u8).unwrap_or(100u8),
            };

            // Find matching resources based on roles and capabilities
            let matching_resources: Vec<&Resource> = resources
                .values()
                .filter(|resource| {
                    resource.available
                        && (request.required_roles.is_empty()
                            || resource
                                .roles
                                .iter()
                                .any(|role| request.required_roles.contains(&role.id)))
                        && (request.required_capabilities.is_empty()
                            || resource
                                .capabilities
                                .iter()
                                .any(|cap| request.required_capabilities.contains(&cap.id)))
                })
                .collect();

            // Select resource based on policy (simplified - full policy logic in allocate())
            if let Some(resource) = matching_resources.first() {
                prebound.insert(task_id.clone(), resource.id);
            }
        }

        Ok(prebound)
    }

    /// 3-phase resource allocation (offer, allocate, start)
    ///
    /// Phase 1: Offer - Propose resources to task
    /// Phase 2: Allocate - Confirm resource allocation
    /// Phase 3: Start - Activate resource allocation
    pub async fn allocate_3phase(
        &self,
        request: AllocationRequest,
    ) -> WorkflowResult<AllocationResult> {
        // Phase 1: Offer - Find matching resources
        let offer_result = self.allocate(request.clone()).await?;

        // Phase 2: Allocate - Confirm allocation (same as offer for now)
        // In production, would validate resource availability at this point
        let allocate_result = offer_result.clone();

        // Phase 3: Start - Activate allocation
        // Update resource workload
        for resource_id in &allocate_result.resource_ids {
            self.update_workload(*resource_id, 1).await?;
        }

        Ok(allocate_result)
    }

    /// Phase 1: Offer resources (proposal)
    pub async fn offer_resources(
        &self,
        request: AllocationRequest,
    ) -> WorkflowResult<AllocationResult> {
        self.allocate(request).await
    }

    /// Phase 2: Allocate resources (confirmation)
    pub async fn confirm_allocation(
        &self,
        request: AllocationRequest,
    ) -> WorkflowResult<AllocationResult> {
        // Validate resources are still available
        let result = self.allocate(request.clone()).await?;

        // Check resource availability
        let resources = self.ctx.resources.read().await;
        for resource_id in &result.resource_ids {
            if let Some(resource) = resources.get(resource_id) {
                if !resource.available {
                    return Err(crate::error::WorkflowError::ResourceUnavailable(format!(
                        "Resource {} is not available",
                        resource_id.0
                    )));
                }
            }
        }

        Ok(result)
    }

    /// Phase 3: Start resource allocation (activation)
    pub async fn start_allocation(&self, resource_ids: Vec<ResourceId>) -> WorkflowResult<()> {
        // Update resource workload
        for resource_id in resource_ids {
            self.update_workload(resource_id, 1).await?;
        }
        Ok(())
    }

    /// Release resources (deallocation)
    pub async fn release_resources(&self, resource_ids: Vec<ResourceId>) -> WorkflowResult<()> {
        // Decrease resource workload
        for resource_id in resource_ids {
            self.update_workload(resource_id, -1).await?;
        }
        Ok(())
    }

    /// Get available resources with filters
    pub async fn get_available_resources(
        &self,
        required_roles: Vec<String>,
        required_capabilities: Vec<String>,
        max_workload: Option<u32>,
    ) -> WorkflowResult<Vec<Resource>> {
        let resources = self.ctx.resources.read().await;
        let mut available = Vec::new();

        for resource in resources.values() {
            if !resource.available {
                continue;
            }

            // Check role requirements
            if !required_roles.is_empty() {
                let has_role = resource
                    .roles
                    .iter()
                    .any(|role| required_roles.contains(&role.id));
                if !has_role {
                    continue;
                }
            }

            // Check capability requirements
            if !required_capabilities.is_empty() {
                let has_capability = resource
                    .capabilities
                    .iter()
                    .any(|cap| required_capabilities.contains(&cap.id));
                if !has_capability {
                    continue;
                }
            }

            // Check workload constraint
            if let Some(max) = max_workload {
                if resource.workload >= max {
                    continue;
                }
            }

            available.push(resource.clone());
        }

        Ok(available)
    }

    /// Get resources by constraints
    pub async fn get_resources_by_constraints(
        &self,
        constraints: serde_json::Value,
    ) -> WorkflowResult<Vec<Resource>> {
        let resources = self.ctx.resources.read().await;
        let mut matching = Vec::new();

        for resource in resources.values() {
            let mut matches = true;

            // Check constraints
            if let Some(constraints_obj) = constraints.as_object() {
                for (key, value) in constraints_obj {
                    match key.as_str() {
                        "available" => {
                            if let Some(avail) = value.as_bool() {
                                if resource.available != avail {
                                    matches = false;
                                    break;
                                }
                            }
                        }
                        "max_workload" => {
                            if let Some(max) = value.as_u64() {
                                if resource.workload >= max as u32 {
                                    matches = false;
                                    break;
                                }
                            }
                        }
                        "min_capability_level" => {
                            if let Some(min_level) = value.as_u64() {
                                let has_capability = resource
                                    .capabilities
                                    .iter()
                                    .any(|cap| cap.level >= min_level as u8);
                                if !has_capability {
                                    matches = false;
                                    break;
                                }
                            }
                        }
                        _ => {
                            // Unknown constraint - skip
                        }
                    }
                }
            }

            if matches {
                matching.push(resource.clone());
            }
        }

        Ok(matching)
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
