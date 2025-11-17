//! Resource Pool Implementation for 3-Phase Allocation
//!
//! Implements ResourcePool trait for core ResourcePool type
//!
//! TRIZ Principle 24: Intermediary - Adapter pattern for resource pool integration

use crate::resource::allocation::types::Resource;
use crate::resource::ResourcePool as CoreResourcePool;
use crate::resourcing::three_phase::ResourcePool as ResourcePoolTrait;
use std::sync::Arc;

/// Wrapper that implements ResourcePool trait for core ResourcePool
///
/// TRIZ Principle 24: Intermediary - Adapts core pool to trait interface
/// TRIZ Principle 40: Composite Materials - Enables multiple pool implementations
pub struct ResourcePoolWrapper {
    pool: Arc<CoreResourcePool>,
}

impl ResourcePoolWrapper {
    /// Create new wrapper
    pub fn new(pool: Arc<CoreResourcePool>) -> Self {
        Self { pool }
    }

    /// Get underlying pool
    pub fn inner(&self) -> &Arc<CoreResourcePool> {
        &self.pool
    }
}

#[async_trait::async_trait]
impl ResourcePoolTrait for ResourcePoolWrapper {
    async fn get_all_resources(&self) -> Vec<Resource> {
        self.pool.get_all_resources().await
    }
}
