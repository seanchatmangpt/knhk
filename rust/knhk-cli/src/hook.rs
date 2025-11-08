//! Hook commands - Hook operations

// Allow non_upper_case_globals - #[verb] macro generates static vars with lowercase names
#![allow(non_upper_case_globals)]

use crate::commands::hook as hook_impl;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize, Debug)]
struct HookResult {
    name: String,
    op: String,
    pred: u64,
    off: u64,
    len: u64,
}

/// Create a hook
#[verb] // Noun "hook" auto-inferred from filename "hook.rs"
fn create(name: String, op: String, pred: u64, off: u64, len: u64) -> Result<HookResult> {
    hook_impl::create(
        name.clone(),
        op.clone(),
        pred,
        off,
        len,
        None,
        None,
        None,
        None,
    )
    .map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("Failed to create hook: {}", e))
    })
    .map(|_| HookResult {
        name,
        op,
        pred,
        off,
        len,
    })
}

#[derive(Serialize, Debug)]
struct HookList {
    hooks: Vec<String>,
}

/// List hooks
#[verb] // Noun "hook" auto-inferred
fn list() -> Result<HookList> {
    hook_impl::list()
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to list hooks: {}", e))
        })
        .map(|hooks| HookList { hooks })
}

#[derive(Serialize, Debug)]
struct EvalHookResult {
    name: String,
    result: String,
}

/// Evaluate a hook
#[verb] // Noun "hook" auto-inferred
fn eval(name: String) -> Result<EvalHookResult> {
    hook_impl::eval(name.clone())
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to eval hook: {}", e))
        })
        .map(|result| EvalHookResult { name, result })
}

#[derive(Serialize, Debug)]
struct ShowHookResult {
    name: String,
    id: String,
    op: String,
    pred: u64,
    off: u64,
    len: u64,
}

/// Show hook details
#[verb] // Noun "hook" auto-inferred
fn show(name: String) -> Result<ShowHookResult> {
    let hook = hook_impl::show(name.clone()).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("Failed to show hook: {}", e))
    })?;

    Ok(ShowHookResult {
        name: hook.name,
        id: hook.id,
        op: hook.op,
        pred: hook.pred,
        off: hook.off,
        len: hook.len,
    })
}
