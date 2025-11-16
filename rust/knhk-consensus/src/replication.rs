// Multi-Region Replication Management
// Handles data replication across geographic regions for high availability

use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum ReplicationError {
    #[error("Region not found: {0}")]
    RegionNotFound(String),

    #[error("Replication failed: {0}")]
    ReplicationFailed(String),

    #[error("Consistency violation")]
    ConsistencyViolation,

    #[error("Region unavailable")]
    RegionUnavailable,
}

/// Region configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegionConfig {
    /// Region identifier
    pub region_id: String,
    /// Leader node in this region
    pub leader: String,
    /// Replica nodes in this region
    pub replicas: Vec<String>,
    /// Network latency to primary region (ms)
    pub latency_ms: u64,
    /// Replication lag tolerance (entries)
    pub lag_tolerance: u64,
}

impl RegionConfig {
    /// Create a new region configuration
    pub fn new(region_id: String, leader: String) -> Self {
        Self {
            region_id,
            leader,
            replicas: vec![],
            latency_ms: 0,
            lag_tolerance: 100,
        }
    }

    /// Add replica to region
    pub fn add_replica(&mut self, replica: String) {
        self.replicas.push(replica);
    }
}

/// Multi-region state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiRegionState {
    /// Log index of last replicated entry
    pub replicated_index: u64,
    /// Committed index in this region
    pub committed_index: u64,
    /// Pending entries awaiting replication
    pub pending_entries: Vec<Vec<u8>>,
}

impl MultiRegionState {
    /// Create new multi-region state
    pub fn new() -> Self {
        Self {
            replicated_index: 0,
            committed_index: 0,
            pending_entries: vec![],
        }
    }
}

impl Default for MultiRegionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Replication manager for multi-region deployments
#[derive(Debug)]
pub struct ReplicationManager {
    /// Primary region
    pub primary_region: String,
    /// All regions in deployment
    pub regions: HashMap<String, RegionConfig>,
    /// Per-region state
    pub region_state: HashMap<String, MultiRegionState>,
    /// Consistency level (local, majority, all)
    pub consistency_level: ConsistencyLevel,
}

/// Consistency level for multi-region writes
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    /// Only commit to primary region
    Local,
    /// Commit when majority of regions acknowledge
    Majority,
    /// Commit only when all regions acknowledge
    AllRegions,
}

impl ReplicationManager {
    /// Create a new replication manager
    pub fn new(primary_region: String) -> Self {
        Self {
            primary_region,
            regions: HashMap::new(),
            region_state: HashMap::new(),
            consistency_level: ConsistencyLevel::Majority,
        }
    }

    /// Add a region
    pub fn add_region(&mut self, region: RegionConfig) -> Result<(), ReplicationError> {
        // Phase 8 implementation stub
        // TODO: Implement region addition
        // Step 1: Validate region configuration
        // Step 2: Initialize replication state
        // Step 3: Send initial replication checkpoint

        let region_id = region.region_id.clone();
        self.regions.insert(region_id.clone(), region);
        self.region_state.insert(region_id, MultiRegionState::new());

        tracing::info!("Replication manager: added region");

        Ok(())
    }

    /// Replicate entry to all regions
    pub fn replicate_entry(&mut self, entry: Vec<u8>) -> Result<(), ReplicationError> {
        // Phase 8 implementation stub
        // TODO: Implement entry replication
        // Step 1: Add entry to primary region pending
        // Step 2: Send ReplicateRPC to all regions
        // Step 3: Wait for acknowledgments based on consistency level
        // Step 4: Update replicated_index when consensus reached

        for state in self.region_state.values_mut() {
            state.pending_entries.push(entry.clone());
        }

        tracing::trace!("Replication manager: replicated entry to all regions");

        Ok(())
    }

    /// Commit replicated entries
    pub fn commit_replicated(&mut self, up_to_index: u64) -> Result<(), ReplicationError> {
        // Phase 8 implementation stub
        // TODO: Implement replicated commit
        // Step 1: Check consistency level is satisfied
        // Step 2: Update committed_index for all regions
        // Step 3: Mark entries as globally committed

        let required_acks = match self.consistency_level {
            ConsistencyLevel::Local => 1,
            ConsistencyLevel::Majority => (self.regions.len() / 2) + 1,
            ConsistencyLevel::AllRegions => self.regions.len(),
        };

        let acknowledged = self.regions.len(); // Stub: assume all acked

        if acknowledged < required_acks {
            return Err(ReplicationError::ReplicationFailed(
                format!("Insufficient acknowledgments: {} < {}", acknowledged, required_acks)
            ));
        }

        for state in self.region_state.values_mut() {
            state.committed_index = up_to_index;
            state.pending_entries.clear();
        }

        tracing::info!(
            "Replication manager: committed up to index {} across all regions",
            up_to_index
        );

        Ok(())
    }

    /// Get replication lag for a region
    pub fn get_replication_lag(&self, region_id: &str) -> Result<u64, ReplicationError> {
        // Phase 8 implementation stub
        // TODO: Calculate replication lag
        // Step 1: Get region's replicated_index
        // Step 2: Compare to primary's committed_index
        // Step 3: Return difference

        let state = self.region_state.get(region_id)
            .ok_or_else(|| ReplicationError::RegionNotFound(region_id.to_string()))?;

        let primary_state = self.region_state.get(&self.primary_region)
            .ok_or_else(|| ReplicationError::RegionNotFound(self.primary_region.clone()))?;

        let lag = primary_state.replicated_index.saturating_sub(state.replicated_index);

        Ok(lag)
    }

    /// Check if region is caught up
    pub fn is_region_caught_up(&self, region_id: &str) -> Result<bool, ReplicationError> {
        let lag = self.get_replication_lag(region_id)?;
        let tolerance = self.regions.get(region_id)
            .map(|r| r.lag_tolerance)
            .unwrap_or(100);

        Ok(lag <= tolerance)
    }

    /// Verify consistency across regions
    pub fn verify_consistency(&self) -> Result<bool, ReplicationError> {
        // Phase 8 implementation stub
        // TODO: Implement consistency verification
        // Step 1: Check all regions have same committed_index
        // Step 2: Verify no diverging log entries
        // Step 3: Check replication lag within tolerance

        let all_same = self.region_state.values()
            .all(|state| state.committed_index == self.region_state.values().next().unwrap().committed_index);

        if !all_same {
            return Err(ReplicationError::ConsistencyViolation);
        }

        Ok(true)
    }

    /// Get replication status summary
    pub fn get_status(&self) -> ReplicationStatus {
        let total_regions = self.regions.len();
        let healthy_regions = self.regions.iter()
            .filter(|(id, _)| self.is_region_caught_up(id).unwrap_or(false))
            .count();

        ReplicationStatus {
            total_regions,
            healthy_regions,
            consistency_level: self.consistency_level,
            primary_region: self.primary_region.clone(),
        }
    }
}

/// Replication status summary
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplicationStatus {
    pub total_regions: usize,
    pub healthy_regions: usize,
    pub consistency_level: ConsistencyLevel,
    pub primary_region: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_config_creation() {
        let mut region = RegionConfig::new("us-east-1".to_string(), "leader1".to_string());
        region.add_replica("replica1".to_string());
        assert_eq!(region.replicas.len(), 1);
    }

    #[test]
    fn test_multi_region_state() {
        let state = MultiRegionState::new();
        assert_eq!(state.replicated_index, 0);
        assert_eq!(state.committed_index, 0);
    }

    #[test]
    fn test_replication_manager_creation() {
        let manager = ReplicationManager::new("primary".to_string());
        assert_eq!(manager.primary_region, "primary");
        assert_eq!(manager.regions.len(), 0);
    }

    #[test]
    fn test_add_region() {
        let mut manager = ReplicationManager::new("primary".to_string());
        let region = RegionConfig::new("us-east-1".to_string(), "leader1".to_string());
        let result = manager.add_region(region);
        assert!(result.is_ok());
        assert_eq!(manager.regions.len(), 1);
    }

    #[test]
    fn test_replicate_entry() {
        let mut manager = ReplicationManager::new("primary".to_string());
        let region = RegionConfig::new("us-east-1".to_string(), "leader1".to_string());
        manager.add_region(region).unwrap();

        let result = manager.replicate_entry(vec![1, 2, 3]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_consistency_levels() {
        assert_eq!(ConsistencyLevel::Local, ConsistencyLevel::Local);
        assert_ne!(ConsistencyLevel::Local, ConsistencyLevel::Majority);
    }

    #[test]
    fn test_replication_status() {
        let manager = ReplicationManager::new("primary".to_string());
        let status = manager.get_status();
        assert_eq!(status.primary_region, "primary");
        assert_eq!(status.total_regions, 0);
    }
}
