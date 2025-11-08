//! Integration modules

mod best_practices;
mod check;
mod connectors;
pub mod fortune5;
mod lockchain;
mod otel;
mod registry;
mod sidecar;

// Re-export integration types
pub use best_practices::BestPracticesIntegration;
pub use check::{HealthCheckResult, HealthStatus, IntegrationHealthChecker};
pub use connectors::ConnectorIntegration;
pub use fortune5::*;
pub use lockchain::LockchainIntegration;
pub use otel::OtelIntegration;
pub use registry::{IntegrationMetadata, IntegrationRegistry, IntegrationStatus};
#[cfg(feature = "sidecar")]
pub use sidecar::SidecarIntegration;
