//! Route commands - Action routing

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use crate::commands::route as route_impl;

#[derive(Serialize, Debug)]
struct RouteResult {
    name: String,
    kind: String,
    target: String,
}

/// Install route
#[verb] // Noun "route" auto-inferred from filename "route.rs"
fn install(name: String, kind: String, target: String) -> Result<RouteResult> {
    route_impl::install(name.clone(), kind.clone(), target.clone())
        .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed to install route: {}", e)))
        .map(|_| RouteResult { name, kind, target })
}

#[derive(Serialize, Debug)]
struct RouteList {
    routes: Vec<String>,
}

/// List routes
#[verb] // Noun "route" auto-inferred
fn list() -> Result<RouteList> {
    route_impl::list()
        .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed to list routes: {}", e)))
        .map(|routes| RouteList { routes })
}

