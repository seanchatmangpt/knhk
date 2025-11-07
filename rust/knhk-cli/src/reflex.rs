//! Reflex commands - Reflex declaration

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use crate::commands::reflex as reflex_impl;

#[derive(Serialize, Debug)]
struct ReflexResult {
    name: String,
    op: String,
    pred: u64,
    off: u64,
    len: u64,
}

/// Declare a reflex
#[verb] // Noun "reflex" auto-inferred from filename "reflex.rs"
fn declare(name: String, op: String, pred: u64, off: u64, len: u64) -> Result<ReflexResult> {
    reflex_impl::declare(name.clone(), op.clone(), pred, off, len)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("Failed to declare reflex: {}", e)))
        .map(|_| ReflexResult { name, op, pred, off, len })
}

#[derive(Serialize, Debug)]
struct ReflexList {
    reflexes: Vec<String>,
}

/// List reflexes
#[verb] // Noun "reflex" auto-inferred
fn list() -> Result<ReflexList> {
    reflex_impl::list()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("Failed to list reflexes: {}", e)))
        .map(|reflexes| ReflexList { reflexes })
}

