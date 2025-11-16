// KNHK Closed Loop System - Dark Matter Implementation
// The 20% critical infrastructure that closes all loops

pub mod receipt;
pub mod observation;
pub mod invariants;
pub mod coordinator;
pub mod promoter;
pub mod chatman_equation;
pub mod doctrine;
pub mod governance;
pub mod shadow;

// Phase 7: LLM-Based Proposer modules
pub mod proposer;
pub mod prompt_engine;
pub mod validator_llm;
pub mod learning;

pub use receipt::{Receipt, ReceiptStore, ReceiptError};
pub use observation::{PatternDetector, Observation, ObservationStore};
pub use invariants::{HardInvariants, InvariantValidator, InvariantViolation};
pub use coordinator::{MapEKCoordinator, CoordinationError, LoopCycle};
pub use promoter::{SnapshotPromoter, PromotionError};
pub use chatman_equation::{ChatmanEquation, Action, ResourceBudget};
pub use doctrine::{
    DoctrineRule, DoctrineStore, DoctrineSnapshot, DoctrineViolation,
    ConstraintType, EnforcementLevel, ValidationContext, Signer,
    DoctrineToBoundInvariant, DoctrineError,
};
pub use governance::{
    Guard, GuardType, Criticality, RelaxationPolicy, GuardRelaxationRequest,
    RequestState, GuardApproval, Approval, GovernanceEngine, RelaxationWindow,
    GuardStatus, RequestStatus, GovernanceError,
};
pub use shadow::{
    ShadowEnvironment, ShadowManager, OntologyData, DeltaSigma,
    ClassDef, PropertyDef, GuardDef, GuardSeverity,
    ShadowTest, TestAssertion, TestCriticality, TestResult,
    ValidationState, IsolationLevel,
};
pub use proposer::{
    LLMProposer, OllamaLLMProposer, Proposal, ProposalRequest,
    SigmaDiff, ClassDefinition, PropertyDefinition, Cardinality,
    ValidationReport, ValidationStage, GuardProfile, PerformanceTier,
    PerformanceBudget, Sector, FewShotExample, LLMClient,
};
pub use prompt_engine::{PromptEngine, PromptEngineError};
pub use validator_llm::{
    ProposalValidator, ValidationPipeline, ValidationError,
    InvariantChecker,
};
pub use learning::{
    LearningSystem, ProposalOutcome, ProposalCorpus, LearningMetrics,
};

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
