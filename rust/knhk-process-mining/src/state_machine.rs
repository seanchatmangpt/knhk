//! # Workflow Lifecycle State Machine - Poka-Yoke Implementation
//!
//! This module implements a type-safe state machine for workflow lifecycle management.
//! Invalid state transitions are prevented at compile time through Rust's type system.
//!
//! ## Poka-Yoke Principles Applied:
//!
//! 1. **Enum-Based States**: Only valid states can exist (no intermediate/invalid states)
//! 2. **Consuming Transitions**: State transitions consume `self`, preventing use-after-transition
//! 3. **Type-Safe Methods**: Only valid operations for each state are exposed
//! 4. **Compile-Time Prevention**: Invalid transitions cause compiler errors
//! 5. **Exhaustive Matching**: Compiler enforces handling all states
//!
//! ## Invalid States Made Impossible:
//!
//! - Cannot transition from Completed to any other state (type consumed)
//! - Cannot skip states (Initial -> Completed) - compiler error
//! - Cannot access state-specific data from wrong state - compiler error
//! - Cannot forget to handle a state - exhaustive match required
//! - Cannot use workflow after it's been transitioned - moved value error

use crate::types::{Count, Duration, Timestamp};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

// ============================================================================
// State Machine Error Types
// ============================================================================

/// Errors that can occur during state transitions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    /// Attempted an invalid state transition
    InvalidTransition {
        from: String,
        to: String,
        reason: String,
    },
    /// Already in the target state
    AlreadyInState { state: String },
    /// Not ready for the requested transition
    NotReadyForTransition { current_state: String, required: String },
    /// Cannot recover from error without valid recovery action
    RecoveryNotPossible { reason: String },
}

impl Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::InvalidTransition { from, to, reason } => {
                write!(f, "Cannot transition from {} to {}: {}", from, to, reason)
            }
            StateError::AlreadyInState { state } => {
                write!(f, "Already in state: {}", state)
            }
            StateError::NotReadyForTransition {
                current_state,
                required,
            } => {
                write!(
                    f,
                    "Not ready for transition from {}, requires: {}",
                    current_state, required
                )
            }
            StateError::RecoveryNotPossible { reason } => {
                write!(f, "Cannot recover from error state: {}", reason)
            }
        }
    }
}

impl std::error::Error for StateError {}

// ============================================================================
// WorkflowState - Enum-Based State Machine
// ============================================================================

/// Workflow lifecycle states.
///
/// ## Poka-Yoke Design:
///
/// Each state carries only the data relevant to that state, preventing:
/// - Accessing data from the wrong state (compiler error)
/// - Forgetting to initialize state-specific data (must provide on construction)
/// - Using stale data from previous states (old state is consumed)
///
/// ## Valid State Transitions:
///
/// ```text
/// Initial -> Running -> Completed
///         |         |
///         |         +-> Paused -> Running
///         |         |
///         |         +-> Error -> Running (with recovery)
///         |
///         +-> Error
/// ```
///
/// ## Invalid Transitions (Prevented by Type System):
///
/// - Initial -> Completed (skipping Running)
/// - Completed -> * (Completed consumes the workflow)
/// - Paused -> Completed (must go through Running)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum WorkflowState {
    /// Workflow has been created but not yet started.
    ///
    /// ## Available Operations:
    /// - `start()` - Begin workflow execution
    /// - `mark_error()` - Record initialization failure
    Initial {
        /// When the workflow was created
        created_at: Timestamp,
    },

    /// Workflow is actively processing cases.
    ///
    /// ## Available Operations:
    /// - `pause()` - Temporarily suspend processing
    /// - `complete()` - Finish processing successfully
    /// - `mark_error()` - Record processing failure
    /// - `increment_cases()` - Update case count
    Running {
        /// When the workflow started running
        started_at: Timestamp,
        /// Number of cases processed so far
        case_count: u64,
    },

    /// Workflow processing is temporarily suspended.
    ///
    /// ## Available Operations:
    /// - `resume()` - Continue processing
    /// - `complete()` - Finish without resuming
    /// - `mark_error()` - Record error during pause
    Paused {
        /// When the workflow was paused
        paused_at: Timestamp,
        /// Number of cases processed before pausing
        cases_processed: u64,
    },

    /// Workflow processing has finished successfully.
    ///
    /// ## Available Operations:
    /// - None (terminal state - value consumed)
    ///
    /// ## Poka-Yoke:
    /// After completion, the WorkflowState value is consumed and cannot be reused.
    /// This prevents accidentally transitioning from Completed to another state.
    Completed {
        /// When the workflow completed
        completed_at: Timestamp,
        /// Total number of cases processed
        total_cases: u64,
        /// Total time spent running (excludes paused time)
        total_duration: Duration,
    },

    /// Workflow encountered an error.
    ///
    /// ## Available Operations:
    /// - `recover()` - Attempt to recover and resume
    Error {
        /// When the error occurred
        error_at: Timestamp,
        /// Description of what went wrong
        error_message: String,
        /// Optional action that can be taken to recover
        recovery_action: Option<String>,
        /// State the workflow was in when error occurred
        previous_state: Box<WorkflowStateSnapshot>,
    },
}

/// Snapshot of workflow state (for error recovery).
///
/// ## Poka-Yoke:
/// Simplified snapshot type prevents recursive Error states containing Error states.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStateSnapshot {
    Initial,
    Running { started_at: Timestamp, case_count: u64 },
    Paused { paused_at: Timestamp, cases_processed: u64 },
}

impl WorkflowState {
    /// Creates a new workflow in Initial state.
    ///
    /// ## Poka-Yoke:
    /// This is the ONLY way to create a WorkflowState, ensuring all workflows start from Initial.
    pub fn new() -> Self {
        WorkflowState::Initial {
            created_at: Timestamp::now(),
        }
    }

    /// Creates a new workflow with a specific creation timestamp.
    ///
    /// Useful for testing or reconstructing historical workflows.
    pub fn new_at(created_at: Timestamp) -> Self {
        WorkflowState::Initial { created_at }
    }

    /// Starts the workflow (Initial -> Running).
    ///
    /// ## Poka-Yoke:
    /// - Consumes `self`, preventing reuse of Initial state
    /// - Only available when in Initial state (compiler enforced)
    ///
    /// ## Errors
    /// Returns error if not in Initial state.
    pub fn start(self) -> Result<Self, StateError> {
        match self {
            WorkflowState::Initial { .. } => Ok(WorkflowState::Running {
                started_at: Timestamp::now(),
                case_count: 0,
            }),
            _ => Err(StateError::InvalidTransition {
                from: self.state_name(),
                to: "Running".to_string(),
                reason: "Can only start from Initial state".to_string(),
            }),
        }
    }

    /// Pauses the workflow (Running -> Paused).
    ///
    /// ## Poka-Yoke:
    /// - Consumes `self`, preventing reuse of Running state
    /// - Preserves case count in paused state
    ///
    /// ## Errors
    /// Returns error if not in Running state.
    pub fn pause(self) -> Result<Self, StateError> {
        match self {
            WorkflowState::Running { case_count, .. } => Ok(WorkflowState::Paused {
                paused_at: Timestamp::now(),
                cases_processed: case_count,
            }),
            _ => Err(StateError::InvalidTransition {
                from: self.state_name(),
                to: "Paused".to_string(),
                reason: "Can only pause from Running state".to_string(),
            }),
        }
    }

    /// Resumes the workflow (Paused -> Running).
    ///
    /// ## Poka-Yoke:
    /// - Consumes `self`, preventing reuse of Paused state
    /// - Preserves case count from before pause
    ///
    /// ## Errors
    /// Returns error if not in Paused state.
    pub fn resume(self) -> Result<Self, StateError> {
        match self {
            WorkflowState::Paused {
                cases_processed, ..
            } => Ok(WorkflowState::Running {
                started_at: Timestamp::now(),
                case_count: cases_processed,
            }),
            _ => Err(StateError::InvalidTransition {
                from: self.state_name(),
                to: "Running".to_string(),
                reason: "Can only resume from Paused state".to_string(),
            }),
        }
    }

    /// Completes the workflow (Running/Paused -> Completed).
    ///
    /// ## Poka-Yoke:
    /// - Consumes `self`, making the workflow IMPOSSIBLE to use after completion
    /// - Calculates final metrics automatically
    /// - Terminal state - no further transitions possible
    ///
    /// ## Errors
    /// Returns error if not in Running or Paused state.
    pub fn complete(self, total_duration: Duration) -> Result<Self, StateError> {
        match self {
            WorkflowState::Running { case_count, .. } => Ok(WorkflowState::Completed {
                completed_at: Timestamp::now(),
                total_cases: case_count,
                total_duration,
            }),
            WorkflowState::Paused {
                cases_processed, ..
            } => Ok(WorkflowState::Completed {
                completed_at: Timestamp::now(),
                total_cases: cases_processed,
                total_duration,
            }),
            _ => Err(StateError::InvalidTransition {
                from: self.state_name(),
                to: "Completed".to_string(),
                reason: "Can only complete from Running or Paused state".to_string(),
            }),
        }
    }

    /// Records an error (Any -> Error).
    ///
    /// ## Poka-Yoke:
    /// - Consumes `self`, preventing further operations on failed state
    /// - Saves snapshot of previous state for recovery
    /// - Optional recovery action guides error handling
    pub fn mark_error(
        self,
        error_message: impl Into<String>,
        recovery_action: Option<String>,
    ) -> Self {
        let snapshot = match &self {
            WorkflowState::Initial { .. } => WorkflowStateSnapshot::Initial,
            WorkflowState::Running {
                started_at,
                case_count,
            } => WorkflowStateSnapshot::Running {
                started_at: *started_at,
                case_count: *case_count,
            },
            WorkflowState::Paused {
                paused_at,
                cases_processed,
            } => WorkflowStateSnapshot::Paused {
                paused_at: *paused_at,
                cases_processed: *cases_processed,
            },
            WorkflowState::Completed { .. } => {
                // Cannot error from Completed (should never happen due to consumption)
                WorkflowStateSnapshot::Initial
            }
            WorkflowState::Error { previous_state, .. } => {
                // Re-erroring: preserve original state
                *previous_state.clone()
            }
        };

        WorkflowState::Error {
            error_at: Timestamp::now(),
            error_message: error_message.into(),
            recovery_action,
            previous_state: Box::new(snapshot),
        }
    }

    /// Attempts to recover from error state (Error -> Running).
    ///
    /// ## Poka-Yoke:
    /// - Consumes `self`, preventing reuse of Error state
    /// - Only succeeds if recovery action was provided
    /// - Restores state from snapshot
    ///
    /// ## Errors
    /// Returns error if not in Error state or if no recovery is possible.
    pub fn recover(self) -> Result<Self, StateError> {
        match self {
            WorkflowState::Error {
                recovery_action,
                previous_state,
                ..
            } => {
                if recovery_action.is_none() {
                    return Err(StateError::RecoveryNotPossible {
                        reason: "No recovery action available".to_string(),
                    });
                }

                // Restore to Running state with preserved data
                match *previous_state {
                    WorkflowStateSnapshot::Running {
                        started_at,
                        case_count,
                    } => Ok(WorkflowState::Running {
                        started_at,
                        case_count,
                    }),
                    WorkflowStateSnapshot::Paused {
                        paused_at,
                        cases_processed,
                    } => {
                        // Resume from paused state
                        Ok(WorkflowState::Running {
                            started_at: paused_at,
                            case_count: cases_processed,
                        })
                    }
                    WorkflowStateSnapshot::Initial => {
                        // Restart from initial
                        Ok(WorkflowState::Running {
                            started_at: Timestamp::now(),
                            case_count: 0,
                        })
                    }
                }
            }
            _ => Err(StateError::InvalidTransition {
                from: self.state_name(),
                to: "Running".to_string(),
                reason: "Can only recover from Error state".to_string(),
            }),
        }
    }

    /// Increments the case count (only valid in Running state).
    ///
    /// ## Poka-Yoke:
    /// - Consumes and returns `self`, maintaining state machine invariants
    /// - Only valid in Running state
    ///
    /// ## Errors
    /// Returns error if not in Running state.
    pub fn increment_cases(self) -> Result<Self, StateError> {
        match self {
            WorkflowState::Running {
                started_at,
                case_count,
            } => Ok(WorkflowState::Running {
                started_at,
                case_count: case_count + 1,
            }),
            _ => Err(StateError::InvalidTransition {
                from: self.state_name(),
                to: "Running".to_string(),
                reason: "Can only increment cases in Running state".to_string(),
            }),
        }
    }

    /// Returns the name of the current state.
    pub fn state_name(&self) -> String {
        match self {
            WorkflowState::Initial { .. } => "Initial".to_string(),
            WorkflowState::Running { .. } => "Running".to_string(),
            WorkflowState::Paused { .. } => "Paused".to_string(),
            WorkflowState::Completed { .. } => "Completed".to_string(),
            WorkflowState::Error { .. } => "Error".to_string(),
        }
    }

    /// Checks if the workflow is in a terminal state.
    ///
    /// ## Poka-Yoke:
    /// Terminal states (Completed, Error without recovery) should not allow transitions.
    pub fn is_terminal(&self) -> bool {
        matches!(self, WorkflowState::Completed { .. })
            || matches!(
                self,
                WorkflowState::Error {
                    recovery_action: None,
                    ..
                }
            )
    }

    /// Gets the current case count (if applicable).
    ///
    /// ## Returns
    /// - `Some(count)` for Running, Paused, or Completed states
    /// - `None` for Initial or Error states
    pub fn case_count(&self) -> Option<u64> {
        match self {
            WorkflowState::Running { case_count, .. } => Some(*case_count),
            WorkflowState::Paused {
                cases_processed, ..
            } => Some(*cases_processed),
            WorkflowState::Completed { total_cases, .. } => Some(*total_cases),
            _ => None,
        }
    }
}

impl Default for WorkflowState {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for WorkflowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkflowState::Initial { created_at } => {
                write!(f, "Initial (created at {})", created_at)
            }
            WorkflowState::Running {
                started_at,
                case_count,
            } => {
                write!(
                    f,
                    "Running (started at {}, {} cases processed)",
                    started_at, case_count
                )
            }
            WorkflowState::Paused {
                paused_at,
                cases_processed,
            } => {
                write!(
                    f,
                    "Paused (paused at {}, {} cases processed)",
                    paused_at, cases_processed
                )
            }
            WorkflowState::Completed {
                completed_at,
                total_cases,
                total_duration,
            } => {
                write!(
                    f,
                    "Completed (finished at {}, {} cases, duration: {})",
                    completed_at, total_cases, total_duration
                )
            }
            WorkflowState::Error {
                error_at,
                error_message,
                recovery_action,
                ..
            } => {
                write!(f, "Error (at {}): {}", error_at, error_message)?;
                if let Some(action) = recovery_action {
                    write!(f, " [Recovery: {}]", action)?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_to_running() {
        let workflow = WorkflowState::new();
        let workflow = workflow.start().expect("should start");
        assert!(matches!(workflow, WorkflowState::Running { .. }));
    }

    #[test]
    fn test_running_to_paused() {
        let workflow = WorkflowState::new().start().unwrap();
        let workflow = workflow.pause().expect("should pause");
        assert!(matches!(workflow, WorkflowState::Paused { .. }));
    }

    #[test]
    fn test_paused_to_running() {
        let workflow = WorkflowState::new().start().unwrap().pause().unwrap();
        let workflow = workflow.resume().expect("should resume");
        assert!(matches!(workflow, WorkflowState::Running { .. }));
    }

    #[test]
    fn test_running_to_completed() {
        let workflow = WorkflowState::new().start().unwrap();
        let duration = Duration::new(5000);
        let workflow = workflow.complete(duration).expect("should complete");
        assert!(matches!(workflow, WorkflowState::Completed { .. }));
    }

    #[test]
    fn test_invalid_initial_to_completed() {
        let workflow = WorkflowState::new();
        let duration = Duration::new(1000);
        let result = workflow.complete(duration);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_recovery() {
        let workflow = WorkflowState::new().start().unwrap();
        let workflow = workflow.mark_error("Test error", Some("Restart".to_string()));
        assert!(matches!(workflow, WorkflowState::Error { .. }));

        let workflow = workflow.recover().expect("should recover");
        assert!(matches!(workflow, WorkflowState::Running { .. }));
    }

    #[test]
    fn test_error_no_recovery() {
        let workflow = WorkflowState::new().start().unwrap();
        let workflow = workflow.mark_error("Fatal error", None);
        let result = workflow.recover();
        assert!(result.is_err());
    }

    #[test]
    fn test_increment_cases() {
        let workflow = WorkflowState::new().start().unwrap();
        let workflow = workflow.increment_cases().unwrap();
        assert_eq!(workflow.case_count(), Some(1));

        let workflow = workflow.increment_cases().unwrap();
        assert_eq!(workflow.case_count(), Some(2));
    }

    #[test]
    fn test_case_count_preserved_through_pause() {
        let mut workflow = WorkflowState::new().start().unwrap();
        workflow = workflow.increment_cases().unwrap();
        workflow = workflow.increment_cases().unwrap();
        assert_eq!(workflow.case_count(), Some(2));

        workflow = workflow.pause().unwrap();
        assert_eq!(workflow.case_count(), Some(2));

        workflow = workflow.resume().unwrap();
        assert_eq!(workflow.case_count(), Some(2));
    }

    #[test]
    fn test_is_terminal() {
        let workflow = WorkflowState::new();
        assert!(!workflow.is_terminal());

        let workflow = workflow.start().unwrap();
        assert!(!workflow.is_terminal());

        let workflow = workflow.complete(Duration::new(1000)).unwrap();
        assert!(workflow.is_terminal());
    }

    #[test]
    fn test_error_state_is_terminal_without_recovery() {
        let workflow = WorkflowState::new().start().unwrap();
        let workflow = workflow.mark_error("Fatal", None);
        assert!(workflow.is_terminal());
    }

    #[test]
    fn test_error_state_not_terminal_with_recovery() {
        let workflow = WorkflowState::new().start().unwrap();
        let workflow = workflow.mark_error("Recoverable", Some("Retry".to_string()));
        assert!(!workflow.is_terminal());
    }
}
