//! Compile-time workflow validation
//!
//! Validates workflow definitions at compile time using:
//! - Petri net analysis for deadlock detection
//! - Graph algorithms for reachability
//! - Type checking for state transitions

use crate::parser::{StateTransition, WorkflowDefinition};
use petgraph::algo::{is_cyclic_directed, tarjan_scc};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::{HashMap, HashSet};
use syn::Ident;

/// Validation error with span information
#[derive(Debug)]
pub struct ValidationError {
    pub message: String,
    pub span: proc_macro2::Span,
    pub help: Option<String>,
}

impl ValidationError {
    pub fn to_compile_error(&self) -> proc_macro2::TokenStream {
        let message = &self.message;
        let span = self.span;

        let mut error = quote::quote_spanned! { span =>
            compile_error!(#message);
        };

        if let Some(help) = &self.help {
            let help_msg = format!("help: {}", help);
            error = quote::quote! {
                #error
                compile_error!(#help_msg);
            };
        }

        error
    }
}

/// Validate workflow definition
pub fn validate_workflow(workflow: &WorkflowDefinition) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Build workflow graph
    let (graph, node_map) = build_workflow_graph(workflow);

    // Check for deadlocks (cycles)
    if let Err(cycle_errors) = check_deadlocks(&graph, &node_map, workflow) {
        errors.extend(cycle_errors);
    }

    // Check for unreachable states
    if let Err(reachability_errors) = check_reachability(&graph, &node_map, workflow) {
        errors.extend(reachability_errors);
    }

    // Check for missing terminal states
    if let Err(terminal_errors) = check_terminal_states(&graph, &node_map, workflow) {
        errors.extend(terminal_errors);
    }

    // Validate pattern usage
    if let Err(pattern_errors) = validate_patterns(workflow) {
        errors.extend(pattern_errors);
    }

    // Validate constraints
    if let Err(constraint_errors) = validate_constraints(workflow) {
        errors.extend(constraint_errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Build directed graph from workflow definition
fn build_workflow_graph(
    workflow: &WorkflowDefinition,
) -> (DiGraph<String, ()>, HashMap<String, NodeIndex>) {
    let mut graph = DiGraph::new();
    let mut node_map = HashMap::new();

    // Helper to get or create node
    let get_or_create_node = |graph: &mut DiGraph<String, ()>,
                               node_map: &mut HashMap<String, NodeIndex>,
                               name: &Ident| -> NodeIndex {
        let name_str = name.to_string();
        *node_map.entry(name_str.clone()).or_insert_with(|| {
            graph.add_node(name_str)
        })
    };

    // Add edges for each transition
    for transition in &workflow.states {
        match transition {
            StateTransition::Simple { from, to } => {
                let from_idx = get_or_create_node(&mut graph, &mut node_map, from);
                let to_idx = get_or_create_node(&mut graph, &mut node_map, to);
                graph.add_edge(from_idx, to_idx, ());
            }
            StateTransition::Split { from, to } => {
                let from_idx = get_or_create_node(&mut graph, &mut node_map, from);
                for to_state in to {
                    let to_idx = get_or_create_node(&mut graph, &mut node_map, to_state);
                    graph.add_edge(from_idx, to_idx, ());
                }
            }
            StateTransition::Join { from, to } => {
                let to_idx = get_or_create_node(&mut graph, &mut node_map, to);
                for from_state in from {
                    let from_idx = get_or_create_node(&mut graph, &mut node_map, from_state);
                    graph.add_edge(from_idx, to_idx, ());
                }
            }
            StateTransition::Complex { from, to } => {
                for from_state in from {
                    for to_state in to {
                        let from_idx = get_or_create_node(&mut graph, &mut node_map, from_state);
                        let to_idx = get_or_create_node(&mut graph, &mut node_map, to_state);
                        graph.add_edge(from_idx, to_idx, ());
                    }
                }
            }
        }
    }

    (graph, node_map)
}

/// Check for deadlocks using cycle detection
fn check_deadlocks(
    graph: &DiGraph<String, ()>,
    _node_map: &HashMap<String, NodeIndex>,
    workflow: &WorkflowDefinition,
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Check if graph is cyclic
    if is_cyclic_directed(graph) {
        // Find all strongly connected components (cycles)
        let sccs = tarjan_scc(graph);

        for scc in sccs {
            // SCCs with more than one node or self-loops are cycles
            if scc.len() > 1 || (scc.len() == 1 && graph.neighbors(scc[0]).any(|n| n == scc[0])) {
                let cycle_states: Vec<String> = scc
                    .iter()
                    .filter_map(|&idx| graph.node_weight(idx))
                    .cloned()
                    .collect();

                let cycle_str = cycle_states.join(" -> ");

                errors.push(ValidationError {
                    message: format!(
                        "Workflow contains deadlock cycle: {} -> {}",
                        cycle_str,
                        cycle_states.first().map(|s| s.as_str()).unwrap_or("")
                    ),
                    span: workflow.name.span(),
                    help: Some("Add an exit condition, timeout, or cancellation pattern".to_string()),
                });
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Check for unreachable states
fn check_reachability(
    graph: &DiGraph<String, ()>,
    node_map: &HashMap<String, NodeIndex>,
    workflow: &WorkflowDefinition,
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Find initial state (assumed to be "Initial" or first state)
    let initial_state = node_map.get("Initial")
        .or_else(|| workflow.states.first().and_then(|t| match t {
            StateTransition::Simple { from, .. } => Some(node_map.get(&from.to_string())),
            StateTransition::Split { from, .. } => Some(node_map.get(&from.to_string())),
            _ => None,
        }).flatten());

    if let Some(&initial_idx) = initial_state {
        // DFS to find all reachable states
        let mut reachable = HashSet::new();
        let mut stack = vec![initial_idx];

        while let Some(node) = stack.pop() {
            if reachable.insert(node) {
                for neighbor in graph.neighbors(node) {
                    stack.push(neighbor);
                }
            }
        }

        // Check for unreachable states
        for (name, &idx) in node_map {
            if !reachable.contains(&idx) && name != "Initial" {
                errors.push(ValidationError {
                    message: format!("State '{}' is unreachable", name),
                    span: workflow.name.span(),
                    help: Some(format!("Add a transition path from Initial to {}", name)),
                });
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Check for terminal states (states with no outgoing edges)
fn check_terminal_states(
    graph: &DiGraph<String, ()>,
    node_map: &HashMap<String, NodeIndex>,
    workflow: &WorkflowDefinition,
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    let mut has_terminal = false;

    for (name, &idx) in node_map {
        if graph.neighbors(idx).count() == 0 {
            has_terminal = true;

            // Terminal states should be named appropriately
            if !name.contains("Complete") && !name.contains("End") && !name.contains("Terminal") {
                errors.push(ValidationError {
                    message: format!(
                        "State '{}' appears to be terminal but is not clearly named",
                        name
                    ),
                    span: workflow.name.span(),
                    help: Some(format!(
                        "Consider renaming '{}' to include 'Complete', 'End', or 'Terminal'",
                        name
                    )),
                });
            }
        }
    }

    if !has_terminal {
        errors.push(ValidationError {
            message: "Workflow has no terminal states".to_string(),
            span: workflow.name.span(),
            help: Some("Add at least one state with no outgoing transitions".to_string()),
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate pattern usage
fn validate_patterns(workflow: &WorkflowDefinition) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Known YAWL patterns
    let valid_patterns = [
        "Sequence",
        "ParallelSplit",
        "Synchronization",
        "ExclusiveChoice",
        "SimpleMerge",
        "MultiChoice",
        "StructuredSynchronizingMerge",
        "MultiMerge",
        "Discriminator",
        "ArbitraryCycles",
        "ImplicitTermination",
    ];

    for pattern in &workflow.patterns {
        let pattern_name = pattern.to_string();
        if !valid_patterns.contains(&pattern_name.as_str()) {
            errors.push(ValidationError {
                message: format!("Unknown workflow pattern: {}", pattern_name),
                span: pattern.span(),
                help: Some(format!(
                    "Valid patterns: {}",
                    valid_patterns.join(", ")
                )),
            });
        }
    }

    // Validate that declared patterns are actually used
    let has_splits = workflow.states.iter().any(|t| matches!(t, StateTransition::Split { .. }));
    let has_joins = workflow.states.iter().any(|t| matches!(t, StateTransition::Join { .. }));

    if has_splits && !workflow.patterns.iter().any(|p| p == "ParallelSplit") {
        errors.push(ValidationError {
            message: "Workflow uses parallel splits but doesn't declare ParallelSplit pattern".to_string(),
            span: workflow.name.span(),
            help: Some("Add 'ParallelSplit' to the patterns list".to_string()),
        });
    }

    if has_joins && !workflow.patterns.iter().any(|p| p == "Synchronization") {
        errors.push(ValidationError {
            message: "Workflow uses synchronization but doesn't declare Synchronization pattern".to_string(),
            span: workflow.name.span(),
            help: Some("Add 'Synchronization' to the patterns list".to_string()),
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate workflow constraints
fn validate_constraints(workflow: &WorkflowDefinition) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    if let Some(max_duration) = workflow.constraints.max_duration {
        if max_duration == 0 {
            errors.push(ValidationError {
                message: "max_duration must be greater than 0".to_string(),
                span: workflow.name.span(),
                help: Some("Set a positive duration value".to_string()),
            });
        }
    }

    if let Some(max_concurrency) = workflow.constraints.max_concurrency {
        if max_concurrency == 0 {
            errors.push(ValidationError {
                message: "max_concurrency must be greater than 0".to_string(),
                span: workflow.name.span(),
                help: Some("Set a positive concurrency value".to_string()),
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
