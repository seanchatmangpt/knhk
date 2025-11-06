//! Cover commands - Cover definition

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use crate::commands::cover as cover_impl;

#[derive(Serialize, Debug)]
struct CoverResult {
    select: String,
    shard: String,
}

/// Define cover over O
#[verb] // Noun "cover" auto-inferred from filename "cover.rs"
fn define(select: String, shard: String) -> Result<CoverResult> {
    cover_impl::define(select.clone(), shard.clone())
        .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed to define cover: {}", e)))
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
        .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed to list covers: {}", e)))
        .map(|covers| CoverList { covers })
}

