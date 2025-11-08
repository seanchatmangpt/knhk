//! Hook registry integration - Integrates with knhk-etl HookRegistry

pub mod registry;
pub mod store;

pub use registry::HookRegistryIntegration;
pub use store::HookStore;
