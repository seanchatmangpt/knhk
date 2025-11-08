#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! State synchronization for multi-region deployment

use crate::case::Case;
use crate::error::WorkflowResult;
use crate::parser::WorkflowSpec;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Sync strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStrategy {
    /// Eventual consistency
    Eventual,
    /// Strong consistency (quorum-based)
    Strong,
    /// Last-write-wins
    LastWriteWins,
}

/// State synchronization manager
pub struct StateSync {
    strategy: SyncStrategy,
    /// Local state cache
    local_cache: Arc<RwLock<std::collections::HashMap<String, Vec<u8>>>>,
}

impl StateSync {
    /// Create a new state sync manager
    pub fn new(strategy: SyncStrategy) -> Self {
        Self {
            strategy,
            local_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Sync case state
    pub async fn sync_case(&self, case: &Case, region: &str) -> WorkflowResult<()> {
        // FUTURE: Implement actual cross-region sync
        // For now, update local cache
        let mut cache = self.local_cache.write().await;
        let key = format!("case:{}:{}", case.id, region);
        let serialized = serde_json::to_vec(case).map_err(|e| {
            crate::error::WorkflowError::Internal(format!("Serialization error: {}", e))
        })?;
        cache.insert(key, serialized);
        Ok(())
    }

    /// Sync workflow spec
    pub async fn sync_workflow_spec(
        &self,
        spec: &WorkflowSpec,
        region: &str,
    ) -> WorkflowResult<()> {
        // FUTURE: Implement actual cross-region sync
        let mut cache = self.local_cache.write().await;
        let key = format!("spec:{}:{}", spec.id, region);
        let serialized = serde_json::to_vec(spec).map_err(|e| {
            crate::error::WorkflowError::Internal(format!("Serialization error: {}", e))
        })?;
        cache.insert(key, serialized);
        Ok(())
    }

    /// Get sync strategy
    pub fn strategy(&self) -> SyncStrategy {
        self.strategy
    }
}

impl Default for StateSync {
    fn default() -> Self {
        Self::new(SyncStrategy::Eventual)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_sync() {
        let sync = StateSync::new(SyncStrategy::Eventual);
        let case =
            crate::case::Case::new(crate::parser::WorkflowSpecId::new(), serde_json::json!({}));
        sync.sync_case(&case, "us-east-1").await.unwrap();
    }
}
