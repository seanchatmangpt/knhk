// KNHK Closed Loop System - Dark Matter Implementation
// The 20% critical infrastructure that closes all loops

pub mod chatman_equation;
pub mod coordinator;
pub mod doctrine;
pub mod governance;
pub mod invariants;
pub mod observation;
pub mod promoter;
pub mod receipt;
pub mod shadow;

// Phase 7: LLM-Based Proposer modules
pub mod learning;
pub mod prompt_engine;
pub mod proposer;
pub mod validator_llm;

pub use chatman_equation::{Action, ChatmanEquation, ResourceBudget};
pub use coordinator::{CoordinationError, LoopCycle, MapEKCoordinator};
pub use doctrine::{
    ConstraintType, DoctrineError, DoctrineRule, DoctrineSnapshot, DoctrineStore,
    DoctrineToBoundInvariant, DoctrineViolation, EnforcementLevel, Signer, ValidationContext,
};
pub use governance::{
    Approval, Criticality, GovernanceEngine, GovernanceError, Guard, GuardApproval,
    GuardRelaxationRequest, GuardStatus, GuardType, RelaxationPolicy, RelaxationWindow,
    RequestState, RequestStatus,
};
pub use invariants::{HardInvariants, InvariantValidator, InvariantViolation};
pub use learning::{LearningMetrics, LearningSystem, ProposalCorpus, ProposalOutcome};
pub use observation::{Observation, ObservationStore, PatternDetector};
pub use promoter::{PromotionError, SnapshotPromoter};
pub use prompt_engine::{PromptEngine, PromptEngineError};
pub use proposer::{
    Cardinality, ClassDefinition, FewShotExample, GuardProfile, LLMClient, LLMProposer,
    OllamaLLMProposer, PerformanceBudget, PerformanceTier, PropertyDefinition, Proposal,
    ProposalRequest, Sector, SigmaDiff, ValidationReport, ValidationStage,
};
pub use receipt::{Receipt, ReceiptError, ReceiptStore};
pub use shadow::{
    ClassDef, DeltaSigma, GuardDef, GuardSeverity, IsolationLevel, OntologyData, PropertyDef,
    ShadowEnvironment, ShadowManager, ShadowTest, TestAssertion, TestCriticality, TestResult,
    ValidationState,
};
pub use validator_llm::{InvariantChecker, ProposalValidator, ValidationError, ValidationPipeline};

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

    #[error("Doctrine error: {0}")]
    Doctrine(#[from] doctrine::DoctrineError),

    #[error("Governance error: {0}")]
    Governance(#[from] governance::GovernanceError),

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
