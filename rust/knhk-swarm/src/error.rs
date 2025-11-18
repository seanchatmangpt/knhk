//! Error types for the swarm framework

use thiserror::Error;

/// Result type alias for swarm operations
pub type SwarmResult<T> = Result<T, SwarmError>;

/// Comprehensive error types for swarm operations
#[derive(Error, Debug)]
pub enum SwarmError {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Agent spawn failed: {0}")]
    AgentSpawnFailed(String),

    #[error("Consensus failed: {0}")]
    ConsensusFailed(String),

    #[error("Consensus timeout after {0}ms")]
    ConsensusTimeout(u64),

    #[error("Byzantine behavior detected: {0}")]
    ByzantineBehavior(String),

    #[error("Insufficient quorum: required {required}, got {actual}")]
    InsufficientQuorum { required: usize, actual: usize },

    #[error("Message delivery failed: {0}")]
    MessageDeliveryFailed(String),

    #[error("Learning convergence failed: {0}")]
    LearningFailed(String),

    #[error("Storage replication failed: {0}")]
    ReplicationFailed(String),

    #[error("Merkle verification failed: {0}")]
    MerkleVerificationFailed(String),

    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl SwarmError {
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            SwarmError::ConsensusTimeout(_)
                | SwarmError::MessageDeliveryFailed(_)
                | SwarmError::HealthCheckFailed(_)
        )
    }

    /// Check if error indicates Byzantine behavior
    pub fn is_byzantine(&self) -> bool {
        matches!(
            self,
            SwarmError::ByzantineBehavior(_) | SwarmError::MerkleVerificationFailed(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recoverability() {
        let timeout_err = SwarmError::ConsensusTimeout(1000);
        assert!(timeout_err.is_recoverable());

        let byzantine_err = SwarmError::ByzantineBehavior("test".into());
        assert!(!byzantine_err.is_recoverable());
        assert!(byzantine_err.is_byzantine());
    }
}
