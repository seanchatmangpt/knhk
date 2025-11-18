//! Core types for the swarm framework

use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

/// Unique identifier for agents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub u64);

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Agent-{}", self.0)
    }
}

/// Agent role types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentRole {
    Queen,
    Worker,
    Scout,
    Guardian,
    Learner,
}

impl fmt::Display for AgentRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentRole::Queen => write!(f, "Queen"),
            AgentRole::Worker => write!(f, "Worker"),
            AgentRole::Scout => write!(f, "Scout"),
            AgentRole::Guardian => write!(f, "Guardian"),
            AgentRole::Learner => write!(f, "Learner"),
        }
    }
}

/// Agent state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    Idle,
    Working,
    Voting,
    Learning,
    Failed,
}

/// Task identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub u64);

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Task-{}", self.0)
    }
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Consensus vote
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vote {
    Approve,
    Reject,
    Abstain,
}

/// Proposal identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProposalId(pub u64);

/// Byzantine fault tolerance parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByzantineParams {
    /// Total number of agents
    pub n: usize,
    /// Maximum Byzantine agents tolerated
    pub f: usize,
    /// Quorum size (2f + 1)
    pub quorum: usize,
}

impl ByzantineParams {
    /// Create new Byzantine parameters
    /// Ensures n >= 3f + 1 for Byzantine fault tolerance
    pub fn new(n: usize) -> Self {
        let f = (n - 1) / 3;
        let quorum = 2 * f + 1;
        Self { n, f, quorum }
    }

    /// Check if we have quorum
    pub fn has_quorum(&self, count: usize) -> bool {
        count >= self.quorum
    }

    /// Calculate required votes for decision
    pub fn required_votes(&self) -> usize {
        self.quorum
    }
}

/// Hash type for Merkle trees
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub fn zero() -> Self {
        Hash([0u8; 32])
    }
}

/// Telemetry span context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
}

impl Default for SpanContext {
    fn default() -> Self {
        Self {
            trace_id: uuid::Uuid::new_v4().to_string(),
            span_id: uuid::Uuid::new_v4().to_string(),
            parent_span_id: None,
        }
    }
}

// Add uuid dependency
use uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byzantine_params() {
        // n = 4, f = 1, quorum = 3
        let params = ByzantineParams::new(4);
        assert_eq!(params.f, 1);
        assert_eq!(params.quorum, 3);
        assert!(params.has_quorum(3));
        assert!(!params.has_quorum(2));

        // n = 7, f = 2, quorum = 5
        let params = ByzantineParams::new(7);
        assert_eq!(params.f, 2);
        assert_eq!(params.quorum, 5);
        assert!(params.has_quorum(5));
        assert!(!params.has_quorum(4));
    }

    #[test]
    fn test_agent_id_display() {
        let id = AgentId(42);
        assert_eq!(format!("{}", id), "Agent-42");
    }

    #[test]
    fn test_task_priority_ordering() {
        assert!(TaskPriority::Critical > TaskPriority::High);
        assert!(TaskPriority::High > TaskPriority::Medium);
        assert!(TaskPriority::Medium > TaskPriority::Low);
    }
}
