//! Resource Pool Adapter for 3-Phase Allocation
//!
//! TRIZ Principle 24: Intermediary - Adapter pattern for resource pool integration
//! Provides unified interface for resource pool access

use crate::error::{WorkflowError, WorkflowResult};
use crate::resource::{Resource, ResourceId, ResourcePool as CoreResourcePool};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Resource pool adapter for 3-phase allocation
///
/// TRIZ Principle 24: Intermediary - Adapts core ResourcePool to 3-phase allocator needs
pub struct ResourcePoolAdapter {
    /// Core resource pool
    pool: Arc<CoreResourcePool>,
    /// Resource registry
    resources: Arc<RwLock<std::collections::HashMap<ResourceId, Resource>>>,
}

impl ResourcePoolAdapter {
    /// Create new adapter from core resource pool
    pub fn new(pool: Arc<CoreResourcePool>) -> Self {
        Self {
            pool,
            resources: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Get all resources for filtering/allocation
    pub async fn get_all_resources(&self) -> Vec<Resource> {
        self.pool.get_all_resources().await
    }

    /// Register resource
    pub async fn register_resource(&self, resource: Resource) -> WorkflowResult<()> {
        let mut resources = self.resources.write().await;
        resources.insert(resource.id.clone(), resource);
        Ok(())
    }

    /// Get resource by ID
    pub async fn get_resource(&self, resource_id: &ResourceId) -> Option<Resource> {
        let resources = self.resources.read().await;
        resources.get(resource_id).cloned()
    }
}

/// Trait for resource pool access (TRIZ Principle 40: Composite Materials)
///
/// Allows multiple pool implementations to be used
pub trait ResourcePoolAccess: Send + Sync {
    /// Get all resources
    fn get_all_resources(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<Resource>> + Send>>;
}

impl ResourcePoolAccess for ResourcePoolAdapter {
    fn get_all_resources(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<Resource>> + Send>> {
        Box::pin(self.get_all_resources())
    }
}

