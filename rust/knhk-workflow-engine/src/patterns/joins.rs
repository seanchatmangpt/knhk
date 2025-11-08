//! Join state tracking for synchronization patterns
//!
//! Provides `JoinState` for tracking synchronization state in AND-join and OR-join patterns.

use serde_json::json;
use std::collections::HashSet;

/// Join state tracker for synchronization patterns
#[derive(Debug, Clone, Default)]
pub struct JoinState {
    /// Arrived edges (for AND-join)
    arrived: HashSet<String>,
    /// Total expected incoming edges (for AND-join)
    expected_total: Option<u32>,
    /// Active upstream edges (for OR-join)
    active_upstream: HashSet<String>,
}

impl JoinState {
    /// Create a new join state
    pub fn new() -> Self {
        Self::default()
    }

    /// Perform AND-join: returns true when all expected edges have arrived
    pub fn and_join(&mut self, expected_total: u32, arrived_edge: &str) -> bool {
        self.expected_total = Some(expected_total);
        self.arrived.insert(arrived_edge.to_string());

        if let Some(expected) = self.expected_total {
            self.arrived.len() >= expected as usize
        } else {
            false
        }
    }

    /// Perform OR-join: returns true when any active upstream edge arrives
    pub fn or_join(&mut self, active_upstream: &HashSet<String>, arrived_edge: &str) -> bool {
        self.active_upstream = active_upstream.clone();
        self.arrived.insert(arrived_edge.to_string());

        // OR-join fires when an active upstream edge arrives
        active_upstream.contains(arrived_edge)
    }

    /// Check if join should fire (all expected edges arrived for AND-join)
    pub fn should_fire(&self) -> bool {
        if let Some(expected) = self.expected_total {
            self.arrived.len() >= expected as usize
        } else {
            false
        }
    }

    /// Get arrived edges count
    pub fn arrived_count(&self) -> usize {
        self.arrived.len()
    }

    /// Get expected total
    pub fn expected_total(&self) -> Option<u32> {
        self.expected_total
    }

    /// Convert to JSON update for pattern execution result
    pub fn to_update(&self) -> serde_json::Value {
        json!({
            "arrived": self.arrived.iter().collect::<Vec<_>>(),
            "expected_total": self.expected_total,
            "active_upstream": self.active_upstream.iter().collect::<Vec<_>>(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_and_join() {
        let mut js = JoinState::new();
        assert!(!js.and_join(3, "edge1"));
        assert!(!js.and_join(3, "edge2"));
        assert!(js.and_join(3, "edge3"));
    }

    #[test]
    fn test_or_join() {
        let mut js = JoinState::new();
        let active: HashSet<String> = ["edge1".to_string(), "edge2".to_string()]
            .iter()
            .cloned()
            .collect();

        assert!(js.or_join(&active, "edge1"));
        assert!(!js.or_join(&active, "edge3")); // edge3 not in active set
    }

    #[test]
    fn test_should_fire() {
        let mut js = JoinState::new();
        assert!(!js.should_fire());

        js.and_join(2, "edge1");
        assert!(!js.should_fire());

        js.and_join(2, "edge2");
        assert!(js.should_fire());
    }
}

