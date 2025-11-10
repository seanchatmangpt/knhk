//! Connector registry - Manages connector instances

#[cfg(feature = "connectors")]
pub mod factory;
#[cfg(feature = "connectors")]
pub mod registry;

// Re-exported for internal use only
// pub use factory::ConnectorFactory;
#[cfg(feature = "connectors")]
pub use registry::ConnectorRegistry;
