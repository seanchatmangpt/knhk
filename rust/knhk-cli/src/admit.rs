//! Admit commands - Delta admission

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use crate::commands::admit as admit_impl;

#[derive(Serialize, Debug)]
struct AdmitResult {
    delta_file: String,
}

/// Admit Δ into O
#[verb] // Noun "admit" auto-inferred from filename "admit.rs"
fn delta(delta_file: String) -> Result<AdmitResult> {
    admit_impl::delta(delta_file.clone())
        .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed to admit delta: {}", e)))
        .map(|_| AdmitResult { delta_file })
}

