//! Config commands - Configuration management

// Allow non_upper_case_globals - #[verb] macro generates static vars with lowercase names
#![allow(non_upper_case_globals)]

use crate::commands::config as config_impl;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize, Debug)]
struct ConfigResult {
    config: String,
}

/// Show current configuration
#[verb] // Noun "config" auto-inferred from filename "config.rs"
fn show() -> Result<ConfigResult> {
    config_impl::show()
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to show config: {}", e))
        })
        .map(|config| ConfigResult { config })
}
