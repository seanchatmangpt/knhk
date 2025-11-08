//! Integration modules

mod check;
mod connectors;
mod fortune5;
mod lockchain;
mod otel;
mod registry;

// Re-export integration types
pub use check::{HealthCheckResult, HealthStatus, IntegrationHealthChecker};
pub use connectors::ConnectorIntegration;
pub use fortune5::*;
pub use lockchain::LockchainIntegration;
pub use otel::OtelIntegration;
pub use registry::{IntegrationMetadata, IntegrationRegistry, IntegrationStatus};
