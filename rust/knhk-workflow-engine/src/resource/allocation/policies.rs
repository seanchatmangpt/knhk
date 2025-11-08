//! Policy-specific allocation logic

use super::types::{AllocationPolicy, AllocationRequest, AllocationResult, Resource, ResourceId};
use crate::error::{WorkflowError, WorkflowResult};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Policy allocation context
pub struct PolicyContext {
    pub resources: Arc<RwLock<HashMap<ResourceId, Resource>>>,
    pub round_robin_state: Arc<RwLock<HashMap<String, VecDeque<ResourceId>>>>,
    pub chained_state: Arc<RwLock<HashMap<String, VecDeque<ResourceId>>>>,
}

impl PolicyContext {
    /// Check if resource matches required roles
    pub fn matches_roles(&self, resource: &Resource, required_roles: &[String]) -> bool {
        if required_roles.is_empty() {
            return true;
        }
        required_roles
            .iter()
            .any(|role_id| resource.roles.iter().any(|r| r.id == *role_id))
    }

    /// Check if resource matches required capabilities
    pub fn matches_capabilities(
        &self,
        resource: &Resource,
        required_capabilities: &[String],
    ) -> bool {
        if required_capabilities.is_empty() {
            return true;
        }
        required_capabilities
            .iter()
            .all(|cap_id| resource.capabilities.iter().any(|c| c.id == *cap_id))
    }
}

/// Four-eyes principle: allocate two resources for approval
pub async fn allocate_four_eyes(
    ctx: &PolicyContext,
    request: &AllocationRequest,
) -> WorkflowResult<AllocationResult> {
    let resources = ctx.resources.read().await;
    let candidates: Vec<ResourceId> = resources
        .values()
        .filter(|r| {
            r.available
                && ctx.matches_roles(r, &request.required_roles)
                && ctx.matches_capabilities(r, &request.required_capabilities)
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
pub async fn allocate_chained(
    ctx: &PolicyContext,
    request: &AllocationRequest,
) -> WorkflowResult<AllocationResult> {
    let mut chained_state = ctx.chained_state.write().await;
    let chain_key = format!("{}:{}", request.spec_id, request.task_id);
    let chain = chained_state
        .entry(chain_key.clone())
        .or_insert_with(VecDeque::new);

    let resources = ctx.resources.read().await;
    let candidates: Vec<ResourceId> = resources
        .values()
        .filter(|r| {
            r.available
                && ctx.matches_roles(r, &request.required_roles)
                && ctx.matches_capabilities(r, &request.required_capabilities)
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
pub async fn allocate_round_robin(
    ctx: &PolicyContext,
    request: &AllocationRequest,
) -> WorkflowResult<AllocationResult> {
    let mut round_robin_state = ctx.round_robin_state.write().await;
    let key = format!("{}:{}", request.spec_id, request.task_id);
    let queue = round_robin_state
        .entry(key.clone())
        .or_insert_with(VecDeque::new);

    let resources = ctx.resources.read().await;
    let candidates: Vec<ResourceId> = resources
        .values()
        .filter(|r| {
            r.available
                && ctx.matches_roles(r, &request.required_roles)
                && ctx.matches_capabilities(r, &request.required_capabilities)
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
    let resource_id = queue
        .pop_front()
        .ok_or_else(|| WorkflowError::ResourceUnavailable("Round-robin queue empty".to_string()))?;

    // Add back to end of queue
    queue.push_back(resource_id);

    Ok(AllocationResult {
        resource_ids: vec![resource_id],
        allocated_at: chrono::Utc::now(),
        policy: AllocationPolicy::RoundRobin,
    })
}

/// Shortest queue allocation
pub async fn allocate_shortest_queue(
    ctx: &PolicyContext,
    request: &AllocationRequest,
) -> WorkflowResult<AllocationResult> {
    let resources = ctx.resources.read().await;
    let mut candidates: Vec<(ResourceId, u32)> = resources
        .values()
        .filter(|r| {
            r.available
                && ctx.matches_roles(r, &request.required_roles)
                && ctx.matches_capabilities(r, &request.required_capabilities)
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
pub async fn allocate_role_based(
    ctx: &PolicyContext,
    request: &AllocationRequest,
) -> WorkflowResult<AllocationResult> {
    let resources = ctx.resources.read().await;
    let candidates: Vec<ResourceId> = resources
        .values()
        .filter(|r| r.available && ctx.matches_roles(r, &request.required_roles))
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
pub async fn allocate_capability_based(
    ctx: &PolicyContext,
    request: &AllocationRequest,
) -> WorkflowResult<AllocationResult> {
    let resources = ctx.resources.read().await;
    let mut candidates: Vec<(ResourceId, u8)> = resources
        .values()
        .filter(|r| r.available && ctx.matches_capabilities(r, &request.required_capabilities))
        .map(|r| {
            // Calculate capability score (sum of matching capability levels)
            let score: u8 = r
                .capabilities
                .iter()
                .filter(|c| request.required_capabilities.contains(&c.id))
                .map(|c| c.level)
                .sum();
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
