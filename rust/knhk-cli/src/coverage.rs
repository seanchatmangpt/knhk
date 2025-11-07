//! Coverage commands - Coverage operations

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use crate::commands::coverage as coverage_impl;

#[derive(Serialize, Debug)]
struct CoverageResult {
    coverage: String,
}

/// Get coverage
#[verb] // Noun "coverage" auto-inferred from filename "coverage.rs"
fn get() -> Result<CoverageResult> {
    coverage_impl::get()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("Failed to get coverage: {}", e)))
        .map(|coverage| CoverageResult { coverage })
}

