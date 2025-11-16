//! Compile-time workflow validation using procedural macros
//!
//! This crate provides procedural macros for defining and validating workflows
//! at compile time, catching errors like deadlocks, type mismatches, and missing
//! transitions before runtime.
//!
//! # Features
//!
//! - **Compile-time deadlock detection**: Analyzes workflow graph for cycles
//! - **Type-safe state transitions**: Invalid transitions = compilation error
//! - **Zero-cost abstraction**: All validation compiled away
//! - **Excellent error messages**: Span-based errors with suggestions
//!
//! # Example
//!
//! ```rust,ignore
//! use knhk_workflow_macros::workflow;
//!
//! workflow! {
//!     name: UserRegistration,
//!     patterns: [Sequence, ExclusiveChoice, ParallelSplit],
//!
//!     states: {
//!         Initial -> ValidateEmail,
//!         ValidateEmail -> [CreateAccount, RejectInvalid],
//!         CreateAccount -> [SendWelcome, CreateProfile],
//!         [SendWelcome, CreateProfile] -> Complete,
//!         RejectInvalid -> Complete,
//!     },
//!
//!     constraints: {
//!         max_duration: 30_000,
//!         max_concurrency: 100,
//!     },
//! }
//! ```

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

mod parser;
mod validator;
mod codegen;

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Define a workflow with compile-time validation
///
/// This macro parses a workflow DSL and validates it at compile time:
/// - Deadlock detection using Petri net analysis
/// - Type checking for state transitions
/// - Pattern compliance verification
/// - Resource bound checking
///
/// # Syntax
///
/// ```ignore
/// workflow! {
///     name: WorkflowName,
///     patterns: [Pattern1, Pattern2, ...],
///
///     states: {
///         State1 -> State2,
///         State2 -> [State3, State4],  // Parallel split
///         [State3, State4] -> State5,  // Synchronization
///     },
///
///     constraints: {
///         max_duration: 30_000,
///         max_concurrency: 100,
///     },
/// }
/// ```
///
/// # Compile-time Checks
///
/// - **Deadlock detection**: Analyzes workflow graph for cycles
/// - **Completeness**: All paths must lead to terminal states
/// - **Type safety**: State transitions must be valid
/// - **Pattern compliance**: Verifies YAWL pattern usage
/// - **Resource bounds**: Checks concurrency limits
///
/// # Error Messages
///
/// ```text
/// error: Workflow contains deadlock cycle: A -> B -> C -> A
///   --> src/workflows.rs:5:9
///    |
/// 5  |         C -> A,
///    |         ^^^^^^ deadlock detected
///    |
///    = help: Add an exit condition or timeout
/// ```
#[proc_macro]
pub fn workflow(input: TokenStream) -> TokenStream {
    let workflow_def = parse_macro_input!(input as parser::WorkflowDefinition);

    // Validate workflow at compile time
    if let Err(errors) = validator::validate_workflow(&workflow_def) {
        // Convert validation errors to compile errors
        let error_tokens: proc_macro2::TokenStream = errors
            .into_iter()
            .map(|e| e.to_compile_error())
            .collect();
        return error_tokens.into();
    }

    // Generate runtime code
    codegen::generate_workflow_code(&workflow_def).into()
}

/// Calculate workflow complexity at compile time
///
/// Returns the cyclomatic complexity of the workflow.
///
/// # Example
///
/// ```ignore
/// const WORKFLOW_COMPLEXITY: usize = calculate_complexity!(UserRegistration);
/// const_assert!(WORKFLOW_COMPLEXITY <= 100);
/// ```
#[proc_macro]
pub fn calculate_complexity(input: TokenStream) -> TokenStream {
    let workflow_name = parse_macro_input!(input as syn::Ident);

    // This would look up the workflow definition and calculate complexity
    // For now, we'll generate a const expression
    quote::quote! {
        {
            // Complexity calculation would be done at compile time
            // This is a placeholder for the actual implementation
            const COMPLEXITY: usize = #workflow_name::CALCULATED_COMPLEXITY;
            COMPLEXITY
        }
    }.into()
}

/// Check for deadlocks at compile time
///
/// Returns true if the workflow contains potential deadlocks.
///
/// # Example
///
/// ```ignore
/// const HAS_DEADLOCK: bool = check_deadlock!(UserRegistration);
/// const_assert!(!HAS_DEADLOCK);
/// ```
#[proc_macro]
pub fn check_deadlock(input: TokenStream) -> TokenStream {
    let workflow_name = parse_macro_input!(input as syn::Ident);

    quote::quote! {
        {
            const HAS_DEADLOCK: bool = #workflow_name::HAS_DEADLOCK;
            HAS_DEADLOCK
        }
    }.into()
}

/// Compile-time assertion
///
/// Similar to `static_assertions::const_assert!` but integrated with our workflow system.
///
/// # Example
///
/// ```ignore
/// const_assert!(WORKFLOW_COMPLEXITY <= 100);
/// const_assert!(!HAS_DEADLOCK);
/// ```
#[proc_macro]
pub fn const_assert(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as syn::Expr);

    quote::quote! {
        const _: () = {
            if !(#expr) {
                panic!("Compile-time assertion failed");
            }
        };
    }.into()
}
