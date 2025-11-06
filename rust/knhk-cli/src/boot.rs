//! Boot commands - System initialization

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use crate::commands::boot as boot_impl;

#[derive(Serialize, Debug)]
struct InitResult {
    sigma: String,
    q: String,
    config_dir: String,
}

/// Initialize Σ and Q
#[verb] // Noun "boot" auto-inferred from filename "boot.rs"
fn init(sigma: String, q: String) -> Result<InitResult> {
    let config_dir = boot_impl::init(sigma.clone(), q.clone())
        .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed to initialize: {}", e)))?;
    
    Ok(InitResult {
        sigma,
        q,
        config_dir: config_dir.to_string_lossy().to_string(),
    })
}

