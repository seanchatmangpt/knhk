// KNHK Closed Loop System - Dark Matter Implementation
// The 20% critical infrastructure that closes all loops

pub mod receipt;
pub mod observation;
pub mod invariants;
pub mod coordinator;
pub mod promoter;

pub use receipt::{Receipt, ReceiptStore, ReceiptError};
pub use observation::{PatternDetector, Observation, ObservationStore};
pub use invariants::{HardInvariants, InvariantValidator, InvariantViolation};
pub use coordinator::{MapEKCoordinator, CoordinationError, LoopCycle};
pub use promoter::{SnapshotPromoter, PromotionError};

/// The Chatman Constant: maximum ticks for a hot path operation
pub const CHATMAN_CONSTANT: u32 = 8;

/// Core result type for closed-loop operations
pub type Result<T> = std::result::Result<T, ClosedLoopError>;

#[derive(Debug, thiserror::Error)]
pub enum ClosedLoopError {
    #[error("Receipt error: {0}")]
    Receipt(#[from] receipt::ReceiptError),

    #[error("Invariant violation: {0}")]
    Invariant(#[from] invariants::InvariantViolation),

    #[error("Coordination error: {0}")]
    Coordination(#[from] coordinator::CoordinationError),

    #[error("Promotion error: {0}")]
    Promotion(#[from] promoter::PromotionError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Cryptographic error: {0}")]
    Crypto(String),

    #[error("RDF query error: {0}")]
    RdfQuery(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chatman_constant_is_eight() {
        assert_eq!(CHATMAN_CONSTANT, 8);
    }
}
