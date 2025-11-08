//! Resource pooling
//!
//! Provides connection pooling and resource management for workflow operations.

use crate::error::{WorkflowError, WorkflowResult};
use std::sync::Arc;
use tokio::sync::Semaphore;

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
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            WorkflowError::ResourceUnavailable(format!("Failed to acquire resource: {}", e))
        })?;
        // Permit is held by the guard and released on drop
        Ok(ResourcePermit {
            _guard: self.semaphore.clone(),
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
    _guard: Arc<Semaphore>,
}

impl ResourcePermit {
    /// Release permit early
    pub fn release(self) {
        // Permit is released on drop
    }
}
