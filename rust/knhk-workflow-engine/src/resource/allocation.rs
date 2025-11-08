#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Resource Allocation System
//!
//! Implements YAWL-style resource allocation policies:
//! - Four-eyes principle (dual approval)
//! - Chained execution (sequential assignment)
//! - Round-robin allocation
//! - Shortest queue allocation
//! - Role-based allocation
//! - Capability-based allocation

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpecId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Resource identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResourceId(#[serde(with = "uuid::serde::compact")] pub Uuid);

impl ResourceId {
    /// Create new resource ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ResourceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource role
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Role {
    /// Role identifier
    pub id: String,
    /// Role name
    pub name: String,
    /// Capabilities required for this role
    pub capabilities: Vec<String>,
}

/// Resource capabilities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Capability {
    /// Capability identifier
    pub id: String,
    /// Capability name
    pub name: String,
    /// Capability level (0-100)
    pub level: u8,
}

/// Resource (user, system, agent)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource identifier
    pub id: ResourceId,
    /// Resource name
    pub name: String,
    /// Assigned roles
    pub roles: Vec<Role>,
    /// Capabilities
    pub capabilities: Vec<Capability>,
    /// Current workload (number of active tasks)
    pub workload: u32,
    /// Queue length
    pub queue_length: u32,
    /// Available (can accept new tasks)
    pub available: bool,
}

/// Allocation policy type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllocationPolicy {
    /// Four-eyes principle: requires two resources to approve
    FourEyes,
    /// Chained execution: sequential assignment to resources
    Chained,
    /// Round-robin: distribute tasks evenly
    RoundRobin,
    /// Shortest queue: assign to resource with shortest queue
    ShortestQueue,
    /// Role-based: assign based on role requirements
    RoleBased,
    /// Capability-based: assign based on capability requirements
    CapabilityBased,
    /// Manual: manual assignment required
    Manual,
}

/// Task allocation request
#[derive(Debug, Clone)]
pub struct AllocationRequest {
    /// Task identifier
    pub task_id: String,
    /// Workflow specification ID
    pub spec_id: WorkflowSpecId,
    /// Required roles
    pub required_roles: Vec<String>,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Allocation policy
    pub policy: AllocationPolicy,
    /// Priority (0-255, higher = more urgent)
    pub priority: u8,
}

/// Task allocation result
#[derive(Debug, Clone)]
pub struct AllocationResult {
    /// Allocated resource IDs
    pub resource_ids: Vec<ResourceId>,
    /// Allocation timestamp
    pub allocated_at: chrono::DateTime<chrono::Utc>,
    /// Allocation policy used
    pub policy: AllocationPolicy,
}

/// Resource allocation manager
pub struct ResourceAllocator {
    /// Registered resources
    resources: Arc<RwLock<HashMap<ResourceId, Resource>>>,
    /// Round-robin state
    round_robin_state: Arc<RwLock<HashMap<String, VecDeque<ResourceId>>>>,
    /// Chained execution state
    chained_state: Arc<RwLock<HashMap<String, VecDeque<ResourceId>>>>,
}

impl ResourceAllocator {
    /// Create new resource allocator
    pub fn new() -> Self {
        Self {
            resources: Arc::new(RwLock::new(HashMap::new())),
            round_robin_state: Arc::new(RwLock::new(HashMap::new())),
            chained_state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a resource
    pub async fn register_resource(&self, resource: Resource) -> WorkflowResult<()> {
        let mut resources = self.resources.write().await;
        resources.insert(resource.id, resource);
        Ok(())
    }

    /// Get resource
    pub async fn get_resource(&self, resource_id: ResourceId) -> WorkflowResult<Resource> {
        let resources = self.resources.read().await;
        resources.get(&resource_id).cloned().ok_or_else(|| {
            WorkflowError::ResourceUnavailable(format!("Resource {} not found", resource_id.0))
        })
    }

    /// Allocate resources for a task
    pub async fn allocate(&self, request: AllocationRequest) -> WorkflowResult<AllocationResult> {
        match request.policy {
            AllocationPolicy::FourEyes => self.allocate_four_eyes(&request).await,
            AllocationPolicy::Chained => self.allocate_chained(&request).await,
            AllocationPolicy::RoundRobin => self.allocate_round_robin(&request).await,
            AllocationPolicy::ShortestQueue => self.allocate_shortest_queue(&request).await,
            AllocationPolicy::RoleBased => self.allocate_role_based(&request).await,
            AllocationPolicy::CapabilityBased => self.allocate_capability_based(&request).await,
            AllocationPolicy::Manual => Err(WorkflowError::ResourceUnavailable(
                "Manual allocation required".to_string(),
            )),
        }
    }

    /// Four-eyes principle: allocate two resources for approval
    async fn allocate_four_eyes(
        &self,
        request: &AllocationRequest,
    ) -> WorkflowResult<AllocationResult> {
        let resources = self.resources.read().await;
        let mut candidates: Vec<ResourceId> = resources
            .values()
            .filter(|r| {
                r.available
                    && self.matches_roles(r, &request.required_roles)
                    && self.matches_capabilities(r, &request.required_capabilities)
            })
            .map(|r| r.id)
            .collect();

        if candidates.len() < 2 {
            return Err(WorkflowError::ResourceUnavailable(
                "Insufficient resources for four-eyes allocation (need at least 2)".to_string(),
            ));
        }

        // Select two different resources
        let resource1 = candidates[0];
        let resource2 = candidates
            .iter()
            .find(|&&id| id != resource1)
            .copied()
            .ok_or_else(|| {
                WorkflowError::ResourceUnavailable(
                    "Cannot find second resource for four-eyes".to_string(),
                )
            })?;

        Ok(AllocationResult {
            resource_ids: vec![resource1, resource2],
            allocated_at: chrono::Utc::now(),
            policy: AllocationPolicy::FourEyes,
        })
    }

    /// Chained execution: sequential assignment
    async fn allocate_chained(
        &self,
        request: &AllocationRequest,
    ) -> WorkflowResult<AllocationResult> {
        let mut chained_state = self.chained_state.write().await;
        let chain_key = format!("{}:{}", request.spec_id, request.task_id);
        let chain = chained_state
            .entry(chain_key.clone())
            .or_insert_with(VecDeque::new);

        let resources = self.resources.read().await;
        let candidates: Vec<ResourceId> = resources
            .values()
            .filter(|r| {
                r.available
                    && self.matches_roles(r, &request.required_roles)
                    && self.matches_capabilities(r, &request.required_capabilities)
            })
            .map(|r| r.id)
            .collect();

        if candidates.is_empty() {
            return Err(WorkflowError::ResourceUnavailable(
                "No available resources for chained allocation".to_string(),
            ));
        }

        // Get next resource in chain, or start new chain
        let resource_id = if let Some(&next_id) = chain.front() {
            if candidates.contains(&next_id) {
                chain.pop_front();
                next_id
            } else {
                candidates[0]
            }
        } else {
            candidates[0]
        };

        // Add to end of chain for next execution
        chain.push_back(resource_id);

        Ok(AllocationResult {
            resource_ids: vec![resource_id],
            allocated_at: chrono::Utc::now(),
            policy: AllocationPolicy::Chained,
        })
    }

    /// Round-robin allocation
    async fn allocate_round_robin(
        &self,
        request: &AllocationRequest,
    ) -> WorkflowResult<AllocationResult> {
        let mut round_robin_state = self.round_robin_state.write().await;
        let key = format!("{}:{}", request.spec_id, request.task_id);
        let queue = round_robin_state
            .entry(key.clone())
            .or_insert_with(VecDeque::new);

        let resources = self.resources.read().await;
        let candidates: Vec<ResourceId> = resources
            .values()
            .filter(|r| {
                r.available
                    && self.matches_roles(r, &request.required_roles)
                    && self.matches_capabilities(r, &request.required_capabilities)
            })
            .map(|r| r.id)
            .collect();

        if candidates.is_empty() {
            return Err(WorkflowError::ResourceUnavailable(
                "No available resources for round-robin allocation".to_string(),
            ));
        }

        // Initialize queue if empty
        if queue.is_empty() {
            for &id in &candidates {
                queue.push_back(id);
            }
        }

        // Get next resource from queue
        let resource_id = queue.pop_front().ok_or_else(|| {
            WorkflowError::ResourceUnavailable("Round-robin queue empty".to_string())
        })?;

        // Add back to end of queue
        queue.push_back(resource_id);

        Ok(AllocationResult {
            resource_ids: vec![resource_id],
            allocated_at: chrono::Utc::now(),
            policy: AllocationPolicy::RoundRobin,
        })
    }

    /// Shortest queue allocation
    async fn allocate_shortest_queue(
        &self,
        request: &AllocationRequest,
    ) -> WorkflowResult<AllocationResult> {
        let resources = self.resources.read().await;
        let mut candidates: Vec<(ResourceId, u32)> = resources
            .values()
            .filter(|r| {
                r.available
                    && self.matches_roles(r, &request.required_roles)
                    && self.matches_capabilities(r, &request.required_capabilities)
            })
            .map(|r| (r.id, r.queue_length))
            .collect();

        if candidates.is_empty() {
            return Err(WorkflowError::ResourceUnavailable(
                "No available resources for shortest queue allocation".to_string(),
            ));
        }

        // Sort by queue length (ascending)
        candidates.sort_by_key(|(_, length)| *length);

        let resource_id = candidates[0].0;

        Ok(AllocationResult {
            resource_ids: vec![resource_id],
            allocated_at: chrono::Utc::now(),
            policy: AllocationPolicy::ShortestQueue,
        })
    }

    /// Role-based allocation
    async fn allocate_role_based(
        &self,
        request: &AllocationRequest,
    ) -> WorkflowResult<AllocationResult> {
        let resources = self.resources.read().await;
        let candidates: Vec<ResourceId> = resources
            .values()
            .filter(|r| r.available && self.matches_roles(r, &request.required_roles))
            .map(|r| r.id)
            .collect();

        if candidates.is_empty() {
            return Err(WorkflowError::ResourceUnavailable(
                "No resources with required roles".to_string(),
            ));
        }

        // Select first matching resource
        let resource_id = candidates[0];

        Ok(AllocationResult {
            resource_ids: vec![resource_id],
            allocated_at: chrono::Utc::now(),
            policy: AllocationPolicy::RoleBased,
        })
    }

    /// Capability-based allocation
    async fn allocate_capability_based(
        &self,
        request: &AllocationRequest,
    ) -> WorkflowResult<AllocationResult> {
        let resources = self.resources.read().await;
        let mut candidates: Vec<(ResourceId, u8)> = resources
            .values()
            .filter(|r| r.available && self.matches_capabilities(r, &request.required_capabilities))
            .map(|r| {
                // Calculate capability score (sum of matching capability levels)
                let score: u8 = r
                    .capabilities
                    .iter()
                    .filter(|c| request.required_capabilities.contains(&c.id))
                    .map(|c| c.level)
                    .sum::<u8>()
                    .min(255);
                (r.id, score)
            })
            .collect();

        if candidates.is_empty() {
            return Err(WorkflowError::ResourceUnavailable(
                "No resources with required capabilities".to_string(),
            ));
        }

        // Sort by capability score (descending)
        candidates.sort_by_key(|(_, score)| std::cmp::Reverse(*score));

        let resource_id = candidates[0].0;

        Ok(AllocationResult {
            resource_ids: vec![resource_id],
            allocated_at: chrono::Utc::now(),
            policy: AllocationPolicy::CapabilityBased,
        })
    }

    /// Check if resource matches required roles
    fn matches_roles(&self, resource: &Resource, required_roles: &[String]) -> bool {
        if required_roles.is_empty() {
            return true;
        }
        required_roles
            .iter()
            .any(|role_id| resource.roles.iter().any(|r| r.id == *role_id))
    }

    /// Check if resource matches required capabilities
    fn matches_capabilities(&self, resource: &Resource, required_capabilities: &[String]) -> bool {
        if required_capabilities.is_empty() {
            return true;
        }
        required_capabilities
            .iter()
            .all(|cap_id| resource.capabilities.iter().any(|c| c.id == *cap_id))
    }

    /// Update resource workload
    pub async fn update_workload(&self, resource_id: ResourceId, delta: i32) -> WorkflowResult<()> {
        let mut resources = self.resources.write().await;
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
        let mut resources = self.resources.write().await;
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
            roles: vec![Role {
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
            roles: vec![Role {
                id: "approver".to_string(),
                name: "Approver".to_string(),
                capabilities: vec![],
            }],
            capabilities: vec![],
            workload: 0,
            queue_length: 0,
            available: true,
        };

        allocator.register_resource(resource1).await.unwrap();
        allocator.register_resource(resource2).await.unwrap();

        // Request four-eyes allocation
        let request = AllocationRequest {
            task_id: "task1".to_string(),
            spec_id: WorkflowSpecId::new(),
            required_roles: vec!["approver".to_string()],
            required_capabilities: vec![],
            policy: AllocationPolicy::FourEyes,
            priority: 100,
        };

        let result = allocator.allocate(request).await.unwrap();
        assert_eq!(result.resource_ids.len(), 2);
        assert_eq!(result.policy, AllocationPolicy::FourEyes);
    }

    #[tokio::test]
    async fn test_round_robin_allocation() {
        let allocator = ResourceAllocator::new();

        // Register resources
        for i in 0..3 {
            let resource = Resource {
                id: ResourceId::new(),
                name: format!("Resource {}", i),
                roles: vec![],
                capabilities: vec![],
                workload: 0,
                queue_length: 0,
                available: true,
            };
            allocator.register_resource(resource).await.unwrap();
        }

        // Request round-robin allocation
        let request = AllocationRequest {
            task_id: "task1".to_string(),
            spec_id: WorkflowSpecId::new(),
            required_roles: vec![],
            required_capabilities: vec![],
            policy: AllocationPolicy::RoundRobin,
            priority: 100,
        };

        // Allocate multiple times - should rotate
        let result1 = allocator.allocate(request.clone()).await.unwrap();
        let result2 = allocator.allocate(request.clone()).await.unwrap();
        let result3 = allocator.allocate(request).await.unwrap();

        // Should get different resources (or cycle through)
        assert_eq!(result1.resource_ids.len(), 1);
        assert_eq!(result2.resource_ids.len(), 1);
        assert_eq!(result3.resource_ids.len(), 1);
    }

    #[tokio::test]
    async fn test_shortest_queue_allocation() {
        let allocator = ResourceAllocator::new();

        // Register resources with different queue lengths
        let resource1 = Resource {
            id: ResourceId::new(),
            name: "Resource 1".to_string(),
            roles: vec![],
            capabilities: vec![],
            workload: 0,
            queue_length: 5,
            available: true,
        };

        let resource2 = Resource {
            id: ResourceId::new(),
            name: "Resource 2".to_string(),
            roles: vec![],
            capabilities: vec![],
            workload: 0,
            queue_length: 2,
            available: true,
        };

        allocator.register_resource(resource1).await.unwrap();
        allocator.register_resource(resource2).await.unwrap();

        // Request shortest queue allocation
        let request = AllocationRequest {
            task_id: "task1".to_string(),
            spec_id: WorkflowSpecId::new(),
            required_roles: vec![],
            required_capabilities: vec![],
            policy: AllocationPolicy::ShortestQueue,
            priority: 100,
        };

        let result = allocator.allocate(request).await.unwrap();
        assert_eq!(result.resource_ids.len(), 1);
        assert_eq!(result.policy, AllocationPolicy::ShortestQueue);
    }
}
