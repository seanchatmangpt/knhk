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
        // Phase 8 implementation: Region addition
        let region_id = region.region_id.clone();

        // Step 1: Validate region configuration
        if region_id.is_empty() {
            return Err(ReplicationError::ReplicationFailed(
                "Region ID cannot be empty".to_string()
            ));
        }

        if region.leader.is_empty() {
            return Err(ReplicationError::ReplicationFailed(
                "Region must have a leader".to_string()
            ));
        }

        // Check for duplicate region
        if self.regions.contains_key(&region_id) {
            return Err(ReplicationError::ReplicationFailed(
                format!("Region {} already exists", region_id)
            ));
        }

        // Step 2: Initialize replication state for new region
        let mut state = MultiRegionState::new();

        // If there's a primary region, sync initial state from it
        if let Some(primary_state) = self.region_state.get(&self.primary_region) {
            state.replicated_index = primary_state.replicated_index;
            state.committed_index = primary_state.committed_index;
            // Step 3: Would send initial replication checkpoint in production
            tracing::debug!(
                "Replication manager: syncing region {} from primary (index: {})",
                region_id,
                state.replicated_index
            );
        }

        self.regions.insert(region_id.clone(), region);
        self.region_state.insert(region_id.clone(), state);

        tracing::info!(
            "Replication manager: added region {} with {} replicas",
            region_id,
            self.regions.get(&region_id).map(|r| r.replicas.len()).unwrap_or(0)
        );

        Ok(())
    }

    /// Replicate entry to all regions
    pub fn replicate_entry(&mut self, entry: Vec<u8>) -> Result<(), ReplicationError> {
        // Phase 8 implementation: Entry replication

        // Step 1: Add entry to primary region pending queue
        if let Some(primary_state) = self.region_state.get_mut(&self.primary_region) {
            primary_state.pending_entries.push(entry.clone());
        }

        // Step 2: Send ReplicateRPC to all regions
        // (In production: async RPC calls to each region's leader)
        let mut regions_replicated = 0;
        for (region_id, state) in self.region_state.iter_mut() {
            state.pending_entries.push(entry.clone());
            regions_replicated += 1;

            tracing::trace!(
                "Replication manager: sent entry to region {} (pending: {})",
                region_id,
                state.pending_entries.len()
            );
        }

        // Step 3: Check if consistency level is satisfied
        let required_acks = match self.consistency_level {
            ConsistencyLevel::Local => 1,
            ConsistencyLevel::Majority => (self.regions.len() / 2) + 1,
            ConsistencyLevel::AllRegions => self.regions.len(),
        };

        // In production: would wait for async RPC responses
        let acks_received = regions_replicated; // Assume immediate ack for now

        if acks_received < required_acks {
            return Err(ReplicationError::ReplicationFailed(
                format!(
                    "Insufficient replication: got {} acks, need {} (consistency: {:?})",
                    acks_received, required_acks, self.consistency_level
                )
            ));
        }

        // Step 4: Update replicated_index when consensus reached
        for state in self.region_state.values_mut() {
            state.replicated_index += 1;
        }

        tracing::trace!(
            "Replication manager: entry replicated to {}/{} regions (required: {})",
            acks_received,
            self.regions.len(),
            required_acks
        );

        Ok(())
    }

    /// Commit replicated entries
    pub fn commit_replicated(&mut self, up_to_index: u64) -> Result<(), ReplicationError> {
        // Phase 8 implementation: Replicated commit

        // Step 1: Check consistency level is satisfied
        // Count regions that have replicated up to this index
        let regions_caught_up = self.region_state.iter()
            .filter(|(_, state)| state.replicated_index >= up_to_index)
            .count();

        let required_acks = match self.consistency_level {
            ConsistencyLevel::Local => 1,
            ConsistencyLevel::Majority => (self.regions.len() / 2) + 1,
            ConsistencyLevel::AllRegions => self.regions.len(),
        };

        if regions_caught_up < required_acks {
            return Err(ReplicationError::ReplicationFailed(
                format!(
                    "Insufficient acknowledgments for commit: {}/{} regions (need {}, level: {:?})",
                    regions_caught_up, self.regions.len(), required_acks, self.consistency_level
                )
            ));
        }

        // Step 2: Update committed_index for all regions that have the data
        let mut committed_regions = 0;
        for (region_id, state) in self.region_state.iter_mut() {
            if state.replicated_index >= up_to_index {
                state.committed_index = up_to_index;
                committed_regions += 1;

                tracing::trace!(
                    "Replication manager: committed index {} in region {}",
                    up_to_index,
                    region_id
                );
            }
        }

        // Step 3: Clear pending entries up to committed index
        // (In production: would clear only committed entries, keep pending)
        for state in self.region_state.values_mut() {
            if state.committed_index == up_to_index {
                let entries_to_keep = state.pending_entries.len()
                    .saturating_sub(up_to_index as usize);
                state.pending_entries.drain(0..state.pending_entries.len() - entries_to_keep);
            }
        }

        tracing::info!(
            "Replication manager: committed index {} across {}/{} regions (consistency: {:?})",
            up_to_index,
            committed_regions,
            self.regions.len(),
            self.consistency_level
        );

        Ok(())
    }

    /// Get replication lag for a region
    pub fn get_replication_lag(&self, region_id: &str) -> Result<u64, ReplicationError> {
        // Phase 8 implementation: Calculate replication lag

        // Step 1: Get region's replicated_index
        let state = self.region_state.get(region_id)
            .ok_or_else(|| ReplicationError::RegionNotFound(region_id.to_string()))?;

        // Step 2: Compare to primary's replicated_index
        let primary_state = self.region_state.get(&self.primary_region)
            .ok_or_else(|| ReplicationError::RegionNotFound(self.primary_region.clone()))?;

        // Step 3: Calculate and return difference
        // Lag = how many entries behind the primary this region is
        let lag = primary_state.replicated_index.saturating_sub(state.replicated_index);

        tracing::trace!(
            "Replication lag for region {}: {} entries (primary: {}, region: {})",
            region_id,
            lag,
            primary_state.replicated_index,
            state.replicated_index
        );

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
        // Phase 8 implementation: Consistency verification

        if self.region_state.is_empty() {
            return Ok(true); // No regions to check
        }

        // Get baseline from first region
        let baseline = self.region_state.values().next().unwrap();

        // Step 1: Check all regions have same committed_index
        let all_same_committed = self.region_state.values()
            .all(|state| state.committed_index == baseline.committed_index);

        if !all_same_committed {
            tracing::warn!(
                "Replication consistency violation: committed_index mismatch across regions"
            );
            return Err(ReplicationError::ConsistencyViolation);
        }

        // Step 2: Verify no diverging log entries
        // Check that replicated indices are within acceptable range
        let max_replicated = self.region_state.values()
            .map(|s| s.replicated_index)
            .max()
            .unwrap_or(0);

        let min_replicated = self.region_state.values()
            .map(|s| s.replicated_index)
            .min()
            .unwrap_or(0);

        // Allow some divergence in replicated_index (replication in progress)
        let max_divergence = 1000; // entries
        if max_replicated - min_replicated > max_divergence {
            tracing::warn!(
                "Replication consistency warning: excessive divergence ({} entries)",
                max_replicated - min_replicated
            );
        }

        // Step 3: Check replication lag within tolerance for all regions
        let mut violations = Vec::new();
        for (region_id, region_config) in &self.regions {
            if let Ok(lag) = self.get_replication_lag(region_id) {
                if lag > region_config.lag_tolerance {
                    violations.push((region_id.clone(), lag, region_config.lag_tolerance));
                    tracing::warn!(
                        "Region {} exceeds lag tolerance: {} > {}",
                        region_id,
                        lag,
                        region_config.lag_tolerance
                    );
                }
            }
        }

        if !violations.is_empty() {
            return Err(ReplicationError::ConsistencyViolation);
        }

        tracing::debug!(
            "Replication consistency verified: {} regions consistent (committed_index: {})",
            self.region_state.len(),
            baseline.committed_index
        );

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
