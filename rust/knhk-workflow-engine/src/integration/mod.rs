//! Integration modules

mod best_practices;
mod check;
mod connectors;
pub mod fortune5;
mod lockchain;
pub mod otel;
pub mod otel_helpers;
#[macro_use]
pub mod otel_macros;
mod pattern_otel;
mod registry;
mod sidecar;
mod weaver;

// Re-export integration types
pub use best_practices::BestPracticesIntegration;
pub use check::{HealthCheckResult, HealthStatus, IntegrationHealthChecker};
pub use connectors::ConnectorIntegration;
pub use fortune5::*;
pub use lockchain::LockchainIntegration;
pub use otel::OtelIntegration;
pub use otel_helpers::*;
pub use pattern_otel::{PatternAttributes, PatternOtelHelper};
pub use registry::{IntegrationMetadata, IntegrationRegistry, IntegrationStatus};
pub use sidecar::SidecarIntegration;
pub use weaver::WeaverIntegration;
