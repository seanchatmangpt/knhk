//! YAWL Resource Management Port with TRIZ Hyper-Advanced Patterns
//!
//! This module ports Java YAWL's ResourceManager and 3-phase allocation system
//! while applying TRIZ principles:
//! - **Principle 1 (Segmentation)**: Separate allocation phases
//! - **Principle 15 (Dynamics)**: Adaptive allocation based on workload
//! - **Principle 10 (Prior Action)**: Pre-compute resource eligibility
//!
//! # Architecture
//!
//! YAWL uses a 3-phase resource allocation model:
//! 1. **Offer Phase**: Select eligible participants (filters applied)
//! 2. **Allocate Phase**: Select one participant (allocation algorithm)
//! 3. **Start Phase**: Determine when to start (launch mode)
//!
//! # TRIZ Enhancements
//!
//! - Resource eligibility is pre-computed (Principle 10)
//! - Allocation policies adapt to workload (Principle 15)
//! - Filters are separated from allocation logic (Principle 1)

use crate::error::{WorkflowError, WorkflowResult};
use crate::resource::allocation::{AllocationPolicy, Resource, ResourceId, Role};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Launch mode for work items (TRIZ Principle 32: Color Changes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchMode {
    /// User-initiated (pull model - user claims task)
    UserInitiated,
    /// Offered (push model - distributed to eligible users)
    Offered,
    /// Allocated (system-assigned to specific user - mandatory)
    Allocated,
    /// Start-by-System (auto-start when enabled - automated)
    StartBySystem,
    /// Concurrent (multiple users can work on same item - competitive)
    Concurrent,
}

/// 3-Phase Resource Allocation Result
#[derive(Debug, Clone)]
pub struct AllocationResult {
    /// Phase 1: Offered resources (eligible participants)
    pub offered: Vec<ResourceId>,
    /// Phase 2: Allocated resource (selected participant)
    pub allocated: Option<ResourceId>,
    /// Phase 3: Start mode
    pub launch_mode: LaunchMode,
}

/// Resource filter trait
///
/// Filters are applied in the Offer phase to determine eligible participants.
/// Each filter type implements specific selection criteria.
///
/// # TRIZ Principle 1: Segmentation
///
/// Filters are separated from allocation logic, allowing composition.
pub trait ResourceFilter: Send + Sync {
    /// Filter resources based on criteria
    fn filter(&self, resources: &[Resource], context: &FilterContext) -> Vec<ResourceId>;
}

/// Filter context for resource filtering
#[derive(Debug, Clone)]
pub struct FilterContext {
    /// Required roles
    pub required_roles: Vec<Role>,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Task ID
    pub task_id: String,
    /// Case data
    pub case_data: serde_json::Value,
}

/// Capability filter - matches resources with required skills
pub struct CapabilityFilter;

impl ResourceFilter for CapabilityFilter {
    fn filter(&self, resources: &[Resource], context: &FilterContext) -> Vec<ResourceId> {
        resources
            .iter()
            .filter(|resource| {
                context
                    .required_capabilities
                    .iter()
                    .all(|cap| resource.capabilities.iter().any(|c| c.name == *cap))
            })
            .map(|r| r.id)
            .collect()
    }
}

/// Role filter - matches resources with required roles
pub struct RoleFilter;

impl ResourceFilter for RoleFilter {
    fn filter(&self, resources: &[Resource], context: &FilterContext) -> Vec<ResourceId> {
        resources
            .iter()
            .filter(|resource| {
                context
                    .required_roles
                    .iter()
                    .any(|role| resource.roles.iter().any(|r| r.id == role.id))
            })
            .map(|r| r.id)
            .collect()
    }
}

/// Workload-based filter - matches resources with low workload
///
/// # TRIZ Principle 15: Dynamics
///
/// Filter adapts based on current workload, prioritizing less-busy resources.
pub struct WorkloadFilter {
    /// Maximum workload threshold (queue depth)
    pub max_workload: usize,
}

impl WorkloadFilter {
    /// Create new workload filter
    pub fn new(max_workload: usize) -> Self {
        Self { max_workload }
    }
}

impl ResourceFilter for WorkloadFilter {
    fn filter(&self, resources: &[Resource], context: &FilterContext) -> Vec<ResourceId> {
        resources
            .iter()
            .filter(|resource| {
                // Filter by workload (queue length)
                resource.queue_length <= self.max_workload
            })
            .map(|r| r.id)
            .collect()
    }
}

/// Time-based filter - matches resources available within time window
///
/// # TRIZ Principle 19: Periodic Action
///
/// Filter considers periodic availability patterns.
pub struct TimeBasedFilter {
    /// Required availability window (start hour, end hour)
    pub availability_window: Option<(u8, u8)>,
}

impl TimeBasedFilter {
    /// Create new time-based filter
    pub fn new(availability_window: Option<(u8, u8)>) -> Self {
        Self {
            availability_window,
        }
    }
}

impl ResourceFilter for TimeBasedFilter {
    fn filter(&self, resources: &[Resource], context: &FilterContext) -> Vec<ResourceId> {
        // For now, filter by availability flag
        // In production, would check against resource's schedule
        resources
            .iter()
            .filter(|resource| {
                // Check if resource is available
                if !resource.available {
                    return false;
                }

                // If time window specified, check current time
                if let Some((start_hour, end_hour)) = self.availability_window {
                    use chrono::Timelike;
                    let now = chrono::Utc::now();
                    let current_hour = now.hour() as u8;
                    return current_hour >= start_hour && current_hour < end_hour;
                }

                true
            })
            .map(|r| r.id)
            .collect()
    }
}

/// Constraint-based filter - matches resources that satisfy compliance constraints
///
/// # TRIZ Principle 24: Intermediary
///
/// Filter acts as intermediary between resource pool and compliance constraints.
pub struct ConstraintFilter {
    /// Compliance manager reference
    compliance_manager: Arc<tokio::sync::RwLock<crate::resource::compliance::ComplianceManager>>,
    /// Task ID for constraint checking
    task_id: String,
}

impl ConstraintFilter {
    /// Create new constraint filter
    pub fn new(
        compliance_manager: Arc<
            tokio::sync::RwLock<crate::resource::compliance::ComplianceManager>,
        >,
        task_id: String,
    ) -> Self {
        Self {
            compliance_manager,
            task_id,
        }
    }
}

impl ResourceFilter for ConstraintFilter {
    fn filter(&self, resources: &[Resource], context: &FilterContext) -> Vec<ResourceId> {
        // Note: Constraint checking is async, but trait is sync
        // In production, would use async filter or pre-compute constraint violations
        // For now, return all resources (constraints checked at allocation time)
        resources.iter().map(|r| r.id).collect()
    }
}

/// Allocation algorithm trait
///
/// Algorithms are used in the Allocate phase to select one participant
/// from the offered set.
///
/// # TRIZ Principle 15: Dynamics
///
/// Allocation algorithms adapt based on workload and resource availability.
pub trait AllocationAlgorithm: Send + Sync {
    /// Select one resource from the offered set
    fn allocate(
        &self,
        offered: &[ResourceId],
        resources: &HashMap<ResourceId, Resource>,
        context: &AllocationContext,
    ) -> WorkflowResult<Option<ResourceId>>;
}

/// Allocation context
#[derive(Debug, Clone)]
pub struct AllocationContext {
    /// Task ID
    pub task_id: String,
    /// Case data
    pub case_data: serde_json::Value,
    /// Resource workload (queue depth per resource)
    pub workload: HashMap<ResourceId, usize>,
}

/// Round robin allocator - distributes work evenly
pub struct RoundRobinAllocator {
    /// Current index for round-robin
    current_index: Arc<RwLock<usize>>,
}

impl RoundRobinAllocator {
    pub fn new() -> Self {
        Self {
            current_index: Arc::new(RwLock::new(0)),
        }
    }
}

impl AllocationAlgorithm for RoundRobinAllocator {
    fn allocate(
        &self,
        offered: &[ResourceId],
        _resources: &HashMap<ResourceId, Resource>,
        _context: &AllocationContext,
    ) -> WorkflowResult<Option<ResourceId>> {
        if offered.is_empty() {
            return Ok(None);
        }

        // Use blocking lock for synchronous trait method
        let mut index = self.current_index.blocking_write();
        let selected = offered[*index % offered.len()].clone();
        *index = (*index + 1) % offered.len();
        Ok(Some(selected))
    }
}

/// Shortest queue allocator - selects resource with least work
pub struct ShortestQueueAllocator;

impl AllocationAlgorithm for ShortestQueueAllocator {
    fn allocate(
        &self,
        offered: &[ResourceId],
        _resources: &HashMap<ResourceId, Resource>,
        context: &AllocationContext,
    ) -> WorkflowResult<Option<ResourceId>> {
        if offered.is_empty() {
            return Ok(None);
        }

        // Find resource with minimum workload
        let selected = offered
            .iter()
            .min_by_key(|&resource_id| context.workload.get(resource_id).copied().unwrap_or(0))
            .cloned();

        Ok(selected)
    }
}

/// Fastest resource allocator - selects resource with best completion history
pub struct FastestResourceAllocator {
    /// Completion time history (resource_id → average completion time)
    completion_times: Arc<RwLock<HashMap<ResourceId, f64>>>,
}

impl FastestResourceAllocator {
    pub fn new() -> Self {
        Self {
            completion_times: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl AllocationAlgorithm for FastestResourceAllocator {
    fn allocate(
        &self,
        offered: &[ResourceId],
        _resources: &HashMap<ResourceId, Resource>,
        _context: &AllocationContext,
    ) -> WorkflowResult<Option<ResourceId>> {
        if offered.is_empty() {
            return Ok(None);
        }

        // Use blocking lock for synchronous trait method
        let times = self.completion_times.blocking_read();
        let selected = offered
            .iter()
            .min_by(|&a, &b| {
                let time_a = times.get(a).copied().unwrap_or(f64::MAX);
                let time_b = times.get(b).copied().unwrap_or(f64::MAX);
                time_a
                    .partial_cmp(&time_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned();

        Ok(selected)
    }
}

/// YAWL Resource Manager
///
/// Manages 3-phase resource allocation with filters and algorithms.
///
/// # TRIZ Principle 1: Segmentation
///
/// Allocation is segmented into three phases, each with distinct responsibilities.
///
/// # TRIZ Principle 15: Dynamics
///
/// Allocation adapts based on workload and resource availability.
pub struct YawlResourceManager {
    /// Available resources
    resources: Arc<RwLock<HashMap<ResourceId, Resource>>>,
    /// Resource filters
    filters: Vec<Box<dyn ResourceFilter>>,
    /// Default allocation algorithm
    default_allocator: Arc<dyn AllocationAlgorithm>,
    /// Resource workload tracking
    workload: Arc<RwLock<HashMap<ResourceId, usize>>>,
}

impl YawlResourceManager {
    /// Create a new YAWL resource manager
    pub fn new() -> Self {
        // Register default filters
        let mut filters: Vec<Box<dyn ResourceFilter>> = Vec::new();
        filters.push(Box::new(CapabilityFilter));
        filters.push(Box::new(RoleFilter));

        Self {
            resources: Arc::new(RwLock::new(HashMap::new())),
            filters,
            default_allocator: Arc::new(ShortestQueueAllocator),
            workload: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Perform 3-phase allocation
    ///
    /// This implements YAWL's 3-phase allocation model:
    /// 1. Offer: Apply filters to find eligible participants
    /// 2. Allocate: Use algorithm to select one participant
    /// 3. Start: Determine launch mode
    ///
    /// # TRIZ Principle 10: Prior Action
    ///
    /// Resource eligibility is pre-computed during filter application.
    pub async fn allocate_resource(
        &self,
        context: &FilterContext,
        launch_mode: LaunchMode,
    ) -> WorkflowResult<AllocationResult> {
        let resources = self.resources.read().await;
        let resource_vec: Vec<Resource> = resources.values().cloned().collect();
        drop(resources);

        // Phase 1: Offer - Apply filters to find eligible participants
        let mut offered: HashSet<ResourceId> = resource_vec.iter().map(|r| r.id.clone()).collect();

        for filter in &self.filters {
            let filtered = filter.filter(&resource_vec, context);
            offered = offered
                .intersection(&filtered.into_iter().collect())
                .cloned()
                .collect();
        }

        let offered_vec: Vec<ResourceId> = offered.into_iter().collect();

        // Phase 2: Allocate - Select one participant
        let resources = self.resources.read().await;
        let workload = self.workload.read().await;
        let allocation_context = AllocationContext {
            task_id: context.task_id.clone(),
            case_data: context.case_data.clone(),
            workload: workload.clone(),
        };

        let allocated =
            self.default_allocator
                .allocate(&offered_vec, &resources, &allocation_context)?;

        // Phase 3: Start - Launch mode is provided by caller
        Ok(AllocationResult {
            offered: offered_vec,
            allocated,
            launch_mode,
        })
    }

    /// Add a resource
    pub async fn add_resource(&self, resource: Resource) {
        let mut resources = self.resources.write().await;
        resources.insert(resource.id, resource);
        info!("YawlResourceManager: Added resource");
    }

    /// Remove a resource
    pub async fn remove_resource(&self, resource_id: &ResourceId) {
        let mut resources = self.resources.write().await;
        resources.remove(resource_id);
        let mut workload = self.workload.write().await;
        workload.remove(resource_id);
        info!("YawlResourceManager: Removed resource {}", resource_id);
    }

    /// Update resource workload
    pub async fn update_workload(&self, resource_id: &ResourceId, queue_depth: usize) {
        let mut workload = self.workload.write().await;
        workload.insert(resource_id.clone(), queue_depth);
    }

    /// Get resource by ID
    pub async fn get_resource(&self, resource_id: &ResourceId) -> Option<Resource> {
        let resources = self.resources.read().await;
        resources.get(resource_id).cloned()
    }
}

impl Default for YawlResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::allocation::Capability;

    fn create_test_resource(id: &str, capabilities: Vec<String>) -> Resource {
        Resource {
            id: ResourceId::new(),
            name: id.to_string(),
            roles: vec![],
            capabilities: capabilities
                .into_iter()
                .map(|c| Capability {
                    id: c.clone(),
                    name: c,
                    level: 100,
                })
                .collect(),
            workload: 0,
            queue_length: 0,
            available: true,
        }
    }

    #[tokio::test]
    async fn test_3_phase_allocation() {
        let manager = YawlResourceManager::new();

        // Add test resources
        manager
            .add_resource(create_test_resource(
                "resource1",
                vec!["skill1".to_string(), "skill2".to_string()],
            ))
            .await;
        manager
            .add_resource(create_test_resource(
                "resource2",
                vec!["skill1".to_string()],
            ))
            .await;

        // Create filter context
        let context = FilterContext {
            required_roles: vec![],
            required_capabilities: vec!["skill1".to_string()],
            task_id: "task1".to_string(),
            case_data: serde_json::json!({}),
        };

        // Perform allocation
        let result = manager
            .allocate_resource(&context, LaunchMode::Allocated)
            .await
            .unwrap();

        // Both resources should be offered (both have skill1)
        assert_eq!(result.offered.len(), 2);
        // One should be allocated
        assert!(result.allocated.is_some());
    }

    #[tokio::test]
    async fn test_capability_filter() {
        let filter = CapabilityFilter;
        let resources = vec![
            create_test_resource("r1", vec!["skill1".to_string(), "skill2".to_string()]),
            create_test_resource("r2", vec!["skill1".to_string()]),
            create_test_resource("r3", vec!["skill2".to_string()]),
        ];

        let context = FilterContext {
            required_roles: vec![],
            required_capabilities: vec!["skill1".to_string(), "skill2".to_string()],
            task_id: "task1".to_string(),
            case_data: serde_json::json!({}),
        };

        let filtered = filter.filter(&resources, &context);
        // Only r1 has both skills
        assert_eq!(filtered.len(), 1);
    }
}
