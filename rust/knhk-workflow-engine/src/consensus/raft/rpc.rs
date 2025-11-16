//! RPC message types for Raft consensus
//!
//! Raft uses two main RPC types:
//! - **AppendEntries**: Replicate log entries and send heartbeats
//! - **RequestVote**: Request votes during leader election

use super::*;
use serde::{Deserialize, Serialize};

/// AppendEntries RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesRequest {
    /// Leader's term
    pub term: Term,

    /// Leader's node ID
    pub leader_id: NodeId,

    /// Index of log entry immediately preceding new ones
    pub prev_log_index: LogIndex,

    /// Term of prev_log_index entry
    pub prev_log_term: Term,

    /// Log entries to store (empty for heartbeat)
    pub entries: Vec<LogEntry>,

    /// Leader's commit index
    pub leader_commit: LogIndex,
}

impl AppendEntriesRequest {
    /// Create a heartbeat (empty AppendEntries)
    pub fn heartbeat(
        leader_id: NodeId,
        term: Term,
        prev_log_index: LogIndex,
        prev_log_term: Term,
        leader_commit: LogIndex,
    ) -> Self {
        Self {
            term,
            leader_id,
            prev_log_index,
            prev_log_term,
            entries: Vec::new(),
            leader_commit,
        }
    }

    /// Create an AppendEntries with log entries
    pub fn with_entries(
        leader_id: NodeId,
        term: Term,
        prev_log_index: LogIndex,
        prev_log_term: Term,
        entries: Vec<LogEntry>,
        leader_commit: LogIndex,
    ) -> Self {
        Self {
            term,
            leader_id,
            prev_log_index,
            prev_log_term,
            entries,
            leader_commit,
        }
    }

    /// Check if this is a heartbeat
    pub fn is_heartbeat(&self) -> bool {
        self.entries.is_empty()
    }
}

/// AppendEntries RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesResponse {
    /// Current term, for leader to update itself
    pub term: Term,

    /// True if follower contained entry matching prev_log_index and prev_log_term
    pub success: bool,

    /// Highest log index replicated (for updating match_index)
    pub match_index: LogIndex,
}

/// RequestVote RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestVoteRequest {
    /// Candidate's term
    pub term: Term,

    /// Candidate's node ID
    pub candidate_id: NodeId,

    /// Index of candidate's last log entry
    pub last_log_index: LogIndex,

    /// Term of candidate's last log entry
    pub last_log_term: Term,
}

/// RequestVote RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestVoteResponse {
    /// Current term, for candidate to update itself
    pub term: Term,

    /// True if candidate received vote
    pub vote_granted: bool,
}

impl RequestVoteResponse {
    /// Create a response granting vote
    pub fn grant(term: Term) -> Self {
        Self {
            term,
            vote_granted: true,
        }
    }

    /// Create a response denying vote
    pub fn deny(term: Term) -> Self {
        Self {
            term,
            vote_granted: false,
        }
    }
}

/// InstallSnapshot RPC request (for log compaction)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallSnapshotRequest {
    /// Leader's term
    pub term: Term,

    /// Leader's node ID
    pub leader_id: NodeId,

    /// Last included index in snapshot
    pub last_included_index: LogIndex,

    /// Last included term in snapshot
    pub last_included_term: Term,

    /// Byte offset of chunk
    pub offset: u64,

    /// Snapshot chunk data
    pub data: Vec<u8>,

    /// True if this is the last chunk
    pub done: bool,
}

/// InstallSnapshot RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallSnapshotResponse {
    /// Current term, for leader to update itself
    pub term: Term,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_entries_heartbeat() {
        let heartbeat = AppendEntriesRequest::heartbeat(
            NodeId::new(1),
            Term::new(1),
            LogIndex::new(0),
            Term::new(0),
            LogIndex::new(0),
        );

        assert!(heartbeat.is_heartbeat());
        assert_eq!(heartbeat.entries.len(), 0);
    }

    #[test]
    fn test_append_entries_with_entries() {
        let entries = vec![
            LogEntry::new(Term::new(1), LogIndex::new(1), b"data1".to_vec()),
            LogEntry::new(Term::new(1), LogIndex::new(2), b"data2".to_vec()),
        ];

        let request = AppendEntriesRequest::with_entries(
            NodeId::new(1),
            Term::new(1),
            LogIndex::new(0),
            Term::new(0),
            entries,
            LogIndex::new(0),
        );

        assert!(!request.is_heartbeat());
        assert_eq!(request.entries.len(), 2);
    }

    #[test]
    fn test_request_vote_response() {
        let grant = RequestVoteResponse::grant(Term::new(1));
        assert!(grant.vote_granted);

        let deny = RequestVoteResponse::deny(Term::new(1));
        assert!(!deny.vote_granted);
    }
}
