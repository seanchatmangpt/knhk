//! Type-level state machines for workflow validation
//!
//! This module provides a type-safe state machine implementation using phantom types,
//! ensuring that invalid state transitions are caught at compile time.

use std::marker::PhantomData;
use std::collections::HashMap;
use serde_json::Value;

/// Workflow state marker trait
///
/// All workflow states must implement this trait. It provides compile-time
/// state identification and transition validation.
pub trait WorkflowState: Sized {
    /// State name for debugging and telemetry
    fn name() -> &'static str;

    /// Whether this is a terminal state
    fn is_terminal() -> bool {
        false
    }

    /// Whether this is an initial state
    fn is_initial() -> bool {
        false
    }
}

/// Type-safe workflow with compile-time state machine
///
/// The type parameter `S` represents the current state of the workflow.
/// State transitions are enforced by the type system - you can only call
/// methods that are valid for the current state.
///
/// # Example
///
/// ```rust,ignore
/// let workflow = TypedWorkflow::<Initial>::new("wf-1");
/// let workflow = workflow.start(); // Transition to Running
/// // workflow.start(); // Compile error: already running!
/// let workflow = workflow.complete(); // Transition to Complete
/// // workflow.complete(); // Compile error: already complete!
/// ```
#[derive(Debug, Clone)]
pub struct TypedWorkflow<S: WorkflowState> {
    /// Phantom type parameter to track current state
    state: PhantomData<S>,
    /// Workflow runtime data
    pub data: WorkflowData,
}

/// Runtime workflow data
#[derive(Debug, Clone)]
pub struct WorkflowData {
    pub id: String,
    pub created_at: std::time::SystemTime,
    pub context: HashMap<String, Value>,
}

impl WorkflowData {
    pub fn new(id: String) -> Self {
        Self {
            id,
            created_at: std::time::SystemTime::now(),
            context: HashMap::new(),
        }
    }

    pub fn with_context(mut self, key: String, value: Value) -> Self {
        self.context.insert(key, value);
        self
    }
}

/// Initial workflow state
#[derive(Debug, Clone, Copy)]
pub struct Initial;

impl WorkflowState for Initial {
    fn name() -> &'static str {
        "Initial"
    }

    fn is_initial() -> bool {
        true
    }
}

/// Email validated state
#[derive(Debug, Clone, Copy)]
pub struct EmailValidated;

impl WorkflowState for EmailValidated {
    fn name() -> &'static str {
        "EmailValidated"
    }
}

/// Account created state
#[derive(Debug, Clone, Copy)]
pub struct AccountCreated;

impl WorkflowState for AccountCreated {
    fn name() -> &'static str {
        "AccountCreated"
    }
}

/// Running workflow state
#[derive(Debug, Clone, Copy)]
pub struct Running;

impl WorkflowState for Running {
    fn name() -> &'static str {
        "Running"
    }
}

/// Complete workflow state
#[derive(Debug, Clone, Copy)]
pub struct Complete;

impl WorkflowState for Complete {
    fn name() -> &'static str {
        "Complete"
    }

    fn is_terminal() -> bool {
        true
    }
}

/// Failed workflow state
#[derive(Debug, Clone, Copy)]
pub struct Failed;

impl WorkflowState for Failed {
    fn name() -> &'static str {
        "Failed"
    }

    fn is_terminal() -> bool {
        true
    }
}

/// Cancelled workflow state
#[derive(Debug, Clone, Copy)]
pub struct Cancelled;

impl WorkflowState for Cancelled {
    fn name() -> &'static str {
        "Cancelled"
    }

    fn is_terminal() -> bool {
        true
    }
}

// Type-safe transitions
impl TypedWorkflow<Initial> {
    /// Create a new workflow in the initial state
    pub fn new(id: String) -> Self {
        Self {
            state: PhantomData,
            data: WorkflowData::new(id),
        }
    }

    /// Validate email and transition to EmailValidated state
    pub fn validate_email(self) -> TypedWorkflow<EmailValidated> {
        TypedWorkflow {
            state: PhantomData,
            data: self.data,
        }
    }

    /// Start workflow directly (skip email validation)
    pub fn start(self) -> TypedWorkflow<Running> {
        TypedWorkflow {
            state: PhantomData,
            data: self.data,
        }
    }
}

impl TypedWorkflow<EmailValidated> {
    /// Create account and transition to AccountCreated state
    pub fn create_account(self) -> TypedWorkflow<AccountCreated> {
        TypedWorkflow {
            state: PhantomData,
            data: self.data,
        }
    }

    /// Reject invalid email and transition to Failed state
    pub fn reject_invalid(self) -> TypedWorkflow<Failed> {
        TypedWorkflow {
            state: PhantomData,
            data: self.data,
        }
    }
}

impl TypedWorkflow<AccountCreated> {
    /// Complete workflow
    pub fn complete(self) -> TypedWorkflow<Complete> {
        TypedWorkflow {
            state: PhantomData,
            data: self.data,
        }
    }
}

impl TypedWorkflow<Running> {
    /// Complete workflow
    pub fn complete(self) -> TypedWorkflow<Complete> {
        TypedWorkflow {
            state: PhantomData,
            data: self.data,
        }
    }

    /// Fail workflow
    pub fn fail(self) -> TypedWorkflow<Failed> {
        TypedWorkflow {
            state: PhantomData,
            data: self.data,
        }
    }

    /// Cancel workflow
    pub fn cancel(self) -> TypedWorkflow<Cancelled> {
        TypedWorkflow {
            state: PhantomData,
            data: self.data,
        }
    }
}

// Terminal states have no transitions (compile-time guarantee of finality)
impl<S: WorkflowState> TypedWorkflow<S> {
    /// Get current state name
    pub fn state_name(&self) -> &'static str {
        S::name()
    }

    /// Check if current state is terminal
    pub fn is_terminal(&self) -> bool {
        S::is_terminal()
    }

    /// Check if current state is initial
    pub fn is_initial(&self) -> bool {
        S::is_initial()
    }

    /// Get workflow ID
    pub fn id(&self) -> &str {
        &self.data.id
    }

    /// Get workflow context
    pub fn context(&self) -> &HashMap<String, Value> {
        &self.data.context
    }

    /// Add context data (works in any state)
    pub fn with_context(mut self, key: String, value: Value) -> Self {
        self.data.context.insert(key, value);
        self
    }
}

/// Transition type-level function
///
/// This trait encodes valid state transitions at the type level.
pub trait Transition<From: WorkflowState, To: WorkflowState> {
    /// Perform the transition
    fn transition(workflow: TypedWorkflow<From>) -> TypedWorkflow<To>;
}

/// Macro to define valid transitions
///
/// # Example
///
/// ```rust,ignore
/// define_transition!(Initial -> EmailValidated);
/// define_transition!(EmailValidated -> AccountCreated);
/// define_transition!(AccountCreated -> Complete);
/// ```
#[macro_export]
macro_rules! define_transition {
    ($from:ty -> $to:ty) => {
        impl $crate::compile_time::Transition<$from, $to> for () {
            fn transition(workflow: $crate::compile_time::TypedWorkflow<$from>) -> $crate::compile_time::TypedWorkflow<$to> {
                $crate::compile_time::TypedWorkflow {
                    state: std::marker::PhantomData,
                    data: workflow.data,
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_safe_transitions() {
        // Valid transition sequence
        let workflow = TypedWorkflow::<Initial>::new("test-1".to_string());
        assert_eq!(workflow.state_name(), "Initial");
        assert!(workflow.is_initial());
        assert!(!workflow.is_terminal());

        let workflow = workflow.validate_email();
        assert_eq!(workflow.state_name(), "EmailValidated");
        assert!(!workflow.is_initial());
        assert!(!workflow.is_terminal());

        let workflow = workflow.create_account();
        assert_eq!(workflow.state_name(), "AccountCreated");

        let workflow = workflow.complete();
        assert_eq!(workflow.state_name(), "Complete");
        assert!(workflow.is_terminal());
    }

    #[test]
    fn test_context_management() {
        let workflow = TypedWorkflow::<Initial>::new("test-2".to_string())
            .with_context("email".to_string(), serde_json::json!("test@example.com"))
            .with_context("user_id".to_string(), serde_json::json!(12345));

        assert_eq!(workflow.context().len(), 2);
        assert_eq!(workflow.context()["email"], "test@example.com");
        assert_eq!(workflow.context()["user_id"], 12345);
    }

    #[test]
    fn test_failure_path() {
        let workflow = TypedWorkflow::<Initial>::new("test-3".to_string());
        let workflow = workflow.validate_email();
        let workflow = workflow.reject_invalid();
        assert_eq!(workflow.state_name(), "Failed");
        assert!(workflow.is_terminal());
    }

    #[test]
    fn test_cancellation() {
        let workflow = TypedWorkflow::<Initial>::new("test-4".to_string());
        let workflow = workflow.start();
        let workflow = workflow.cancel();
        assert_eq!(workflow.state_name(), "Cancelled");
        assert!(workflow.is_terminal());
    }
}
