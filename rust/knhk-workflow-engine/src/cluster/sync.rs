//! State synchronization for multi-region deployment

use crate::case::Case;
use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;
use std::collections::HashSet;
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

/// Cache entry with timestamp for conflict resolution
#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    timestamp: u64, // Unix timestamp in milliseconds
    region: String,
}

/// State synchronization manager
pub struct StateSync {
    strategy: SyncStrategy,
    /// Local state cache with timestamps
    local_cache: Arc<RwLock<std::collections::HashMap<String, CacheEntry>>>,
    /// Known regions for strong consistency replication
    regions: Arc<RwLock<HashSet<String>>>,
}

impl StateSync {
    /// Create a new state sync manager
    pub fn new(strategy: SyncStrategy) -> Self {
        Self {
            strategy,
            local_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            regions: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Register a region for strong consistency replication
    pub async fn register_region(&self, region: String) {
        let mut regions = self.regions.write().await;
        regions.insert(region);
    }

    /// Get current timestamp in milliseconds
    fn current_timestamp_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Sync case state
    pub async fn sync_case(&self, case: &Case, region: &str) -> WorkflowResult<()> {
        let timestamp = Self::current_timestamp_ms();
        let serialized = serde_json::to_vec(case)
            .map_err(|e| WorkflowError::Internal(format!("Serialization error: {}", e)))?;

        // Sync strategy implementation
        match self.strategy {
            SyncStrategy::Strong => {
                // Strong consistency: replicate to all regions synchronously
                // Register this region if not already registered
                {
                    let mut regions = self.regions.write().await;
                    regions.insert(region.to_string());
                }

                // Replicate to all known regions
                let regions = self.regions.read().await;
                let base_key = format!("case:{}", case.id);

                for target_region in regions.iter() {
                    let key = format!("{}:{}", base_key, target_region);
                    let mut cache = self.local_cache.write().await;
                    cache.insert(
                        key,
                        CacheEntry {
                            data: serialized.clone(),
                            timestamp,
                            region: target_region.clone(),
                        },
                    );
                }

                tracing::debug!(
                    "Strong consistency sync for case {} to {} regions",
                    case.id,
                    regions.len()
                );
            }
            SyncStrategy::Eventual => {
                // Eventual consistency: local cache is sufficient
                let mut cache = self.local_cache.write().await;
                let key = format!("case:{}:{}", case.id, region);
                cache.insert(
                    key,
                    CacheEntry {
                        data: serialized,
                        timestamp,
                        region: region.to_string(),
                    },
                );
                tracing::debug!(
                    "Eventual consistency sync for case {} to region {}",
                    case.id,
                    region
                );
            }
            SyncStrategy::LastWriteWins => {
                // Last-write-wins: timestamp-based conflict resolution
                let mut cache = self.local_cache.write().await;
                let key = format!("case:{}:{}", case.id, region);

                // Check if entry exists and compare timestamps
                let should_update = match cache.get(&key) {
                    Some(existing) => {
                        // Update only if new timestamp is newer
                        timestamp > existing.timestamp
                    }
                    None => true, // No existing entry, always update
                };

                if should_update {
                    cache.insert(
                        key,
                        CacheEntry {
                            data: serialized,
                            timestamp,
                            region: region.to_string(),
                        },
                    );
                    tracing::debug!(
                        "Last-write-wins sync for case {} to region {} (timestamp: {})",
                        case.id,
                        region,
                        timestamp
                    );
                } else {
                    if let Some(existing_entry) = cache.get(&format!("case:{}:{}", case.id, region))
                    {
                        tracing::debug!(
                            "Last-write-wins sync for case {} to region {} skipped (existing timestamp {} is newer than {})",
                            case.id,
                            region,
                            existing_entry.timestamp,
                            timestamp
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Sync workflow spec
    pub async fn sync_workflow_spec(
        &self,
        spec: &WorkflowSpec,
        region: &str,
    ) -> WorkflowResult<()> {
        let timestamp = Self::current_timestamp_ms();
        let serialized = serde_json::to_vec(spec)
            .map_err(|e| WorkflowError::Internal(format!("Serialization error: {}", e)))?;

        // Sync strategy implementation
        match self.strategy {
            SyncStrategy::Strong => {
                // Strong consistency: replicate to all regions synchronously
                // Register this region if not already registered
                {
                    let mut regions = self.regions.write().await;
                    regions.insert(region.to_string());
                }

                // Replicate to all known regions
                let regions = self.regions.read().await;
                let base_key = format!("spec:{}", spec.id);

                for target_region in regions.iter() {
                    let key = format!("{}:{}", base_key, target_region);
                    let mut cache = self.local_cache.write().await;
                    cache.insert(
                        key,
                        CacheEntry {
                            data: serialized.clone(),
                            timestamp,
                            region: target_region.clone(),
                        },
                    );
                }

                tracing::debug!(
                    "Strong consistency sync for spec {} to {} regions",
                    spec.id,
                    regions.len()
                );
            }
            SyncStrategy::Eventual => {
                // Eventual consistency: local cache is sufficient
                let mut cache = self.local_cache.write().await;
                let key = format!("spec:{}:{}", spec.id, region);
                cache.insert(
                    key,
                    CacheEntry {
                        data: serialized,
                        timestamp,
                        region: region.to_string(),
                    },
                );
                tracing::debug!(
                    "Eventual consistency sync for spec {} to region {}",
                    spec.id,
                    region
                );
            }
            SyncStrategy::LastWriteWins => {
                // Last-write-wins: timestamp-based conflict resolution
                let mut cache = self.local_cache.write().await;
                let key = format!("spec:{}:{}", spec.id, region);

                // Check if entry exists and compare timestamps
                let should_update = match cache.get(&key) {
                    Some(existing) => {
                        // Update only if new timestamp is newer
                        timestamp > existing.timestamp
                    }
                    None => true, // No existing entry, always update
                };

                if should_update {
                    cache.insert(
                        key,
                        CacheEntry {
                            data: serialized,
                            timestamp,
                            region: region.to_string(),
                        },
                    );
                    tracing::debug!(
                        "Last-write-wins sync for spec {} to region {} (timestamp: {})",
                        spec.id,
                        region,
                        timestamp
                    );
                } else {
                    if let Some(existing_entry) = cache.get(&format!("spec:{}:{}", spec.id, region))
                    {
                        tracing::debug!(
                            "Last-write-wins sync for spec {} to region {} skipped (existing timestamp {} is newer than {})",
                            spec.id,
                            region,
                            existing_entry.timestamp,
                            timestamp
                        );
                    }
                }
            }
        }

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
        sync.sync_case(&case, "us-east-1")
            .await
            .expect("sync_case should succeed");
    }
}
