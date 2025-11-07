// knhk-warm: Warm path operations (≤500ms budget)
// CONSTRUCT8 and other emit operations moved from hot path
// SPARQL query execution with oxigraph integration

// CRITICAL: Enforce proper error handling - no unwrap/expect in production code
// EXCEPTION: Default trait impl fallback .expect() is acceptable (see graph.rs)
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

#[cfg(not(feature = "std"))]
compile_error!("knhk-warm requires std feature for oxigraph integration");

pub mod ffi;
pub mod warm_path;
pub mod graph;
pub mod query;
pub mod executor;
pub mod hot_path;
pub mod scheduler;

pub use warm_path::*;
pub use graph::WarmPathGraph;
pub use query::*;
pub use executor::WarmPathExecutor;
pub use scheduler::{EpochScheduler, EpochPlan, ExecutionPlan};
// Hot path types are re-exported from ffi module
pub use ffi::{Op, Ctx, Ir, Receipt, Run};

