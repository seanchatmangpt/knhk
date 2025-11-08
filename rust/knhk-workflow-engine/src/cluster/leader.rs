#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Leader election for distributed workflow engine

use crate::error::{WorkflowError, WorkflowResult};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

/// Leader state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderState {
    /// This node is the leader
    Leader,
    /// This node is a follower
    Follower,
    /// Leader election in progress
    Electing,
}

/// Leader election manager
pub struct LeaderElection {
    node_id: String,
    state: Arc<Mutex<LeaderState>>,
    leader_id: Arc<Mutex<Option<String>>>,
    lease_duration: Duration,
    last_heartbeat: Arc<Mutex<Option<u64>>>, // Store as timestamp instead of Instant
}

impl LeaderElection {
    /// Create a new leader election manager
    pub fn new(node_id: String, lease_duration_secs: u64) -> Self {
        Self {
            node_id,
            state: Arc::new(Mutex::new(LeaderState::Follower)),
            leader_id: Arc::new(Mutex::new(None)),
            lease_duration: Duration::from_secs(lease_duration_secs),
            last_heartbeat: Arc::new(Mutex::new(None)),
        }
    }

    /// Check if this node is the leader
    pub fn is_leader(&self) -> bool {
        let state = self.state.lock().unwrap();
        *state == LeaderState::Leader
    }

    /// Get current leader ID
    pub fn get_leader(&self) -> Option<String> {
        let leader_id = self.leader_id.lock().unwrap();
        leader_id.clone()
    }

    /// Attempt to become leader
    pub fn try_become_leader(&self) -> WorkflowResult<bool> {
        let mut state = self.state.lock().map_err(|e| {
            WorkflowError::Internal(format!("Failed to acquire leader lock: {}", e))
        })?;

        // Check if leader lease is expired
        let leader_expired = {
            let last_heartbeat = self.last_heartbeat.lock().unwrap();
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            last_heartbeat
                .map(|hb| now_ms.saturating_sub(hb) >= self.lease_duration.as_millis() as u64)
                .unwrap_or(true)
        };

        if leader_expired || self.get_leader().is_none() {
            *state = LeaderState::Leader;
            let mut leader_id = self.leader_id.lock().unwrap();
            *leader_id = Some(self.node_id.clone());
            let mut heartbeat = self.last_heartbeat.lock().unwrap();
            *heartbeat = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            );
            Ok(true)
        } else {
            *state = LeaderState::Follower;
            Ok(false)
        }
    }

    /// Renew leader lease
    pub fn renew_lease(&self) -> WorkflowResult<()> {
        if !self.is_leader() {
            return Err(WorkflowError::Internal("Not the leader".to_string()));
        }

        let mut heartbeat = self.last_heartbeat.lock().map_err(|e| {
            WorkflowError::Internal(format!("Failed to acquire heartbeat lock: {}", e))
        })?;
        *heartbeat = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        );
        Ok(())
    }

    /// Step down from leadership
    pub fn step_down(&self) -> WorkflowResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            WorkflowError::Internal(format!("Failed to acquire leader lock: {}", e))
        })?;
        *state = LeaderState::Follower;
        let mut leader_id = self.leader_id.lock().unwrap();
        *leader_id = None;
        Ok(())
    }

    /// Get node ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leader_election() {
        let election = LeaderElection::new("node-1".to_string(), 30);
        assert!(!election.is_leader());

        let became_leader = election.try_become_leader().unwrap();
        assert!(became_leader);
        assert!(election.is_leader());
        assert_eq!(election.get_leader(), Some("node-1".to_string()));
    }
}
