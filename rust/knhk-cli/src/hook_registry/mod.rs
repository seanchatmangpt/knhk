//! Hook registry integration - Integrates with knhk-etl HookRegistry

pub mod registry;
pub mod store;

#[cfg(feature = "etl")]
pub use registry::HookRegistryIntegration;
// Re-exported for internal use only
// pub use store::HookStore;
