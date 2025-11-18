//! Versioned State for Gossip Consensus
//!
//! Each agent maintains a versioned state with cryptographic hash proof.
//! Conflict resolution uses version ordering (higher version wins).

use super::{AgentId, Result, Timestamp};
use blake3::Hash as Blake3Hash;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use tracing::{debug, trace};

/// State value (generic payload)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum StateValue {
    /// Empty state (initial)
    Empty,
    /// Numeric value
    Number(i64),
    /// Text value
    Text(String),
    /// Binary blob
    Binary(Vec<u8>),
    /// Structured JSON
    Json(serde_json::Value),
}

impl StateValue {
    /// Compute Blake3 hash of state value
    pub fn hash(&self) -> Blake3Hash {
        let bytes = bincode::serialize(self).unwrap_or_default();
        blake3::hash(&bytes)
    }

    /// Size in bytes
    pub fn size(&self) -> usize {
        match self {
            StateValue::Empty => 0,
            StateValue::Number(_) => 8,
            StateValue::Text(s) => s.len(),
            StateValue::Binary(b) => b.len(),
            StateValue::Json(v) => serde_json::to_string(v).unwrap_or_default().len(),
        }
    }
}

/// Versioned state with cryptographic proof
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionedState {
    /// Version number (monotonically increasing)
    pub version: u64,
    /// State hash (Blake3)
    pub state_hash: Blake3Hash,
    /// Actual state value
    pub value: StateValue,
    /// Source agent that created this version
    pub source: AgentId,
    /// Timestamp (milliseconds since epoch)
    pub timestamp: Timestamp,
    /// Signature (optional, for Byzantine proof)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<Vec<u8>>,
}

impl VersionedState {
    /// Create new versioned state
    pub fn new(version: u64, value: StateValue, source: AgentId) -> Self {
        let state_hash = value.hash();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            version,
            state_hash,
            value,
            source,
            timestamp,
            signature: None,
        }
    }

    /// Verify state hash matches value
    pub fn verify_hash(&self) -> bool {
        let computed_hash = self.value.hash();
        self.state_hash == computed_hash
    }

    /// Compare versions (for conflict resolution)
    pub fn compare_version(&self, other: &VersionedState) -> Ordering {
        // Higher version wins
        match self.version.cmp(&other.version) {
            Ordering::Equal => {
                // If versions equal, use timestamp as tiebreaker
                match self.timestamp.cmp(&other.timestamp) {
                    Ordering::Equal => {
                        // If timestamps equal, use source ID as tiebreaker
                        self.source.cmp(&other.source)
                    }
                    ord => ord,
                }
            }
            ord => ord,
        }
    }

    /// Check if this state is newer than other
    pub fn is_newer_than(&self, other: &VersionedState) -> bool {
        self.compare_version(other) == Ordering::Greater
    }

    /// Merge with another state (higher version wins)
    pub fn merge(&mut self, other: VersionedState) -> bool {
        if other.is_newer_than(self) {
            trace!(
                "Merging state: v{} -> v{} (source: {} -> {})",
                self.version,
                other.version,
                self.source,
                other.source
            );
            *self = other;
            true
        } else {
            false
        }
    }
}

impl PartialEq for VersionedState {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version && self.state_hash == other.state_hash
    }
}

impl Eq for VersionedState {}

impl PartialOrd for VersionedState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VersionedState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare_version(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_value_hash() {
        let val1 = StateValue::Number(42);
        let val2 = StateValue::Number(42);
        let val3 = StateValue::Number(43);

        assert_eq!(val1.hash(), val2.hash());
        assert_ne!(val1.hash(), val3.hash());
    }

    #[test]
    fn test_versioned_state_creation() {
        let state = VersionedState::new(1, StateValue::Number(100), 1);
        assert_eq!(state.version, 1);
        assert_eq!(state.source, 1);
        assert!(state.verify_hash());
    }

    #[test]
    fn test_versioned_state_comparison() {
        let state1 = VersionedState::new(1, StateValue::Number(10), 1);
        let state2 = VersionedState::new(2, StateValue::Number(20), 2);
        let state3 = VersionedState::new(2, StateValue::Number(30), 3);

        assert!(state2.is_newer_than(&state1));
        assert!(!state1.is_newer_than(&state2));
        assert!(state2 < state3); // Same version, different source
    }

    #[test]
    fn test_versioned_state_merge() {
        let mut state1 = VersionedState::new(1, StateValue::Number(10), 1);
        let state2 = VersionedState::new(2, StateValue::Number(20), 2);

        let merged = state1.merge(state2.clone());
        assert!(merged);
        assert_eq!(state1.version, 2);
        assert_eq!(state1.value, StateValue::Number(20));
    }

    #[test]
    fn test_versioned_state_no_merge_older() {
        let mut state1 = VersionedState::new(2, StateValue::Number(20), 2);
        let state2 = VersionedState::new(1, StateValue::Number(10), 1);

        let merged = state1.merge(state2);
        assert!(!merged);
        assert_eq!(state1.version, 2);
    }
}
