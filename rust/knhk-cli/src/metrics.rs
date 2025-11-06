//! Metrics commands - Metrics operations

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use crate::commands::metrics as metrics_impl;

#[derive(Serialize, Debug)]
struct MetricsResult {
    metrics: std::collections::HashMap<String, String>,
}

/// Get metrics
#[verb] // Noun "metrics" auto-inferred from filename "metrics.rs"
fn get() -> Result<MetricsResult> {
    metrics_impl::get()
        .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed to get metrics: {}", e)))
        .map(|metrics| MetricsResult { metrics })
}

