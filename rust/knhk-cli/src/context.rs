//! Context commands - Context management

// Allow non_upper_case_globals - #[verb] macro generates static vars with lowercase names
#![allow(non_upper_case_globals)]

use crate::commands::context as context_impl;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize, Debug)]
struct ContextList {
    contexts: Vec<String>,
}

/// List contexts
#[verb] // Noun "context" auto-inferred from filename "context.rs"
fn list() -> Result<ContextList> {
    context_impl::list()
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to list contexts: {}",
                e
            ))
        })
        .map(|contexts| ContextList { contexts })
}

#[derive(Serialize, Debug)]
struct CurrentContextResult {
    context: String,
}

/// Show current context
#[verb] // Noun "context" auto-inferred
fn current() -> Result<CurrentContextResult> {
    context_impl::current()
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to get current context: {}",
                e
            ))
        })
        .map(|context| CurrentContextResult { context })
}

#[derive(Serialize, Debug)]
struct CreateContextResult {
    id: String,
    name: String,
    schema: String,
}

/// Create context
#[verb] // Noun "context" auto-inferred
fn create(id: String, name: String, schema: String) -> Result<CreateContextResult> {
    context_impl::create(id.clone(), name.clone(), schema.clone())
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to create context: {}",
                e
            ))
        })
        .map(|_| CreateContextResult { id, name, schema })
}

#[derive(Serialize, Debug)]
struct UseContextResult {
    id: String,
}

/// Use context
#[verb] // Noun "context" auto-inferred
fn use_context(id: String) -> Result<UseContextResult> {
    context_impl::use_context(id.clone())
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to use context: {}", e))
        })
        .map(|_| UseContextResult { id })
}
