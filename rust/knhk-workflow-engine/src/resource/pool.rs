//! Resource pooling
//!
//! Provides connection pooling and resource management for workflow operations.

use crate::error::{WorkflowError, WorkflowResult};
use std::sync::Arc;
use tokio::sync::{Semaphore, SemaphorePermit};

/// Resource pool for managing limited resources
pub struct ResourcePool {
    /// Maximum concurrent resources
    max_resources: usize,
    /// Semaphore for resource access
    semaphore: Arc<Semaphore>,
}

impl ResourcePool {
    /// Create new resource pool
    pub fn new(max_resources: usize) -> Self {
        Self {
            max_resources,
            semaphore: Arc::new(Semaphore::new(max_resources)),
        }
    }

    /// Acquire resource permit
    pub async fn acquire(&self) -> WorkflowResult<ResourcePermit> {
        let permit = self.semaphore.acquire().await.map_err(|e| {
            WorkflowError::ResourceUnavailable(format!("Failed to acquire resource: {}", e))
        })?;
        Ok(ResourcePermit {
            permit: permit.forget(),
        })
    }

    /// Get available resources count
    pub fn available(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Get maximum resources
    pub fn max_resources(&self) -> usize {
        self.max_resources
    }
}

/// Resource permit (automatically released on drop)
pub struct ResourcePermit {
    permit: SemaphorePermit<'static>,
}

impl ResourcePermit {
    /// Release permit early
    pub fn release(self) {
        drop(self.permit);
    }
}
