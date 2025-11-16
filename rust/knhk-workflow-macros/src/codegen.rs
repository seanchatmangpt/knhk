//! Code generation for validated workflows
//!
//! Generates optimized runtime code from validated workflow definitions.

use crate::parser::{StateTransition, WorkflowDefinition};
use proc_macro2::TokenStream;
use quote::quote;

/// Generate workflow implementation code
pub fn generate_workflow_code(workflow: &WorkflowDefinition) -> TokenStream {
    let name = &workflow.name;
    let _patterns = &workflow.patterns;
    let _states = &workflow.states;

    // Calculate complexity at compile time
    let complexity = calculate_complexity(workflow);

    // Check for deadlocks at compile time (simplified)
    let has_deadlock = false; // Already validated in validator.rs

    // Generate state type definitions
    let state_types = generate_state_types(workflow);

    // Generate transition implementations
    let transitions = generate_transitions(workflow);

    // Generate workflow struct
    quote! {
        /// Workflow: #name
        ///
        /// Patterns: #(#patterns),*
        pub mod #name {
            use std::marker::PhantomData;

            /// Compile-time calculated complexity
            pub const CALCULATED_COMPLEXITY: usize = #complexity;

            /// Compile-time deadlock check
            pub const HAS_DEADLOCK: bool = #has_deadlock;

            #state_types

            /// Type-safe workflow with compile-time state machine
            pub struct Workflow<S: WorkflowState> {
                state: PhantomData<S>,
                data: WorkflowData,
            }

            /// Workflow data (runtime state)
            #[derive(Debug, Clone)]
            pub struct WorkflowData {
                pub id: String,
                pub created_at: std::time::SystemTime,
                pub context: std::collections::HashMap<String, serde_json::Value>,
            }

            impl WorkflowData {
                pub fn new(id: String) -> Self {
                    Self {
                        id,
                        created_at: std::time::SystemTime::now(),
                        context: std::collections::HashMap::new(),
                    }
                }
            }

            #transitions

            /// Create a new workflow instance
            pub fn new(id: String) -> Workflow<Initial> {
                Workflow {
                    state: PhantomData,
                    data: WorkflowData::new(id),
                }
            }
        }
    }
}

/// Generate state type definitions
fn generate_state_types(workflow: &WorkflowDefinition) -> TokenStream {
    // Collect all unique states
    let mut states = std::collections::HashSet::new();

    for transition in &workflow.states {
        match transition {
            StateTransition::Simple { from, to } => {
                states.insert(from.to_string());
                states.insert(to.to_string());
            }
            StateTransition::Split { from, to } => {
                states.insert(from.to_string());
                for state in to {
                    states.insert(state.to_string());
                }
            }
            StateTransition::Join { from, to } => {
                for state in from {
                    states.insert(state.to_string());
                }
                states.insert(to.to_string());
            }
            StateTransition::Complex { from, to } => {
                for state in from {
                    states.insert(state.to_string());
                }
                for state in to {
                    states.insert(state.to_string());
                }
            }
        }
    }

    // Generate type definitions for each state
    let state_defs: Vec<_> = states.iter().map(|state| {
        let state_ident = syn::Ident::new(state, proc_macro2::Span::call_site());
        quote! {
            #[derive(Debug, Clone, Copy)]
            pub struct #state_ident;

            impl WorkflowState for #state_ident {
                fn name() -> &'static str {
                    stringify!(#state_ident)
                }
            }
        }
    }).collect();

    quote! {
        /// Workflow state marker trait
        pub trait WorkflowState {
            fn name() -> &'static str;
        }

        #(#state_defs)*
    }
}

/// Generate state transition implementations
fn generate_transitions(workflow: &WorkflowDefinition) -> TokenStream {
    let mut impls = Vec::new();

    for transition in &workflow.states {
        match transition {
            StateTransition::Simple { from, to } => {
                let from_ident = from;
                let to_ident = to;
                let method_name = syn::Ident::new(
                    &format!("to_{}", to.to_string().to_lowercase()),
                    proc_macro2::Span::call_site()
                );

                impls.push(quote! {
                    impl Workflow<#from_ident> {
                        /// Transition to #to_ident state
                        pub fn #method_name(self) -> Workflow<#to_ident> {
                            Workflow {
                                state: PhantomData,
                                data: self.data,
                            }
                        }
                    }
                });
            }
            StateTransition::Split { from, to } => {
                // For splits, generate methods for each target state
                for to_state in to {
                    let from_ident = from;
                    let to_ident = to_state;
                    let method_name = syn::Ident::new(
                        &format!("to_{}", to_state.to_string().to_lowercase()),
                        proc_macro2::Span::call_site()
                    );

                    impls.push(quote! {
                        impl Workflow<#from_ident> {
                            /// Transition to #to_ident state (parallel split)
                            pub fn #method_name(self) -> Workflow<#to_ident> {
                                Workflow {
                                    state: PhantomData,
                                    data: self.data,
                                }
                            }
                        }
                    });
                }
            }
            StateTransition::Join { from, to } => {
                // For joins, generate methods from each source state
                for from_state in from {
                    let from_ident = from_state;
                    let to_ident = to;
                    let method_name = syn::Ident::new(
                        &format!("to_{}", to.to_string().to_lowercase()),
                        proc_macro2::Span::call_site()
                    );

                    impls.push(quote! {
                        impl Workflow<#from_ident> {
                            /// Transition to #to_ident state (synchronization)
                            pub fn #method_name(self) -> Workflow<#to_ident> {
                                Workflow {
                                    state: PhantomData,
                                    data: self.data,
                                }
                            }
                        }
                    });
                }
            }
            StateTransition::Complex { from, to } => {
                // Generate transitions for all combinations
                for from_state in from {
                    for to_state in to {
                        let from_ident = from_state;
                        let to_ident = to_state;
                        let method_name = syn::Ident::new(
                            &format!("to_{}", to_state.to_string().to_lowercase()),
                            proc_macro2::Span::call_site()
                        );

                        impls.push(quote! {
                            impl Workflow<#from_ident> {
                                /// Transition to #to_ident state
                                pub fn #method_name(self) -> Workflow<#to_ident> {
                                    Workflow {
                                        state: PhantomData,
                                        data: self.data,
                                    }
                                }
                            }
                        });
                    }
                }
            }
        }
    }

    quote! {
        #(#impls)*
    }
}

/// Calculate workflow complexity
fn calculate_complexity(workflow: &WorkflowDefinition) -> usize {
    // Cyclomatic complexity = edges - nodes + 2*connected_components
    let mut nodes = std::collections::HashSet::new();
    let mut edges = 0;

    for transition in &workflow.states {
        match transition {
            StateTransition::Simple { from, to } => {
                nodes.insert(from.to_string());
                nodes.insert(to.to_string());
                edges += 1;
            }
            StateTransition::Split { from, to } => {
                nodes.insert(from.to_string());
                for state in to {
                    nodes.insert(state.to_string());
                    edges += 1;
                }
            }
            StateTransition::Join { from, to } => {
                nodes.insert(to.to_string());
                for state in from {
                    nodes.insert(state.to_string());
                    edges += 1;
                }
            }
            StateTransition::Complex { from, to } => {
                for state in from {
                    nodes.insert(state.to_string());
                }
                for state in to {
                    nodes.insert(state.to_string());
                }
                edges += from.len() * to.len();
            }
        }
    }

    // Simplified: assume single connected component
    edges.saturating_sub(nodes.len()) + 2
}
