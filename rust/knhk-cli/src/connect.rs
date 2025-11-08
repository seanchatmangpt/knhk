//! Connect commands - Connector management

// Allow non_upper_case_globals - #[verb] macro generates static vars with lowercase names
#![allow(non_upper_case_globals)]

use crate::commands::connect as connect_impl;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize, Debug)]
struct RegisterResult {
    name: String,
    schema: String,
    source: String,
}

/// Register a connector
#[verb] // Noun "connect" auto-inferred from filename "connect.rs"
fn register(name: String, schema: String, source: String) -> Result<RegisterResult> {
    connect_impl::register(name.clone(), schema.clone(), source.clone())
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to register connector: {}",
                e
            ))
        })
        .map(|_| RegisterResult {
            name,
            schema,
            source,
        })
}

#[derive(Serialize, Debug)]
struct ConnectorList {
    connectors: Vec<String>,
}

/// List connectors
#[verb] // Noun "connect" auto-inferred
fn list() -> Result<ConnectorList> {
    connect_impl::list()
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to list connectors: {}",
                e
            ))
        })
        .map(|connectors| ConnectorList { connectors })
}
