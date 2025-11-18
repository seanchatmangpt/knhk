//! Error types for Byzantine consensus

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ByzantineError>;

#[derive(Error, Debug)]
pub enum ByzantineError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Consensus timeout after {timeout_ms}ms")]
    ConsensusTimeout { timeout_ms: u64 },

    #[error("Insufficient quorum: got {got}, need {need}")]
    InsufficientQuorum { got: usize, need: usize },

    #[error("Invalid signature from node {node_id}")]
    InvalidSignature { node_id: u64 },

    #[error("Byzantine node detected: {node_id}, reason: {reason}")]
    ByzantineNodeDetected { node_id: u64, reason: String },

    #[error("View change failed: {0}")]
    ViewChangeFailed(String),

    #[error("Invalid block: {0}")]
    InvalidBlock(String),

    #[error("Invalid quorum certificate: {0}")]
    InvalidQC(String),

    #[error("Duplicate message from node {node_id}")]
    DuplicateMessage { node_id: u64 },

    #[error("Message from unknown node {node_id}")]
    UnknownNode { node_id: u64 },

    #[error("Invalid view number: expected {expected}, got {got}")]
    InvalidView { expected: u64, got: u64 },

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Channel send error")]
    ChannelSend,

    #[error("Channel receive error")]
    ChannelReceive,

    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),

    #[error("MAPE-K analysis failed: {0}")]
    MAPEKAnalysisFailed(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Cryptographic error: {0}")]
    Crypto(String),
}

impl ByzantineError {
    /// Returns true if this error indicates a Byzantine fault
    pub fn is_byzantine_fault(&self) -> bool {
        matches!(
            self,
            ByzantineError::ByzantineNodeDetected { .. }
                | ByzantineError::InvalidSignature { .. }
                | ByzantineError::DuplicateMessage { .. }
        )
    }

    /// Returns true if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            ByzantineError::ConsensusTimeout { .. }
                | ByzantineError::InsufficientQuorum { .. }
                | ByzantineError::Network(_)
        )
    }

    /// Returns the node ID if this error is associated with a specific node
    pub fn node_id(&self) -> Option<u64> {
        match self {
            ByzantineError::InvalidSignature { node_id }
            | ByzantineError::ByzantineNodeDetected { node_id, .. }
            | ByzantineError::DuplicateMessage { node_id }
            | ByzantineError::UnknownNode { node_id } => Some(*node_id),
            _ => None,
        }
    }
}
