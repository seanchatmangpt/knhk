//! Replicated log implementation for Raft
//!
//! The log is the core data structure in Raft. Each entry contains:
//! - A command for the state machine
//! - The term when the entry was created
//! - An index identifying its position
//!
//! # Log Properties
//!
//! - **Append-only**: Entries are never modified or deleted (except during log compaction)
//! - **Ordered**: Entries have monotonically increasing indices
//! - **Replicated**: Leader replicates entries to followers
//! - **Committed**: Entries are committed when replicated to majority

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Log entry in the replicated log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Term when entry was created
    pub term: Term,

    /// Index in the log
    pub index: LogIndex,

    /// Entry data (serialized command)
    pub data: Vec<u8>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(term: Term, index: LogIndex, data: Vec<u8>) -> Self {
        Self { term, index, data }
    }
}

/// Replicated log
pub struct ReplicatedLog {
    /// Log entries
    entries: VecDeque<LogEntry>,

    /// Last included index (for snapshots)
    last_included_index: LogIndex,

    /// Last included term (for snapshots)
    last_included_term: Term,
}

impl ReplicatedLog {
    /// Create a new replicated log
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            last_included_index: LogIndex::new(0),
            last_included_term: Term::new(0),
        }
    }

    /// Get the last log index
    pub fn last_index(&self) -> LogIndex {
        if let Some(entry) = self.entries.back() {
            entry.index
        } else {
            self.last_included_index
        }
    }

    /// Get the last log term
    pub fn last_term(&self) -> Term {
        if let Some(entry) = self.entries.back() {
            entry.term
        } else {
            self.last_included_term
        }
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if log is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get entry at index
    pub fn get(&self, index: LogIndex) -> Option<&LogEntry> {
        if index.inner() <= self.last_included_index.inner() {
            // Entry is in snapshot
            return None;
        }

        let offset = (index.inner() - self.last_included_index.inner() - 1) as usize;
        self.entries.get(offset)
    }

    /// Get entries starting from index
    pub fn get_entries_from(&self, start_index: LogIndex) -> Vec<LogEntry> {
        if start_index.inner() <= self.last_included_index.inner() {
            // Start index is in snapshot, return all entries
            return self.entries.iter().cloned().collect();
        }

        let offset = (start_index.inner() - self.last_included_index.inner() - 1) as usize;
        self.entries.iter().skip(offset).cloned().collect()
    }

    /// Append a new entry to the log
    pub fn append_entry(&mut self, term: Term, data: Vec<u8>) -> LogIndex {
        let index = LogIndex::new(self.last_index().inner() + 1);
        let entry = LogEntry::new(term, index, data);
        self.entries.push_back(entry);
        index
    }

    /// Append multiple entries
    pub fn append_entries(&mut self, entries: Vec<LogEntry>) {
        for entry in entries {
            self.entries.push_back(entry);
        }
    }

    /// Truncate log from index onwards
    pub fn truncate_from(&mut self, index: LogIndex) {
        if index.inner() <= self.last_included_index.inner() {
            // Cannot truncate snapshot
            return;
        }

        let offset = (index.inner() - self.last_included_index.inner() - 1) as usize;
        self.entries.truncate(offset);
    }

    /// Check if our log is at least as up-to-date as the given index and term
    pub fn is_up_to_date(&self, last_log_index: LogIndex, last_log_term: Term) -> bool {
        let our_term = self.last_term();
        let our_index = self.last_index();

        // Log is more up-to-date if:
        // 1. Last term is higher, OR
        // 2. Last term is equal and last index is higher or equal
        our_term > last_log_term || (our_term == last_log_term && our_index >= last_log_index)
    }

    /// Check if we have the entry at prev_log_index with prev_log_term
    /// (for AppendEntries consistency check)
    pub fn matches_prev_log(&self, prev_log_index: LogIndex, prev_log_term: Term) -> bool {
        if prev_log_index.inner() == 0 {
            // No previous entry required
            return true;
        }

        if prev_log_index.inner() == self.last_included_index.inner() {
            // Previous entry is the snapshot
            return prev_log_term == self.last_included_term;
        }

        if let Some(entry) = self.get(prev_log_index) {
            entry.term == prev_log_term
        } else {
            false
        }
    }

    /// Create a snapshot up to the given index
    pub fn create_snapshot(&mut self, last_included_index: LogIndex, state_data: Vec<u8>) -> Snapshot {
        // Find the term at last_included_index
        let last_included_term = if let Some(entry) = self.get(last_included_index) {
            entry.term
        } else {
            self.last_included_term
        };

        // Remove entries up to last_included_index
        let offset = (last_included_index.inner() - self.last_included_index.inner()) as usize;
        self.entries.drain(..offset);

        // Update snapshot metadata
        self.last_included_index = last_included_index;
        self.last_included_term = last_included_term;

        Snapshot {
            last_included_index,
            last_included_term,
            data: state_data,
        }
    }

    /// Install a snapshot
    pub fn install_snapshot(&mut self, snapshot: Snapshot) {
        self.last_included_index = snapshot.last_included_index;
        self.last_included_term = snapshot.last_included_term;

        // Discard entire log if snapshot is newer
        self.entries.clear();
    }
}

impl Default for ReplicatedLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot for log compaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Last included index
    pub last_included_index: LogIndex,

    /// Last included term
    pub last_included_term: Term,

    /// Snapshot data (serialized state machine state)
    pub data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replicated_log_append() {
        let mut log = ReplicatedLog::new();

        let idx1 = log.append_entry(Term::new(1), b"entry1".to_vec());
        assert_eq!(idx1.inner(), 1);

        let idx2 = log.append_entry(Term::new(1), b"entry2".to_vec());
        assert_eq!(idx2.inner(), 2);

        assert_eq!(log.len(), 2);
        assert_eq!(log.last_index().inner(), 2);
        assert_eq!(log.last_term().inner(), 1);
    }

    #[test]
    fn test_replicated_log_get() {
        let mut log = ReplicatedLog::new();

        log.append_entry(Term::new(1), b"entry1".to_vec());
        log.append_entry(Term::new(2), b"entry2".to_vec());

        let entry = log.get(LogIndex::new(1)).unwrap();
        assert_eq!(entry.term.inner(), 1);
        assert_eq!(entry.data, b"entry1");

        assert!(log.get(LogIndex::new(99)).is_none());
    }

    #[test]
    fn test_replicated_log_truncate() {
        let mut log = ReplicatedLog::new();

        log.append_entry(Term::new(1), b"entry1".to_vec());
        log.append_entry(Term::new(1), b"entry2".to_vec());
        log.append_entry(Term::new(2), b"entry3".to_vec());

        log.truncate_from(LogIndex::new(2));
        assert_eq!(log.len(), 1);
        assert_eq!(log.last_index().inner(), 1);
    }

    #[test]
    fn test_replicated_log_up_to_date() {
        let mut log = ReplicatedLog::new();

        log.append_entry(Term::new(1), b"entry1".to_vec());
        log.append_entry(Term::new(2), b"entry2".to_vec());

        // Our log is more up-to-date (higher term)
        assert!(log.is_up_to_date(LogIndex::new(5), Term::new(1)));

        // Our log is more up-to-date (same term, higher index)
        assert!(log.is_up_to_date(LogIndex::new(1), Term::new(2)));

        // Our log is less up-to-date (lower term)
        assert!(!log.is_up_to_date(LogIndex::new(1), Term::new(3)));
    }

    #[test]
    fn test_replicated_log_matches_prev() {
        let mut log = ReplicatedLog::new();

        log.append_entry(Term::new(1), b"entry1".to_vec());
        log.append_entry(Term::new(2), b"entry2".to_vec());

        // Matches
        assert!(log.matches_prev_log(LogIndex::new(0), Term::new(0)));
        assert!(log.matches_prev_log(LogIndex::new(1), Term::new(1)));
        assert!(log.matches_prev_log(LogIndex::new(2), Term::new(2)));

        // Does not match (wrong term)
        assert!(!log.matches_prev_log(LogIndex::new(1), Term::new(2)));

        // Does not match (index doesn't exist)
        assert!(!log.matches_prev_log(LogIndex::new(99), Term::new(1)));
    }

    #[test]
    fn test_snapshot() {
        let mut log = ReplicatedLog::new();

        log.append_entry(Term::new(1), b"entry1".to_vec());
        log.append_entry(Term::new(1), b"entry2".to_vec());
        log.append_entry(Term::new(2), b"entry3".to_vec());

        let snapshot = log.create_snapshot(LogIndex::new(2), b"state".to_vec());
        assert_eq!(snapshot.last_included_index.inner(), 2);
        assert_eq!(snapshot.last_included_term.inner(), 1);
        assert_eq!(log.len(), 1); // Only entry 3 remains
    }
}
