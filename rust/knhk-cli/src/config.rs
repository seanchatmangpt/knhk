//! Config commands - Configuration management

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use crate::commands::config as config_impl;

#[derive(Serialize, Debug)]
struct ConfigResult {
    config: String,
}

/// Show current configuration
#[verb] // Noun "config" auto-inferred from filename "config.rs"
fn show() -> Result<ConfigResult> {
    config_impl::show()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("Failed to show config: {}", e)))
        .map(|config| ConfigResult { config })
}

