//! Fortune 5 Enterprise Best Practices
//!
//! This module provides enterprise-grade features for Fortune 5 deployments:
//! - Comprehensive observability (OTEL spans, metrics, distributed tracing)
//! - Security (SPIFFE/SPIRE, KMS integration, RBAC)
//! - Scalability (multi-region, horizontal scaling, state management)
//! - Reliability (SLOs, circuit breakers, retries, promotion gates)
//! - Performance (hot path optimization, SIMD support, caching)

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
}

impl Default for EnterpriseConfig {
    fn default() -> Self {
        Self {
            observability: ObservabilityConfig::default(),
            security: SecurityConfig::default(),
            scalability: ScalabilityConfig::default(),
            reliability: ReliabilityConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}
