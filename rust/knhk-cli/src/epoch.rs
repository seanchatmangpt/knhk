//! Epoch commands - Epoch operations

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use crate::commands::epoch as epoch_impl;

#[derive(Serialize, Debug)]
struct EpochResult {
    id: String,
    tau: u32,
    lambda: String,
}

/// Create epoch
#[verb] // Noun "epoch" auto-inferred from filename "epoch.rs"
fn create(id: String, tau: u32, lambda: String) -> Result<EpochResult> {
    epoch_impl::create(id.clone(), tau, lambda.clone())
        .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed to create epoch: {}", e)))
        .map(|_| EpochResult { id, tau, lambda })
}

#[derive(Serialize, Debug)]
struct RunEpochResult {
    id: String,
    success: bool,
}

/// Run epoch
#[verb] // Noun "epoch" auto-inferred
fn run(id: String) -> Result<RunEpochResult> {
    epoch_impl::run(id.clone())
        .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed to run epoch: {}", e)))
        .map(|_| RunEpochResult { id, success: true })
}

#[derive(Serialize, Debug)]
struct EpochList {
    epochs: Vec<String>,
}

/// List epochs
#[verb] // Noun "epoch" auto-inferred
fn list() -> Result<EpochList> {
    epoch_impl::list()
        .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed to list epochs: {}", e)))
        .map(|epochs| EpochList { epochs })
}

