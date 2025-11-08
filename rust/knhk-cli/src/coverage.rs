//! Coverage commands - Coverage operations

// Allow non_upper_case_globals - #[verb] macro generates static vars with lowercase names
#![allow(non_upper_case_globals)]

use crate::commands::coverage as coverage_impl;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize, Debug)]
struct CoverageResult {
    coverage: String,
}

/// Get coverage
#[verb] // Noun "coverage" auto-inferred from filename "coverage.rs"
fn get() -> Result<CoverageResult> {
    coverage_impl::get()
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to get coverage: {}", e))
        })
        .map(|coverage| CoverageResult { coverage })
}
