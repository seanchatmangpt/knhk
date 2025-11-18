//! Convergence Tracking for Gossip Protocol
//!
//! Tracks when all agents converge to the same state and measures convergence time.

use super::state::VersionedState;
use super::{AgentId, RoundNumber};
use blake3::Hash as Blake3Hash;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Convergence state
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConvergenceState {
    /// Not yet converged
    Diverged,
    /// Partially converged (some agents agree)
    PartiallyConverged { percentage: u8 },
    /// Fully converged (all non-Byzantine agents agree)
    Converged { rounds: RoundNumber, duration: Duration },
}

/// Convergence tracker
pub struct ConvergenceTracker {
    /// Total agents in swarm
    total_agents: usize,
    /// Maximum Byzantine faults
    max_byzantine_faults: usize,
    /// Agent states (agent_id -> state_hash)
    agent_states: HashMap<AgentId, Blake3Hash>,
    /// Convergence start time
    start_time: Instant,
    /// Last convergence check round
    last_check_round: RoundNumber,
    /// Convergence history
    history: Vec<ConvergenceSnapshot>,
}

/// Convergence snapshot at a specific round
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConvergenceSnapshot {
    /// Round number
    pub round: RoundNumber,
    /// State hash distribution (hash -> count)
    pub state_distribution: HashMap<String, usize>,
    /// Convergence percentage (0-100)
    pub convergence_percentage: u8,
    /// Is fully converged
    pub is_converged: bool,
    /// Duration since start
    pub duration: Duration,
}

impl ConvergenceTracker {
    /// Create new convergence tracker
    pub fn new(total_agents: usize, max_byzantine_faults: usize) -> Self {
        Self {
            total_agents,
            max_byzantine_faults,
            agent_states: HashMap::new(),
            start_time: Instant::now(),
            last_check_round: 0,
            history: Vec::new(),
        }
    }

    /// Update agent state
    pub fn update_agent_state(&mut self, agent_id: AgentId, state: &VersionedState) {
        self.agent_states.insert(agent_id, state.state_hash);
    }

    /// Check convergence at current round
    pub fn check_convergence(&mut self, round: RoundNumber) -> ConvergenceState {
        self.last_check_round = round;

        if self.agent_states.is_empty() {
            return ConvergenceState::Diverged;
        }

        // Count state distribution
        let mut state_counts: HashMap<Blake3Hash, usize> = HashMap::new();
        for state_hash in self.agent_states.values() {
            *state_counts.entry(*state_hash).or_insert(0) += 1;
        }

        // Find majority state
        let (majority_hash, majority_count) = state_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(hash, count)| (*hash, *count))
            .unwrap_or((Blake3Hash::from([0u8; 32]), 0));

        // Calculate convergence percentage
        let convergence_percentage =
            ((majority_count as f64 / self.total_agents as f64) * 100.0) as u8;

        // Check if converged (all non-Byzantine agents agree)
        let min_converged = self.total_agents - self.max_byzantine_faults;
        let is_converged = majority_count >= min_converged;

        // Record snapshot
        let snapshot = ConvergenceSnapshot {
            round,
            state_distribution: state_counts
                .iter()
                .map(|(hash, count)| (format!("{:?}", hash), *count))
                .collect(),
            convergence_percentage,
            is_converged,
            duration: self.start_time.elapsed(),
        };
        self.history.push(snapshot);

        // Return convergence state
        if is_converged {
            let duration = self.start_time.elapsed();
            info!(
                round = round,
                duration_ms = duration.as_millis(),
                convergence_percentage,
                "Gossip converged"
            );
            ConvergenceState::Converged { rounds: round, duration }
        } else if convergence_percentage >= 50 {
            debug!(
                round = round,
                convergence_percentage,
                "Partial convergence"
            );
            ConvergenceState::PartiallyConverged {
                percentage: convergence_percentage,
            }
        } else {
            ConvergenceState::Diverged
        }
    }

    /// Get convergence percentage
    pub fn convergence_percentage(&self) -> u8 {
        if self.agent_states.is_empty() {
            return 0;
        }

        let mut state_counts: HashMap<Blake3Hash, usize> = HashMap::new();
        for state_hash in self.agent_states.values() {
            *state_counts.entry(*state_hash).or_insert(0) += 1;
        }

        let max_count = state_counts.values().max().copied().unwrap_or(0);
        ((max_count as f64 / self.total_agents as f64) * 100.0) as u8
    }

    /// Get convergence history
    pub fn history(&self) -> &[ConvergenceSnapshot] {
        &self.history
    }

    /// Reset tracker
    pub fn reset(&mut self) {
        self.agent_states.clear();
        self.start_time = Instant::now();
        self.last_check_round = 0;
        self.history.clear();
    }

    /// Calculate expected convergence time
    pub fn expected_convergence_rounds(swarm_size: usize, peer_sample_size: usize) -> usize {
        super::expected_convergence_rounds(swarm_size, peer_sample_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gossip::state::StateValue;

    #[test]
    fn test_convergence_tracker_creation() {
        let tracker = ConvergenceTracker::new(10, 3);
        assert_eq!(tracker.convergence_percentage(), 0);
    }

    #[test]
    fn test_convergence_tracking() {
        let mut tracker = ConvergenceTracker::new(10, 3);

        // Create states
        let state1 = VersionedState::new(1, StateValue::Number(100), 1);
        let state2 = VersionedState::new(1, StateValue::Number(100), 2);
        let state3 = VersionedState::new(1, StateValue::Number(200), 3);

        // All agents except one converge to same state
        for i in 1..=8 {
            tracker.update_agent_state(i, &state1);
        }
        tracker.update_agent_state(9, &state3);

        let convergence = tracker.check_convergence(5);
        match convergence {
            ConvergenceState::Converged { rounds, .. } => {
                assert_eq!(rounds, 5);
            }
            _ => panic!("Expected converged state"),
        }

        assert!(tracker.convergence_percentage() >= 70);
    }

    #[test]
    fn test_partial_convergence() {
        let mut tracker = ConvergenceTracker::new(10, 3);

        let state1 = VersionedState::new(1, StateValue::Number(100), 1);
        let state2 = VersionedState::new(1, StateValue::Number(200), 2);

        // 6 agents on state1, 4 on state2
        for i in 1..=6 {
            tracker.update_agent_state(i, &state1);
        }
        for i in 7..=10 {
            tracker.update_agent_state(i, &state2);
        }

        let convergence = tracker.check_convergence(3);
        match convergence {
            ConvergenceState::PartiallyConverged { percentage } => {
                assert!(percentage >= 50 && percentage < 100);
            }
            _ => panic!("Expected partially converged state"),
        }
    }

    #[test]
    fn test_expected_convergence_rounds() {
        assert_eq!(ConvergenceTracker::expected_convergence_rounds(100, 10), 2);
        assert_eq!(
            ConvergenceTracker::expected_convergence_rounds(1000, 10),
            3
        );
        assert_eq!(
            ConvergenceTracker::expected_convergence_rounds(10000, 10),
            4
        );
    }
}
