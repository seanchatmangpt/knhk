//! Leader implementation for Raft consensus
//!
//! The leader:
//! - Accepts client requests and appends them to the log
//! - Replicates log entries to followers via AppendEntries RPCs
//! - Sends periodic heartbeats to maintain authority
//! - Commits entries when replicated to majority
//! - Applies committed entries to the state machine

use super::*;
use std::collections::HashMap;
use tokio::time::{interval, Duration};

/// Leader state
pub struct Leader {
    /// Node ID
    node_id: NodeId,

    /// Current term
    term: Term,

    /// Next index to send to each follower
    next_index: HashMap<NodeId, LogIndex>,

    /// Highest index replicated to each follower
    match_index: HashMap<NodeId, LogIndex>,

    /// Heartbeat interval
    heartbeat_interval: Duration,

    /// Peers
    peers: Vec<NodeId>,
}

impl Leader {
    /// Create a new leader
    pub fn new(node_id: NodeId, term: Term, last_log_index: LogIndex, peers: Vec<NodeId>) -> Self {
        // Initialize next_index to last_log_index + 1 for all followers
        let mut next_index = HashMap::new();
        let mut match_index = HashMap::new();

        for peer in &peers {
            next_index.insert(*peer, LogIndex::new(last_log_index.inner() + 1));
            match_index.insert(*peer, LogIndex::new(0));
        }

        Self {
            node_id,
            term,
            next_index,
            match_index,
            heartbeat_interval: Duration::from_millis(50),
            peers,
        }
    }

    /// Send heartbeats to all followers
    pub async fn send_heartbeats(&self, log: &ReplicatedLog) -> ConsensusResult<()> {
        debug!(
            node_id = ?self.node_id,
            term = ?self.term,
            peers = ?self.peers.len(),
            "Sending heartbeats"
        );

        // In a real implementation, this would:
        // 1. For each follower, send AppendEntries RPC (possibly with new entries)
        // 2. Handle responses and update next_index/match_index
        // 3. Commit entries when replicated to majority

        Ok(())
    }

    /// Replicate log entries to followers
    pub async fn replicate_log(
        &mut self,
        log: &ReplicatedLog,
        commit_index: LogIndex,
    ) -> ConsensusResult<LogIndex> {
        // For each follower, send AppendEntries with new entries
        for peer in &self.peers {
            let next_idx = self.next_index.get(peer).copied().unwrap_or(LogIndex::new(1));

            // Get entries starting from next_index
            let entries = log.get_entries_from(next_idx);

            if !entries.is_empty() {
                debug!(
                    node_id = ?self.node_id,
                    peer = ?peer,
                    next_index = ?next_idx,
                    num_entries = entries.len(),
                    "Replicating entries to follower"
                );

                // In a real implementation:
                // 1. Send AppendEntries RPC with entries
                // 2. Handle response:
                //    - If success: update next_index and match_index
                //    - If failure: decrement next_index and retry
            }
        }

        // Calculate new commit index (median of match_index)
        let mut match_indices: Vec<LogIndex> = self.match_index.values().copied().collect();
        match_indices.push(log.last_index()); // Add our own index
        match_indices.sort();

        // Commit index is the highest index replicated to majority
        let majority_idx = match_indices.len() / 2;
        let new_commit_index = match_indices[majority_idx];

        Ok(new_commit_index.max(commit_index))
    }

    /// Handle AppendEntries response from follower
    pub fn handle_append_entries_response(
        &mut self,
        follower: NodeId,
        success: bool,
        match_index: LogIndex,
    ) {
        if success {
            // Update next_index and match_index
            self.next_index.insert(follower, LogIndex::new(match_index.inner() + 1));
            self.match_index.insert(follower, match_index);

            debug!(
                node_id = ?self.node_id,
                follower = ?follower,
                match_index = ?match_index,
                "AppendEntries succeeded"
            );
        } else {
            // Decrement next_index and retry
            let next_idx = self.next_index.get(&follower).copied().unwrap_or(LogIndex::new(1));
            if next_idx.inner() > 1 {
                self.next_index.insert(follower, LogIndex::new(next_idx.inner() - 1));
            }

            debug!(
                node_id = ?self.node_id,
                follower = ?follower,
                next_index = ?next_idx,
                "AppendEntries failed, decrementing next_index"
            );
        }
    }

    /// Get current commit index based on match_index
    pub fn calculate_commit_index(&self, current_commit: LogIndex) -> LogIndex {
        let mut match_indices: Vec<u64> = self.match_index.values().map(|idx| idx.inner()).collect();
        match_indices.sort_unstable();

        // Commit index is the median (majority) match index
        if match_indices.is_empty() {
            current_commit
        } else {
            let majority_idx = match_indices.len() / 2;
            LogIndex::new(match_indices[majority_idx]).max(current_commit)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leader_creation() {
        let peers = vec![NodeId::new(2), NodeId::new(3)];
        let leader = Leader::new(NodeId::new(1), Term::new(1), LogIndex::new(5), peers);

        assert_eq!(leader.term.inner(), 1);
        assert_eq!(leader.next_index.len(), 2);
        assert_eq!(leader.next_index.get(&NodeId::new(2)).unwrap().inner(), 6);
    }

    #[test]
    fn test_leader_append_entries_response() {
        let peers = vec![NodeId::new(2), NodeId::new(3)];
        let mut leader = Leader::new(NodeId::new(1), Term::new(1), LogIndex::new(5), peers);

        // Success case
        leader.handle_append_entries_response(NodeId::new(2), true, LogIndex::new(10));
        assert_eq!(leader.match_index.get(&NodeId::new(2)).unwrap().inner(), 10);
        assert_eq!(leader.next_index.get(&NodeId::new(2)).unwrap().inner(), 11);

        // Failure case
        leader.handle_append_entries_response(NodeId::new(3), false, LogIndex::new(0));
        assert_eq!(leader.next_index.get(&NodeId::new(3)).unwrap().inner(), 5); // Decremented from 6
    }

    #[test]
    fn test_leader_calculate_commit_index() {
        let peers = vec![NodeId::new(2), NodeId::new(3), NodeId::new(4)];
        let mut leader = Leader::new(NodeId::new(1), Term::new(1), LogIndex::new(10), peers);

        // Set match indices
        leader.match_index.insert(NodeId::new(2), LogIndex::new(10));
        leader.match_index.insert(NodeId::new(3), LogIndex::new(8));
        leader.match_index.insert(NodeId::new(4), LogIndex::new(7));

        let commit_index = leader.calculate_commit_index(LogIndex::new(5));
        assert_eq!(commit_index.inner(), 8); // Median of [7, 8, 10]
    }
}
