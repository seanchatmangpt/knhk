// Raft Consensus Algorithm
// Crash-fault tolerant consensus with leader election and log replication
// Based on Diego Ongaro & John Ousterhout's Raft paper

use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum RaftError {
    #[error("Not a leader")]
    NotLeader,

    #[error("Log entry not found")]
    EntryNotFound,

    #[error("Election timeout")]
    ElectionTimeout,

    #[error("Replication failed: {0}")]
    ReplicationFailed(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

/// Raft node state
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaftState {
    Follower,
    Candidate,
    Leader,
}

/// Log entry in Raft
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub index: u64,
    pub term: u64,
    pub data: Vec<u8>,
    pub committed: bool,
}

/// Raft node configuration
#[derive(Clone, Debug)]
pub struct RaftConfig {
    /// Node identifier
    pub node_id: String,
    /// Total peers (not including self)
    pub peers: Vec<String>,
    /// Election timeout in milliseconds
    pub election_timeout_ms: u64,
    /// Heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,
}

/// Raft consensus node
#[derive(Debug)]
pub struct RaftNode {
    pub config: RaftConfig,
    pub state: RaftState,
    pub current_term: u64,
    pub voted_for: Option<String>,
    pub log: Vec<LogEntry>,
    pub commit_index: u64,
    pub last_applied: u64,
    pub leader_id: Option<String>,
    pub next_index: HashMap<String, u64>,
    pub match_index: HashMap<String, u64>,
}

impl RaftNode {
    /// Create a new Raft node
    pub fn new(config: RaftConfig) -> Self {
        let mut next_index = HashMap::new();
        let mut match_index = HashMap::new();

        for peer in &config.peers {
            next_index.insert(peer.clone(), 1);
            match_index.insert(peer.clone(), 0);
        }

        Self {
            config,
            state: RaftState::Follower,
            current_term: 0,
            voted_for: None,
            log: vec![],
            commit_index: 0,
            last_applied: 0,
            leader_id: None,
            next_index,
            match_index,
        }
    }

    /// Check if this node is a leader
    pub fn is_leader(&self) -> bool {
        self.state == RaftState::Leader
    }

    /// Append entry to log (leader only)
    pub fn append_entry(&mut self, data: Vec<u8>) -> Result<u64, RaftError> {
        // Phase 8 implementation stub
        // TODO: Implement Raft log replication
        // Step 1: Verify node is leader
        // Step 2: Create log entry with current term
        // Step 3: Append to local log
        // Step 4: Send AppendEntries RPC to all followers
        // Step 5: Wait for majority acknowledgment
        // Step 6: Commit when replicated to majority

        if !self.is_leader() {
            return Err(RaftError::NotLeader);
        }

        let index = self.log.len() as u64 + 1;
        let entry = LogEntry {
            index,
            term: self.current_term,
            data,
            committed: false,
        };

        self.log.push(entry);

        tracing::trace!(
            "Raft append_entry: appended entry {} to log",
            index
        );

        Ok(index)
    }

    /// Start leader election
    pub fn start_election(&mut self) -> Result<(), RaftError> {
        // Phase 8 implementation stub
        // TODO: Implement Raft leader election
        // Step 1: Increment current term
        // Step 2: Vote for self
        // Step 3: Send RequestVote RPC to all peers
        // Step 4: If receive majority votes, become leader
        // Step 5: If receive heartbeat from leader with term >= currentTerm, revert to follower

        self.current_term += 1;
        self.state = RaftState::Candidate;
        self.voted_for = Some(self.config.node_id.clone());

        tracing::trace!(
            "Raft start_election: node {} started election for term {}",
            self.config.node_id,
            self.current_term
        );

        Ok(())
    }

    /// Become leader
    pub fn become_leader(&mut self) -> Result<(), RaftError> {
        // Phase 8 implementation stub
        // TODO: Implement leader assumption
        // Step 1: Verify is candidate with majority votes
        // Step 2: Set state to Leader
        // Step 3: Initialize next_index and match_index
        // Step 4: Send initial heartbeat (empty AppendEntries)

        if self.state != RaftState::Candidate {
            return Err(RaftError::InvalidState("Not a candidate".to_string()));
        }

        self.state = RaftState::Leader;
        self.leader_id = Some(self.config.node_id.clone());

        for peer in &self.config.peers {
            self.next_index.insert(peer.clone(), self.log.len() as u64 + 1);
            self.match_index.insert(peer.clone(), 0);
        }

        tracing::info!(
            "Raft become_leader: node {} became leader for term {}",
            self.config.node_id,
            self.current_term
        );

        Ok(())
    }

    /// Commit entries up to index
    pub fn commit_entries(&mut self, commit_index: u64) -> Result<(), RaftError> {
        // Phase 8 implementation stub
        // TODO: Implement entry commitment
        // Step 1: Update commit_index
        // Step 2: Apply entries from last_applied+1 to commit_index
        // Step 3: Update last_applied

        if commit_index > self.commit_index {
            self.commit_index = commit_index;

            for i in self.last_applied + 1..=self.commit_index.min(self.log.len() as u64) {
                if let Some(entry) = self.log.get((i - 1) as usize) {
                    tracing::trace!("Raft commit_entries: committed entry {}", i);
                    self.last_applied = i;
                }
            }
        }

        Ok(())
    }

    /// Get the index of the last log entry
    pub fn last_log_index(&self) -> u64 {
        self.log.len() as u64
    }

    /// Get the term of the last log entry
    pub fn last_log_term(&self) -> u64 {
        self.log.last().map(|e| e.term).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raft_node_creation() {
        let config = RaftConfig {
            node_id: "node1".to_string(),
            peers: vec!["node2".to_string(), "node3".to_string()],
            election_timeout_ms: 150,
            heartbeat_interval_ms: 50,
        };

        let node = RaftNode::new(config.clone());
        assert_eq!(node.state, RaftState::Follower);
        assert_eq!(node.current_term, 0);
        assert!(!node.is_leader());
    }

    #[test]
    fn test_raft_leader_append_entry() {
        let config = RaftConfig {
            node_id: "node1".to_string(),
            peers: vec!["node2".to_string(), "node3".to_string()],
            election_timeout_ms: 150,
            heartbeat_interval_ms: 50,
        };

        let mut node = RaftNode::new(config);
        node.state = RaftState::Leader;

        let result = node.append_entry(b"test data".to_vec());
        assert!(result.is_ok());
    }

    #[test]
    fn test_raft_non_leader_append_fails() {
        let config = RaftConfig {
            node_id: "node1".to_string(),
            peers: vec!["node2".to_string(), "node3".to_string()],
            election_timeout_ms: 150,
            heartbeat_interval_ms: 50,
        };

        let mut node = RaftNode::new(config);
        assert_eq!(node.state, RaftState::Follower);

        let result = node.append_entry(b"test data".to_vec());
        assert!(matches!(result, Err(RaftError::NotLeader)));
    }

    #[test]
    fn test_raft_start_election() {
        let config = RaftConfig {
            node_id: "node1".to_string(),
            peers: vec!["node2".to_string(), "node3".to_string()],
            election_timeout_ms: 150,
            heartbeat_interval_ms: 50,
        };

        let mut node = RaftNode::new(config);
        let result = node.start_election();
        assert!(result.is_ok());
        assert_eq!(node.state, RaftState::Candidate);
        assert_eq!(node.current_term, 1);
    }

    #[test]
    fn test_raft_become_leader() {
        let config = RaftConfig {
            node_id: "node1".to_string(),
            peers: vec!["node2".to_string(), "node3".to_string()],
            election_timeout_ms: 150,
            heartbeat_interval_ms: 50,
        };

        let mut node = RaftNode::new(config);
        node.start_election().unwrap();
        let result = node.become_leader();
        assert!(result.is_ok());
        assert_eq!(node.state, RaftState::Leader);
    }

    #[test]
    fn test_raft_commit_entries() {
        let config = RaftConfig {
            node_id: "node1".to_string(),
            peers: vec!["node2".to_string(), "node3".to_string()],
            election_timeout_ms: 150,
            heartbeat_interval_ms: 50,
        };

        let mut node = RaftNode::new(config);
        node.state = RaftState::Leader;
        node.append_entry(b"entry 1".to_vec()).unwrap();
        node.append_entry(b"entry 2".to_vec()).unwrap();

        let result = node.commit_entries(2);
        assert!(result.is_ok());
        assert_eq!(node.commit_index, 2);
    }

    #[test]
    fn test_raft_log_index_tracking() {
        let config = RaftConfig {
            node_id: "node1".to_string(),
            peers: vec!["node2".to_string(), "node3".to_string()],
            election_timeout_ms: 150,
            heartbeat_interval_ms: 50,
        };

        let mut node = RaftNode::new(config);
        node.state = RaftState::Leader;
        assert_eq!(node.last_log_index(), 0);

        node.append_entry(b"data".to_vec()).unwrap();
        assert_eq!(node.last_log_index(), 1);

        node.append_entry(b"data".to_vec()).unwrap();
        assert_eq!(node.last_log_index(), 2);
    }
}
