//! Cover commands - Cover definition

// Allow non_upper_case_globals - #[verb] macro generates static vars with lowercase names
#![allow(non_upper_case_globals)]

use crate::commands::cover as cover_impl;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize, Debug)]
struct CoverResult {
    select: String,
    shard: String,
}

/// Define cover over O
#[verb] // Noun "cover" auto-inferred from filename "cover.rs"
fn define(select: String, shard: String) -> Result<CoverResult> {
    cover_impl::define(select.clone(), shard.clone())
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to define cover: {}", e))
        })
        .map(|_| CoverResult { select, shard })
}

#[derive(Serialize, Debug)]
struct CoverList {
    covers: Vec<String>,
}

/// List covers
#[verb] // Noun "cover" auto-inferred
fn list() -> Result<CoverList> {
    cover_impl::list()
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to list covers: {}", e))
        })
        .map(|covers| CoverList { covers })
}
