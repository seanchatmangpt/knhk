//! State validation
//!
//! Validates state transitions and state consistency.

use crate::case::CaseState;
use crate::error::{WorkflowError, WorkflowResult};

/// Validate state transition
pub fn validate_state_transition(from: CaseState, to: CaseState) -> WorkflowResult<()> {
    use CaseState::*;

    match (from, to) {
        (Created, Started) => Ok(()),
        (Started, Running) => Ok(()),
        (Running, Completed) => Ok(()),
        (Running, Failed) => Ok(()),
        (Running, Cancelled) => Ok(()),
        (Started, Cancelled) => Ok(()),
        (Created, Cancelled) => Ok(()),
        (Failed, Started) => Ok(()), // Retry allowed
        (Cancelled, Started) => Ok(()), // Restart allowed
        _ => Err(WorkflowError::InvalidStateTransition {
            from: format!("{:?}", from),
            to: format!("{:?}", to),
        }),
    }
}

/// Validate case state is valid for operation
pub fn validate_state_for_operation(state: CaseState, operation: &str) -> WorkflowResult<()> {
    use CaseState::*;

    match (state, operation) {
        (Created, "start") => Ok(()),
        (Started, "execute") => Ok(()),
        (Running, "complete") => Ok(()),
        (Running, "cancel") => Ok(()),
        (Running, "fail") => Ok(()),
        (Failed, "retry") => Ok(()),
        (Cancelled, "restart") => Ok(()),
        _ => Err(WorkflowError::Validation(format!(
            "Operation {} not allowed in state {:?}",
            operation, state
        ))),
    }
}

