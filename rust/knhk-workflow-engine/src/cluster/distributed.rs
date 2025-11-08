#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Distributed state store for multi-region deployment

// Unused imports removed - will be used when implementing distributed state
use crate::state::StateStore;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Replication configuration
#[derive(Debug, Clone)]
pub struct ReplicationConfig {
    /// Number of replicas
    pub replica_count: usize,
    /// Quorum threshold (0.0-1.0)
    pub quorum_threshold: f64,
    /// Cross-region sync enabled
    pub cross_region_sync: bool,
    /// Sync endpoints (other regions)
    pub sync_endpoints: Vec<String>,
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            replica_count: 3,
            quorum_threshold: 0.5,
            cross_region_sync: false,
            sync_endpoints: Vec::new(),
        }
    }
}

/// Distributed state store wrapper
pub struct DistributedStateStore {
    local_store: Arc<RwLock<StateStore>>,
    config: ReplicationConfig,
    /// Region identifier
    region: String,
}

impl DistributedStateStore {
    /// Create a new distributed state store
    pub fn new(local_store: StateStore, region: String, config: ReplicationConfig) -> Self {
        Self {
            local_store: Arc::new(RwLock::new(local_store)),
            config,
            region,
        }
    }

    /// Get local store
    pub fn local_store(&self) -> Arc<RwLock<StateStore>> {
        Arc::clone(&self.local_store)
    }

    /// Get replication config
    pub fn config(&self) -> &ReplicationConfig {
        &self.config
    }

    /// Get region
    pub fn region(&self) -> &str {
        &self.region
    }

    /// Check if quorum is met
    pub fn check_quorum(&self, responses: usize) -> bool {
        let threshold =
            (self.config.replica_count as f64 * self.config.quorum_threshold).ceil() as usize;
        responses >= threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateStore;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_distributed_state_store() {
        let temp_dir = TempDir::new().unwrap();
        let local_store = StateStore::new(temp_dir.path()).unwrap();
        let config = ReplicationConfig::default();
        let distributed = DistributedStateStore::new(local_store, "us-east-1".to_string(), config);

        assert_eq!(distributed.region(), "us-east-1");
        assert!(distributed.check_quorum(2));
    }
}
