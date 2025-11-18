// knhk-yawl/src/triz.rs
// TRIZ principle enumeration and mapping for workflow pattern decomposition

use serde::{Deserialize, Serialize};

/// TRIZ (Theory of Inventive Problem Solving) principles used for pattern decomposition
///
/// Each YAWL pattern is mapped to one or more TRIZ principles that guide its
/// implementation and composition strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrizPrinciple {
    /// Principle 1: Segmentation - Divide workflow into independent steps
    /// Used for: Sequence, basic control flow
    Segmentation,

    /// Principle 2: Extraction - Extract decision criteria from execution
    /// Used for: Choice patterns, conditional branching
    Extraction,

    /// Principle 3: Taking Out - Separate resource concerns from logic
    /// Used for: Resource allocation, suspension, resumption
    TakingOut,

    /// Principle 4: Asymmetry - Allow different execution paths
    /// Used for: Parallel execution, asymmetric branching
    Asymmetry,

    /// Principle 10: Prior Action - Pre-synchronize before merge
    /// Used for: Synchronization patterns
    PriorAction,

    /// Principle 13: Do It in Reverse - Reverse cycles for termination
    /// Used for: Loop patterns, arbitrary cycles, backward flow
    DoItInReverse,

    /// Principle 18: Intermediary - Use intermediate error handlers
    /// Used for: Exception handling, compensation
    Intermediary,

    /// Principle 25: Self-Service - Let resource decide execution
    /// Used for: Deferred choice, runtime decisions
    SelfService,

    /// Principle 27: Cheap Short-lived - Use cheap merge strategies
    /// Used for: Advanced branching, discriminator, OR-join
    CheapShortLived,

    /// Principle 34: Discarding/Recovering - Add/remove branches dynamically
    /// Used for: Exclusive/Inclusive choice, cancellation
    DiscardingRecovering,
}

impl TrizPrinciple {
    /// Get the description of this TRIZ principle
    pub fn description(&self) -> &'static str {
        match self {
            Self::Segmentation => "Divide workflow into ordered, independent steps",
            Self::Extraction => "Extract decision criteria from execution logic",
            Self::TakingOut => "Separate resource concerns from workflow logic",
            Self::Asymmetry => "Allow different execution paths and parallel branches",
            Self::PriorAction => "Pre-synchronize state before merging branches",
            Self::DoItInReverse => "Use reverse cycles and backward flow for loops",
            Self::Intermediary => "Use intermediate handlers for errors and compensation",
            Self::SelfService => "Let resources decide execution path at runtime",
            Self::CheapShortLived => "Use cheap, short-lived merge strategies",
            Self::DiscardingRecovering => "Add or remove branches dynamically",
        }
    }

    /// Get example patterns that use this TRIZ principle
    pub fn example_patterns(&self) -> &'static [&'static str] {
        match self {
            Self::Segmentation => &["Sequence", "Sequential Routing"],
            Self::Extraction => &["Exclusive Choice", "Multi-Choice"],
            Self::TakingOut => &["Resource Allocation", "Suspension", "Resumption"],
            Self::Asymmetry => &["Parallel Split", "Synchronization"],
            Self::PriorAction => &["Synchronization", "Synchronizing Merge"],
            Self::DoItInReverse => &["Arbitrary Cycles", "Structured Loop", "Recursion"],
            Self::Intermediary => &["Cancel Task", "Compensation", "Exception Handling"],
            Self::SelfService => &["Deferred Choice", "Interleaved Routing"],
            Self::CheapShortLived => &["OR-Join", "Discriminator", "Multiple Merge"],
            Self::DiscardingRecovering => &["Cancellation", "Inclusive Choice"],
        }
    }
}

/// Mapping from pattern categories to primary TRIZ principles
pub struct TrizPatternMapping;

impl TrizPatternMapping {
    /// Get TRIZ principles for Basic Control Patterns (6 patterns)
    pub fn basic_control() -> &'static [TrizPrinciple] {
        &[
            TrizPrinciple::Segmentation,
            TrizPrinciple::Asymmetry,
            TrizPrinciple::PriorAction,
            TrizPrinciple::Extraction,
        ]
    }

    /// Get TRIZ principles for Advanced Branching Patterns (8 patterns)
    pub fn advanced_branching() -> &'static [TrizPrinciple] {
        &[
            TrizPrinciple::CheapShortLived,
            TrizPrinciple::DiscardingRecovering,
        ]
    }

    /// Get TRIZ principles for Structural Patterns (8 patterns)
    pub fn structural() -> &'static [TrizPrinciple] {
        &[TrizPrinciple::DoItInReverse, TrizPrinciple::Segmentation]
    }

    /// Get TRIZ principles for Resource Patterns (10 patterns)
    pub fn resource() -> &'static [TrizPrinciple] {
        &[TrizPrinciple::TakingOut, TrizPrinciple::DiscardingRecovering]
    }

    /// Get TRIZ principles for Exception Handling Patterns (5 patterns)
    pub fn exception_handling() -> &'static [TrizPrinciple] {
        &[TrizPrinciple::Intermediary, TrizPrinciple::DiscardingRecovering]
    }

    /// Get TRIZ principles for Data-Flow Patterns (6 patterns)
    pub fn data_flow() -> &'static [TrizPrinciple] {
        &[TrizPrinciple::Extraction, TrizPrinciple::TakingOut]
    }
}
