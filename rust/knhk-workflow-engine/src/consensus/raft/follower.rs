//! Follower and Candidate implementations for Raft consensus
//!
//! Followers are passive - they respond to RPCs from leaders and candidates.
//! Candidates compete for leadership during elections.

use super::*;
use fastrand::Rng;
use std::time::Instant;
use tokio::time::Duration;

/// Follower state
pub struct Follower {
    /// Node ID
    node_id: NodeId,

    /// Current term
    term: Term,

    /// Who we voted for in current term
    voted_for: Option<NodeId>,

    /// Current leader (if known)
    current_leader: Option<NodeId>,

    /// Last time we heard from leader
    last_heartbeat: Instant,

    /// Election timeout
    election_timeout: Duration,

    /// Random number generator for election timeout
    rng: Rng,
}

impl Follower {
    /// Create a new follower
    pub fn new(node_id: NodeId, term: Term) -> Self {
        Self {
            node_id,
            term,
            voted_for: None,
            current_leader: None,
            last_heartbeat: Instant::now(),
            election_timeout: Duration::from_millis(150 + fastrand::u64(0..150)),
            rng: Rng::new(),
        }
    }

    /// Check if election timeout has elapsed
    pub fn is_election_timeout(&self) -> bool {
        self.last_heartbeat.elapsed() > self.election_timeout
    }

    /// Reset election timeout (after receiving heartbeat)
    pub fn reset_election_timeout(&mut self) {
        self.last_heartbeat = Instant::now();
        // Randomize timeout for next election
        let random_ms = self.rng.u64(0..150);
        self.election_timeout = Duration::from_millis(150 + random_ms);
    }

    /// Handle AppendEntries RPC from leader
    pub fn handle_append_entries(
        &mut self,
        request: &AppendEntriesRequest,
        log: &mut ReplicatedLog,
    ) -> AppendEntriesResponse {
        // Reset election timeout - we heard from a leader
        self.reset_election_timeout();
        self.current_leader = Some(request.leader_id);

        // Reply false if term < currentTerm
        if request.term < self.term {
            return AppendEntriesResponse {
                term: self.term,
                success: false,
                match_index: LogIndex::new(0),
            };
        }

        // Update term if necessary
        if request.term > self.term {
            self.term = request.term;
            self.voted_for = None;
        }

        // Reply false if log doesn't contain entry at prev_log_index with prev_log_term
        if !log.matches_prev_log(request.prev_log_index, request.prev_log_term) {
            return AppendEntriesResponse {
                term: self.term,
                success: false,
                match_index: LogIndex::new(0),
            };
        }

        // If an existing entry conflicts with a new one, delete the existing entry and all that follow
        for entry in &request.entries {
            if let Some(existing) = log.get(entry.index) {
                if existing.term != entry.term {
                    log.truncate_from(entry.index);
                    break;
                }
            }
        }

        // Append any new entries
        log.append_entries(request.entries.clone());

        // Success
        let match_index = log.last_index();
        AppendEntriesResponse {
            term: self.term,
            success: true,
            match_index,
        }
    }

    /// Convert to candidate (start election)
    pub fn become_candidate(self) -> Candidate {
        Candidate::new(self.node_id, Term::new(self.term.inner() + 1))
    }
}

/// Candidate state
pub struct Candidate {
    /// Node ID
    node_id: NodeId,

    /// Current term
    term: Term,

    /// Votes received
    votes_received: Vec<NodeId>,

    /// Election start time
    election_start: Instant,

    /// Election timeout
    election_timeout: Duration,
}

impl Candidate {
    /// Create a new candidate
    pub fn new(node_id: NodeId, term: Term) -> Self {
        Self {
            node_id,
            term,
            votes_received: vec![node_id], // Vote for self
            election_start: Instant::now(),
            election_timeout: Duration::from_millis(150 + fastrand::u64(0..150)),
        }
    }

    /// Request vote from a peer
    pub fn request_vote(&self, last_log_index: LogIndex, last_log_term: Term) -> RequestVoteRequest {
        RequestVoteRequest {
            term: self.term,
            candidate_id: self.node_id,
            last_log_index,
            last_log_term,
        }
    }

    /// Handle RequestVote response
    pub fn handle_vote_response(&mut self, voter: NodeId, granted: bool) {
        if granted {
            if !self.votes_received.contains(&voter) {
                self.votes_received.push(voter);
            }
        }
    }

    /// Check if we have majority votes
    pub fn has_majority(&self, total_nodes: usize) -> bool {
        self.votes_received.len() >= (total_nodes / 2 + 1)
    }

    /// Check if election timeout has elapsed
    pub fn is_election_timeout(&self) -> bool {
        self.election_start.elapsed() > self.election_timeout
    }

    /// Convert to leader
    pub fn become_leader(self, last_log_index: LogIndex, peers: Vec<NodeId>) -> Leader {
        Leader::new(self.node_id, self.term, last_log_index, peers)
    }

    /// Convert back to follower
    pub fn become_follower(self) -> Follower {
        Follower::new(self.node_id, self.term)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_follower_creation() {
        let follower = Follower::new(NodeId::new(1), Term::new(0));
        assert_eq!(follower.term.inner(), 0);
        assert!(follower.voted_for.is_none());
    }

    #[test]
    fn test_follower_election_timeout() {
        let mut follower = Follower::new(NodeId::new(1), Term::new(0));

        // Should not timeout immediately
        assert!(!follower.is_election_timeout());

        // Reset should update last_heartbeat
        follower.reset_election_timeout();
        assert!(!follower.is_election_timeout());
    }

    #[test]
    fn test_candidate_creation() {
        let candidate = Candidate::new(NodeId::new(1), Term::new(1));
        assert_eq!(candidate.term.inner(), 1);
        assert_eq!(candidate.votes_received.len(), 1); // Voted for self
    }

    #[test]
    fn test_candidate_majority() {
        let mut candidate = Candidate::new(NodeId::new(1), Term::new(1));

        // 3-node cluster: need 2 votes
        assert!(!candidate.has_majority(3)); // Only have 1 vote (self)

        candidate.handle_vote_response(NodeId::new(2), true);
        assert!(candidate.has_majority(3)); // Now have 2 votes

        // 5-node cluster: need 3 votes
        assert!(!candidate.has_majority(5)); // Only have 2 votes

        candidate.handle_vote_response(NodeId::new(3), true);
        assert!(candidate.has_majority(5)); // Now have 3 votes
    }

    #[test]
    fn test_candidate_request_vote() {
        let candidate = Candidate::new(NodeId::new(1), Term::new(1));
        let request = candidate.request_vote(LogIndex::new(10), Term::new(1));

        assert_eq!(request.term.inner(), 1);
        assert_eq!(request.candidate_id.inner(), 1);
        assert_eq!(request.last_log_index.inner(), 10);
        assert_eq!(request.last_log_term.inner(), 1);
    }
}
