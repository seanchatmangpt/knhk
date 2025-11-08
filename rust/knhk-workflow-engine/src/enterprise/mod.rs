//! Fortune 5 Enterprise Best Practices
//!
//! This module provides enterprise-grade features for Fortune 5 deployments:
//! - Comprehensive observability (OTEL spans, metrics, distributed tracing)
//! - Security (SPIFFE/SPIRE, KMS integration, RBAC)
//! - Scalability (multi-region, horizontal scaling, state management)
//! - Reliability (SLOs, circuit breakers, retries, promotion gates)
//! - Performance (hot path optimization, SIMD support, caching)
//! - Compliance (provenance, audit logging, retention policies)

pub mod observability;
pub mod performance;
pub mod reliability;
pub mod scalability;
pub mod security;

pub use observability::ObservabilityConfig;
pub use performance::PerformanceConfig;
pub use reliability::ReliabilityConfig;
pub use scalability::ScalabilityConfig;
pub use security::SecurityConfig;

/// Compliance configuration
#[derive(Debug, Clone)]
pub struct ComplianceConfig {
    /// Enable provenance tracking
    pub enable_provenance: bool,
    /// Enable audit logging
    pub enable_audit: bool,
    /// Enable retention policies
    pub enable_retention: bool,
    /// Retention period in days
    pub retention_days: u32,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            enable_provenance: true,
            enable_audit: true,
            enable_retention: false,
            retention_days: 90,
        }
    }
}

/// Fortune 5 enterprise configuration
#[derive(Debug, Clone)]
pub struct EnterpriseConfig {
    /// Observability configuration
    pub observability: ObservabilityConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Scalability configuration
    pub scalability: ScalabilityConfig,
    /// Reliability configuration
    pub reliability: ReliabilityConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Compliance configuration
    pub compliance: ComplianceConfig,
}

impl Default for EnterpriseConfig {
    fn default() -> Self {
        Self {
            observability: ObservabilityConfig::default(),
            security: SecurityConfig::default(),
            scalability: ScalabilityConfig::default(),
            reliability: ReliabilityConfig::default(),
            performance: PerformanceConfig::default(),
            compliance: ComplianceConfig::default(),
        }
    }
}
