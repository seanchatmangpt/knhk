// knhk-warm: Warm path operations (≤500ms budget)
// CONSTRUCT8 and other emit operations moved from hot path
// SPARQL query execution with oxigraph integration

#[cfg(not(feature = "std"))]
compile_error!("knhk-warm requires std feature for oxigraph integration");

pub mod ffi;
pub mod warm_path;
pub mod graph;
pub mod query;
pub mod executor;
pub mod hot_path;
pub mod mphf_cache;

pub use warm_path::*;
pub use graph::WarmPathGraph;
pub use query::*;
pub use executor::WarmPathExecutor;
pub use mphf_cache::{MPHFCache, MPHFCacheStats};

