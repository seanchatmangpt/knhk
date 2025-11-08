//! Resource allocation types

use crate::parser::WorkflowSpecId;
use serde::{Deserialize, Serialize};
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
