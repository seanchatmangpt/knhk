// knhk-warm: Warm path operations (≤500ms budget)
// CONSTRUCT8 and other emit operations moved from hot path
// SPARQL query execution with oxigraph integration (optional rdf feature)

// CRITICAL: Enforce proper error handling - no unwrap/expect in production code
// EXCEPTION: Default trait impl fallback expect() calls are acceptable (see graph.rs)
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
// Allow acceptable warnings for clean build
#![allow(unused_imports)] // Some imports are conditional or reserved for planned use
#![allow(unused_variables)] // Some variables are used in conditional compilation
#![allow(unused_mut)] // Some mut variables are used in conditional compilation
#![allow(dead_code)] // Some code is reserved for planned features
#![allow(deprecated)] // Some dependencies use deprecated APIs (will be updated)
#![allow(unexpected_cfgs)] // Some cfg values are informational

#[cfg(not(feature = "std"))]
compile_error!("knhk-warm requires std feature");

pub mod construct8;
pub mod error;
#[cfg(feature = "rdf")]
pub mod executor;
pub mod ffi;
#[cfg(feature = "rdf")]
pub mod graph;
#[cfg(feature = "rdf")]
pub mod hot_path;
pub mod kernel;
#[cfg(feature = "rdf")]
pub mod query;
pub mod scheduler;
pub mod warm_path;

#[cfg(feature = "rdf")]
pub use executor::WarmPathExecutor;
#[cfg(feature = "rdf")]
pub use graph::WarmPathGraph;
#[cfg(feature = "rdf")]
pub use query::*;
pub use scheduler::{EpochPlan, EpochScheduler, ExecutionPlan};
pub use warm_path::*;
// Hot path types are re-exported from ffi module
pub use ffi::{Ctx, Ir, Op, Receipt, Run};
