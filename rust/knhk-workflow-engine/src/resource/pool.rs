//! Resource Pool Manager - Improved resource management
//!
//! Provides:
//! - Resource pools
//! - Resource lifecycle management
//! - Better scheduling

use crate::error::{WorkflowError, WorkflowResult};
use crate::resource::{AllocationRequest, AllocationResult, Resource, ResourceId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Resource pool manager
pub struct ResourcePoolManager {
    /// Resource pools by type
    pools: Arc<RwLock<HashMap<String, ResourcePool>>>,
    /// Resource registry
    resources: Arc<RwLock<HashMap<ResourceId, Resource>>>,
}

/// Resource pool
///
/// TRIZ Principle 37: Thermal Expansion - Pool size adapts to load temperature
pub struct ResourcePool {
    /// Pool name
    pub name: String,
    /// Available resources
    pub available: Vec<ResourceId>,
    /// Allocated resources
    pub allocated: HashMap<String, ResourceId>,
    /// Pool size
    pub size: usize,
    /// Resource registry (for lookup)
    resources: Arc<RwLock<HashMap<ResourceId, Resource>>>,
}

impl ResourcePoolManager {
    /// Create new resource pool manager
    pub fn new() -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create resource pool
    pub async fn create_pool(&self, name: String, size: usize) -> WorkflowResult<()> {
        let resources = self.resources.clone();
        let mut pools = self.pools.write().await;
        pools.insert(
            name.clone(),
            ResourcePool {
                name,
                available: Vec::new(),
                allocated: HashMap::new(),
                size,
                resources,
            },
        );
        Ok(())
    }

    /// Allocate resource from pool
    pub async fn allocate_from_pool(
        &self,
        pool_name: &str,
        request: &AllocationRequest,
    ) -> WorkflowResult<AllocationResult> {
        let mut pools = self.pools.write().await;
        let pool = pools.get_mut(pool_name).ok_or_else(|| {
            WorkflowError::ResourceUnavailable(format!("Pool {} not found", pool_name))
        })?;

        // Find available resource
        if let Some(resource_id) = pool.available.pop() {
            let allocation_key = format!("{}:{}", request.spec_id, request.task_id);
            pool.allocated.insert(allocation_key, resource_id);

            Ok(AllocationResult {
                resource_ids: vec![resource_id],
                allocated_at: chrono::Utc::now(),
                policy: request.policy,
            })
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "No available resources in pool {}",
                pool_name
            )))
        }
    }

    /// Release resource back to pool
    pub async fn release_to_pool(
        &self,
        pool_name: &str,
        resource_id: ResourceId,
    ) -> WorkflowResult<()> {
        let mut pools = self.pools.write().await;
        let pool = pools.get_mut(pool_name).ok_or_else(|| {
            WorkflowError::ResourceUnavailable(format!("Pool {} not found", pool_name))
        })?;

        // Remove from allocated
        pool.allocated.retain(|_, &mut id| id != resource_id);

        // Add back to available
        pool.available.push(resource_id);

        Ok(())
    }

    /// Register resource
    pub async fn register_resource(&self, resource: Resource) -> WorkflowResult<()> {
        let mut resources = self.resources.write().await;
        resources.insert(resource.id.clone(), resource);
        Ok(())
    }

    /// Get all resources (for filtering/allocation)
    pub async fn get_all_resources(&self) -> Vec<Resource> {
        let resources = self.resources.read().await;
        resources.values().cloned().collect()
    }
}

impl ResourcePool {
    /// Create a new resource pool
    pub fn new(name: String, resources: Arc<RwLock<HashMap<ResourceId, Resource>>>) -> Self {
        Self {
            name,
            available: Vec::new(),
            allocated: HashMap::new(),
            size: 0,
            resources,
        }
    }

    /// Get all resources in this pool
    pub async fn get_all_resources(&self) -> Vec<Resource> {
        let resources = self.resources.read().await;
        resources.values().cloned().collect()
    }

    /// Add resource to pool
    pub fn add_resource(&mut self, resource_id: ResourceId) {
        self.available.push(resource_id);
        self.size += 1;
    }
}

impl Default for ResourcePoolManager {
    fn default() -> Self {
        Self::new()
    }
}
