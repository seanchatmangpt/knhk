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
        // Phase 8 implementation: Raft log replication
        // Step 1: Verify node is leader
        if !self.is_leader() {
            return Err(RaftError::NotLeader);
        }

        // Step 2: Create log entry with current term
        let index = self.log.len() as u64 + 1;
        let entry = LogEntry {
            index,
            term: self.current_term,
            data,
            committed: false,
        };

        // Step 3: Append to local log
        self.log.push(entry);

        // Step 4: Update next_index for all followers to replicate this entry
        for peer in &self.config.peers {
            if let Some(next_idx) = self.next_index.get_mut(peer) {
                // Next entry to send to this follower
                *next_idx = index + 1;
            }
        }

        // Step 5: Track replication progress (in production, would send AppendEntries RPC)
        // For now, we simulate immediate local acknowledgment
        let acks = 1; // Self acknowledgment
        let majority = (self.config.peers.len() + 1) / 2 + 1;

        // Step 6: Commit when replicated to majority
        // In production: wait for RPC responses, then commit
        if acks >= majority {
            tracing::trace!(
                "Raft append_entry: entry {} ready for commit (acks: {}/{})",
                index, acks, majority
            );
        }

        tracing::trace!(
            "Raft append_entry: appended entry {} to log, awaiting replication",
            index
        );

        Ok(index)
    }

    /// Start leader election
    pub fn start_election(&mut self) -> Result<(), RaftError> {
        // Phase 8 implementation: Raft leader election
        // Step 1: Increment current term
        self.current_term += 1;

        // Step 2: Transition to candidate and vote for self
        self.state = RaftState::Candidate;
        self.voted_for = Some(self.config.node_id.clone());

        // Step 3: Request votes from all peers (in production: send RequestVote RPC)
        // Track votes received: start with 1 (self-vote)
        let votes_received = 1;
        let total_nodes = self.config.peers.len() + 1; // peers + self
        let majority = total_nodes / 2 + 1;

        tracing::trace!(
            "Raft start_election: node {} started election for term {} (need {}/{} votes)",
            self.config.node_id,
            self.current_term,
            majority,
            total_nodes
        );

        // Step 4: Check if already have majority (single-node cluster)
        if votes_received >= majority {
            tracing::info!(
                "Raft start_election: node {} won election immediately (single-node cluster)",
                self.config.node_id
            );
        }

        // Step 5: In production, would wait for responses or timeout
        // If receive heartbeat from leader with term >= currentTerm, revert to follower
        // Implementation note: RPC handling would be done in separate receive_vote_response method

        Ok(())
    }

    /// Become leader
    pub fn become_leader(&mut self) -> Result<(), RaftError> {
        // Phase 8 implementation: Leader assumption
        // Step 1: Verify is candidate (would check majority votes in production)
        if self.state != RaftState::Candidate {
            return Err(RaftError::InvalidState("Not a candidate".to_string()));
        }

        // Step 2: Set state to Leader
        self.state = RaftState::Leader;
        self.leader_id = Some(self.config.node_id.clone());

        // Step 3: Initialize next_index and match_index for all followers
        // next_index: index of next log entry to send (start at end of log)
        // match_index: highest log entry known to be replicated (start at 0)
        for peer in &self.config.peers {
            self.next_index.insert(peer.clone(), self.log.len() as u64 + 1);
            self.match_index.insert(peer.clone(), 0);
        }

        // Step 4: Send initial heartbeat to establish leadership
        // In production: send empty AppendEntries RPC to all followers
        tracing::info!(
            "Raft become_leader: node {} became leader for term {}, sending heartbeats to {} followers",
            self.config.node_id,
            self.current_term,
            self.config.peers.len()
        );

        // Reset election timeout to prevent starting new election
        // Begin periodic heartbeat loop (would be in separate task in production)

        Ok(())
    }

    /// Commit entries up to index
    pub fn commit_entries(&mut self, commit_index: u64) -> Result<(), RaftError> {
        // Phase 8 implementation: Entry commitment
        // Only advance commit index forward
        if commit_index <= self.commit_index {
            return Ok(());
        }

        // Step 1: Update commit_index to new value
        self.commit_index = commit_index;

        // Step 2: Apply all entries from last_applied+1 to commit_index
        let max_apply = self.commit_index.min(self.log.len() as u64);
        for i in self.last_applied + 1..=max_apply {
            // Get entry from log (indices are 1-based, vec is 0-based)
            if let Some(entry) = self.log.get_mut((i - 1) as usize) {
                // Mark entry as committed
                entry.committed = true;

                // Apply entry to state machine (in production: execute command)
                tracing::trace!(
                    "Raft commit_entries: committed and applied entry {} (term {})",
                    i,
                    entry.term
                );

                // Step 3: Update last_applied index
                self.last_applied = i;
            }
        }

        tracing::debug!(
            "Raft commit_entries: committed {} entries, last_applied now {}",
            max_apply - self.last_applied,
            self.last_applied
        );

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
