// knhk-warm: Warm path operations (≤500ms budget)
// CONSTRUCT8 and other emit operations moved from hot path
// SPARQL query execution with oxigraph integration

// CRITICAL: Enforce proper error handling - no unwrap/expect in production code
// EXCEPTION: Default trait impl fallback expect() calls are acceptable (see graph.rs)
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

#[cfg(not(feature = "std"))]
compile_error!("knhk-warm requires std feature for oxigraph integration");

pub mod executor;
pub mod ffi;
pub mod graph;
pub mod hot_path;
pub mod query;
pub mod scheduler;
pub mod warm_path;

pub use executor::WarmPathExecutor;
pub use graph::WarmPathGraph;
pub use query::*;
pub use scheduler::{EpochPlan, EpochScheduler, ExecutionPlan};
pub use warm_path::*;
// Hot path types are re-exported from ffi module
pub use ffi::{Ctx, Ir, Op, Receipt, Run};
