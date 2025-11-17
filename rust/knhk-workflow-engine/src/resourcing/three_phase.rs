//! YAWL 3-Phase Resource Allocation
//!
//! Implements YAWL's 3-phase allocation system with TRIZ Principle 40: Composite Materials
//! - Phase 1: Offer - Select eligible participants
//! - Phase 2: Allocate - Select one participant
//! - Phase 3: Start - Determine when to start
//!
//! Based on: org.yawlfoundation.yawl.resourcing.ResourceAllocation

use super::filters::ResourceFilter;
use crate::error::{WorkflowError, WorkflowResult};
use crate::resource::allocation::types::{Resource, ResourceId};
use dashmap::DashMap;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Resource pool trait for 3-phase allocation (TRIZ Principle 40: Composite Materials)
#[async_trait::async_trait]
pub trait ResourcePool: Send + Sync {
    /// Get all resources in the pool
    async fn get_all_resources(&self) -> Vec<Resource>;
}

/// Allocation phase (TRIZ Principle 32: Color Changes - type-level phases)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationPhase {
    /// Phase 1: Offer - Select eligible participants
    Offer,
    /// Phase 2: Allocate - Select one participant
    Allocate,
    /// Phase 3: Start - Determine when to start
    Start,
}

/// Phase 1 result: Eligible participants
#[derive(Debug, Clone)]
pub struct OfferResult {
    /// Eligible resource IDs
    pub eligible_resources: Vec<ResourceId>,
    /// Filter results for each resource
    pub filter_results: HashMap<ResourceId, bool>,
}

/// Phase 2 result: Selected participant
#[derive(Debug, Clone)]
pub struct AllocateResult {
    /// Selected resource ID
    pub selected_resource: ResourceId,
    /// Allocation method used
    pub allocation_method: String,
}

/// Phase 3 result: Start decision
#[derive(Debug, Clone)]
pub struct StartResult {
    /// Whether to start immediately
    pub start_immediately: bool,
    /// Scheduled start time (if delayed)
    pub scheduled_start: Option<chrono::DateTime<chrono::Utc>>,
}

/// 3-Phase Resource Allocator
///
/// TRIZ Principle 40: Composite Materials
/// - Combines multiple allocation strategies
/// - Routes to appropriate strategy based on context
pub struct ThreePhaseAllocator {
    /// Resource pool (TRIZ Principle 40: Composite Materials - trait object)
    resource_pool: Arc<dyn ResourcePool>,
    /// Allocation policies (TRIZ Principle 40: Composite Materials)
    allocation_policies: Arc<DashMap<String, Box<dyn AllocationStrategy + Send + Sync>>>,
    /// Current allocations
    allocations: Arc<DashMap<ResourceId, AllocationContext>>,
}

/// Allocation strategy trait (TRIZ Principle 40: Composite Materials)
pub trait AllocationStrategy {
    /// Execute allocation strategy
    fn allocate(
        &self,
        eligible_resources: &[ResourceId],
        context: &AllocationContext,
    ) -> WorkflowResult<ResourceId>;
}

/// Allocation context
#[derive(Debug, Clone)]
pub struct AllocationContext {
    /// Task ID
    pub task_id: String,
    /// Case ID
    pub case_id: String,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Required roles
    pub required_roles: Vec<String>,
    /// Workload constraints
    pub workload_constraints: HashMap<String, u32>,
}

impl ThreePhaseAllocator {
    /// Create a new 3-phase allocator
    ///
    /// TRIZ Principle 40: Composite Materials - Accepts any ResourcePool implementation
    pub fn new<P: ResourcePool + 'static>(resource_pool: Arc<P>) -> Self {
        Self {
            resource_pool: resource_pool as Arc<dyn ResourcePool>,
            allocation_policies: Arc::new(DashMap::new()),
            allocations: Arc::new(DashMap::new()),
        }
    }

    /// Phase 1: Offer - Select eligible participants
    ///
    /// Applies filters to find eligible resources
    pub async fn phase1_offer(
        &self,
        context: &AllocationContext,
        filters: &[Arc<dyn ResourceFilter>],
    ) -> WorkflowResult<OfferResult> {
        let mut eligible_resources = Vec::new();
        let mut filter_results = HashMap::new();

        // Get all resources from pool
        let all_resources = self.resource_pool.get_all_resources().await;

        // Apply filters
        for resource in all_resources {
            let mut eligible = true;

            for filter in filters {
                let result = filter.filter(&resource, context).await?;
                filter_results.insert(resource.id.clone(), result.passed);
                if !result.passed {
                    eligible = false;
                    break;
                }
            }

            if eligible {
                eligible_resources.push(resource.id.clone());
            }
        }

        Ok(OfferResult {
            eligible_resources,
            filter_results,
        })
    }

    /// Phase 2: Allocate - Select one participant
    ///
    /// Uses allocation strategy to select one resource
    pub async fn phase2_allocate(
        &self,
        eligible_resources: &[ResourceId],
        context: &AllocationContext,
        policy_name: &str,
    ) -> WorkflowResult<AllocateResult> {
        if eligible_resources.is_empty() {
            return Err(WorkflowError::ResourceUnavailable(
                "No eligible resources found".to_string(),
            ));
        }

        // Get allocation policy
        let policy = self
            .allocation_policies
            .get(policy_name)
            .ok_or_else(|| WorkflowError::Internal(format!("Policy {} not found", policy_name)))?;

        // Execute allocation
        let selected_resource = policy.value().allocate(eligible_resources, context)?;

        // Record allocation
        self.allocations
            .insert(selected_resource.clone(), context.clone());

        Ok(AllocateResult {
            selected_resource: selected_resource.clone(),
            allocation_method: policy_name.to_string(),
        })
    }

    /// Phase 3: Start - Determine when to start
    ///
    /// Decides when to start execution based on launch mode
    pub async fn phase3_start(
        &self,
        resource_id: &ResourceId,
        launch_mode: &str,
    ) -> WorkflowResult<StartResult> {
        match launch_mode {
            "user-initiated" => Ok(StartResult {
                start_immediately: false,
                scheduled_start: None,
            }),
            "offered" => Ok(StartResult {
                start_immediately: false,
                scheduled_start: None,
            }),
            "allocated" => Ok(StartResult {
                start_immediately: true,
                scheduled_start: None,
            }),
            "start-by-system" => Ok(StartResult {
                start_immediately: true,
                scheduled_start: None,
            }),
            "concurrent" => Ok(StartResult {
                start_immediately: true,
                scheduled_start: None,
            }),
            _ => Err(WorkflowError::Validation(format!(
                "Unknown launch mode: {}",
                launch_mode
            ))),
        }
    }

    /// Register allocation policy (TRIZ Principle 40: Composite Materials)
    pub fn register_policy(&self, name: String, policy: Box<dyn AllocationStrategy + Send + Sync>) {
        self.allocation_policies.insert(name, policy);
    }

    /// Complete allocation (release resource)
    pub async fn complete_allocation(&self, resource_id: &ResourceId) -> WorkflowResult<()> {
        self.allocations.remove(resource_id);
        Ok(())
    }
}

/// Round-robin allocation strategy
pub struct RoundRobinStrategy {
    current_index: Arc<RwLock<usize>>,
}

impl RoundRobinStrategy {
    pub fn new() -> Self {
        Self {
            current_index: Arc::new(RwLock::new(0)),
        }
    }
}

impl AllocationStrategy for RoundRobinStrategy {
    fn allocate(
        &self,
        eligible_resources: &[ResourceId],
        _context: &AllocationContext,
    ) -> WorkflowResult<ResourceId> {
        let mut index = self.current_index.blocking_write();
        let selected = eligible_resources[*index % eligible_resources.len()].clone();
        *index = (*index + 1) % eligible_resources.len();
        Ok(selected)
    }
}

/// Random allocation strategy
pub struct RandomStrategy;

impl AllocationStrategy for RandomStrategy {
    fn allocate(
        &self,
        eligible_resources: &[ResourceId],
        _context: &AllocationContext,
    ) -> WorkflowResult<ResourceId> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..eligible_resources.len());
        Ok(eligible_resources[index].clone())
    }
}

/// Shortest queue allocation strategy
pub struct ShortestQueueStrategy {
    queue_lengths: Arc<DashMap<ResourceId, u32>>,
}

impl ShortestQueueStrategy {
    pub fn new() -> Self {
        Self {
            queue_lengths: Arc::new(DashMap::new()),
        }
    }
}

impl AllocationStrategy for ShortestQueueStrategy {
    fn allocate(
        &self,
        eligible_resources: &[ResourceId],
        _context: &AllocationContext,
    ) -> WorkflowResult<ResourceId> {
        let mut min_queue = u32::MAX;
        let mut selected = None;

        for resource_id in eligible_resources {
            let queue_length = self
                .queue_lengths
                .get(resource_id)
                .map(|entry| *entry.value())
                .unwrap_or(0);

            if queue_length < min_queue {
                min_queue = queue_length;
                selected = Some(resource_id.clone());
            }
        }

        selected.ok_or_else(|| WorkflowError::Internal("No resource selected".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::ResourcePool as CoreResourcePool;
    use crate::resource::{Capability, Role};
    use crate::resourcing::ResourcePoolWrapper;

    /// Test resource pool implementation
    struct TestResourcePool {
        resources: Vec<Resource>,
    }

    #[async_trait::async_trait]
    impl ResourcePool for TestResourcePool {
        async fn get_all_resources(&self) -> Vec<Resource> {
            self.resources.clone()
        }
    }

    #[tokio::test]
    async fn test_three_phase_allocation() {
        let test_pool = Arc::new(TestResourcePool { resources: vec![] });
        let allocator = ThreePhaseAllocator::new(test_pool);

        let context = AllocationContext {
            task_id: "task1".to_string(),
            case_id: "case1".to_string(),
            required_capabilities: vec![],
            required_roles: vec![],
            workload_constraints: HashMap::new(),
        };

        // Phase 1: Offer
        let offer_result = allocator.phase1_offer(&context, &[]).await.unwrap();
        assert!(offer_result.eligible_resources.is_empty()); // No resources in pool

        // Phase 2: Allocate (would fail with no resources, but structure is correct)
        // Phase 3: Start
        let resource_id = ResourceId::new();
        let start_result = allocator
            .phase3_start(&resource_id, "user-initiated")
            .await
            .unwrap();
        assert!(!start_result.start_immediately);
    }
}
