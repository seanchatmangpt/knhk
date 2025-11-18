//! YAWL Pattern definitions
//!
//! Covenant 4: All patterns are expressible via permutations
//!
//! This module implements all 43 W3C workflow patterns plus extensions.
//! Patterns are decomposable into combinations of split/join types.

use crate::core::ExecutionContext;
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod basic;
pub mod advanced;
pub mod state_based;

/// Error types for pattern operations
#[derive(Debug, thiserror::Error)]
pub enum PatternError {
    /// Invalid pattern configuration
    #[error("Invalid pattern configuration: {0}")]
    InvalidConfiguration(String),

    /// Pattern execution failed
    #[error("Pattern execution failed: {0}")]
    ExecutionFailed(String),

    /// Timeout exceeded (Covenant 5: Chatman constant)
    #[error("Pattern execution exceeded time bound: {0} ticks")]
    TimeoutExceeded(u64),
}

/// Pattern type enumeration (all 43+ W3C patterns)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PatternType {
    // Basic Control Flow (Patterns 1-5)
    Sequence,
    ParallelSplit,
    Synchronization,
    ExclusiveChoice,
    SimpleMerge,

    // Advanced Branching (Patterns 6-9)
    MultiChoice,
    StructuredSynchronizingMerge,
    MultiMerge,
    StructuredDiscriminator,

    // Structural Patterns (Patterns 10-13)
    ArbitraryCircles,
    ImplicitTermination,
    MultipleInstancesWithoutSynchronization,
    MultipleInstancesWithAPrioriDesignTimeKnowledge,

    // Multiple Instance Patterns (Patterns 14-18)
    MultipleInstancesWithAPrioriRuntimeKnowledge,
    MultipleInstancesWithoutAPrioriRuntimeKnowledge,
    DeferredChoice,
    InterleavedParallelRouting,
    Milestone,

    // State-Based Patterns (Patterns 19-29)
    CancelActivity,
    CancelCase,
    StructuredLoop,
    Recursion,
    Transient,
    PersistentTrigger,
    Cancel,
    CompleteWorkflowState,
    ThreadMerge,
    ThreadSplit,
    ExplicitTermination,

    // Cancellation and Force Completion (Patterns 30-32)
    ImplicitTerminationWithCancel,
    StaticPartialJoin,
    CancellingPartialJoin,

    // Iteration and Recursion (Patterns 33-39)
    GeneralizedANDJoin,
    LocalSynchronizingMerge,
    CancellationRegion,
    CompleteMultipleInstanceActivity,
    BlockingDiscriminator,
    CancellingDiscriminator,
    StructuredPartialJoin,

    // Advanced Patterns (40-43+)
    CriticalSection,
    InterleavedRouting,
    AtomicBlock,
    AdvancedSynchronization,
}

impl fmt::Display for PatternType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Output from pattern execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternOutput {
    /// Pattern that was executed
    pub pattern_type: PatternType,

    /// Execution duration in ticks (must be ≤ 8 for hot path - Covenant 5)
    pub duration_ticks: u64,

    /// Task IDs that were activated
    pub activated_tasks: Vec<String>,

    /// Any output data produced
    pub output_data: std::collections::HashMap<String, String>,
}

impl PatternOutput {
    /// Validate output against Chatman constant (Covenant 5)
    ///
    /// # Errors
    /// Returns error if duration exceeds 8 ticks
    pub fn validate_chatman_constant(&self) -> Result<(), PatternError> {
        const MAX_TICKS: u64 = 8;
        if self.duration_ticks > MAX_TICKS {
            return Err(PatternError::TimeoutExceeded(self.duration_ticks));
        }
        Ok(())
    }
}

/// Core trait for all YAWL patterns
///
/// All 43+ W3C patterns implement this trait.
/// Patterns must be:
/// - Send + Sync (safe for concurrent execution)
/// - Debug (observable via telemetry - Covenant 6)
pub trait YawlPattern: Send + Sync + fmt::Debug {
    /// Get the pattern type
    fn pattern_type(&self) -> PatternType;

    /// Decompose pattern into sub-patterns
    ///
    /// Covenant 4: All patterns are expressible via permutations
    fn decompose(&self) -> Vec<Box<dyn YawlPattern>>;

    /// Execute the pattern
    ///
    /// # Errors
    /// Returns error if execution fails or exceeds time bound
    ///
    /// # Covenant 5: Chatman Constant
    /// Hot path execution MUST complete in ≤ 8 ticks
    fn execute(&self, context: &ExecutionContext) -> Result<PatternOutput, PatternError>;

    /// Get pattern metadata
    fn metadata(&self) -> PatternMetadata {
        PatternMetadata {
            pattern_type: self.pattern_type(),
            is_hot_path: false,
            estimated_ticks: 0,
        }
    }
}

/// Metadata about a pattern
#[derive(Debug, Clone)]
pub struct PatternMetadata {
    /// Pattern type
    pub pattern_type: PatternType,

    /// Is this pattern on the hot path? (must be ≤ 8 ticks if true)
    pub is_hot_path: bool,

    /// Estimated execution time in ticks
    pub estimated_ticks: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_output_validation() {
        let output = PatternOutput {
            pattern_type: PatternType::Sequence,
            duration_ticks: 5,
            activated_tasks: vec!["task1".to_string()],
            output_data: std::collections::HashMap::new(),
        };

        assert!(output.validate_chatman_constant().is_ok());
    }

    #[test]
    fn test_pattern_output_chatman_violation() {
        let output = PatternOutput {
            pattern_type: PatternType::Sequence,
            duration_ticks: 10, // Exceeds 8 tick limit
            activated_tasks: vec!["task1".to_string()],
            output_data: std::collections::HashMap::new(),
        };

        assert!(output.validate_chatman_constant().is_err());
    }
}
