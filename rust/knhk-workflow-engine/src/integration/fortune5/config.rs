//! Fortune 5 Enterprise Configuration
//!
//! Configuration types for Fortune 5 enterprise integrations including
//! SPIFFE/SPIRE, KMS, multi-region, SLO, and promotion gates.

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Fortune 5 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fortune5Config {
    /// SPIFFE configuration
    pub spiffe: Option<SpiffeConfig>,
    /// KMS configuration
    pub kms: Option<KmsConfig>,
    /// Multi-region configuration
    pub multi_region: Option<MultiRegionConfig>,
    /// SLO configuration
    pub slo: Option<SloConfig>,
    /// Promotion gate configuration
    pub promotion: Option<PromotionConfig>,
}

impl Default for Fortune5Config {
    fn default() -> Self {
        Self {
            spiffe: None,
            kms: None,
            multi_region: None,
            slo: None,
            promotion: None,
        }
    }
}

/// SPIFFE/SPIRE configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiffeConfig {
    /// SPIFFE socket path
    pub socket_path: String,
    /// Trust domain
    pub trust_domain: String,
    /// SPIFFE ID
    pub spiffe_id: Option<String>,
    /// Certificate refresh interval (seconds)
    pub refresh_interval: u64,
}

/// KMS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KmsConfig {
    /// KMS provider type
    pub provider: KmsProvider,
    /// Provider-specific configuration
    pub provider_config: HashMap<String, String>,
    /// Key rotation interval (hours, must be ≤24)
    pub rotation_interval_hours: u32,
}

impl KmsConfig {
    /// Validate KMS configuration
    pub fn validate(&self) -> WorkflowResult<()> {
        if self.rotation_interval_hours > 24 {
            return Err(WorkflowError::Validation(format!(
                "Key rotation interval {} hours exceeds maximum 24 hours",
                self.rotation_interval_hours
            )));
        }
        Ok(())
    }
}

/// KMS provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KmsProvider {
    /// AWS KMS
    Aws,
    /// Azure Key Vault
    Azure,
    /// Google Cloud KMS
    Gcp,
    /// HashiCorp Vault
    Vault,
}

/// Multi-region configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiRegionConfig {
    /// Current region
    pub current_region: String,
    /// Replication regions
    pub replication_regions: Vec<String>,
    /// Replication strategy
    pub replication_strategy: ReplicationStrategy,
}

/// Replication strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplicationStrategy {
    /// Synchronous replication
    Sync,
    /// Asynchronous replication
    Async,
    /// Eventual consistency
    Eventual,
}

/// SLO configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloConfig {
    /// R1 P99 max latency (nanoseconds, must be ≤2ns)
    pub r1_p99_max_ns: u64,
    /// W1 P99 max latency (milliseconds, must be ≤1ms)
    pub w1_p99_max_ms: u64,
    /// C1 P99 max latency (milliseconds, must be ≤500ms)
    pub c1_p99_max_ms: u64,
    /// SLO window size (seconds)
    pub window_size_seconds: u64,
}

impl SloConfig {
    /// Validate SLO configuration
    pub fn validate(&self) -> WorkflowResult<()> {
        if self.r1_p99_max_ns > 2 {
            return Err(WorkflowError::Validation(format!(
                "R1 P99 max {}ns exceeds maximum 2ns",
                self.r1_p99_max_ns
            )));
        }
        if self.w1_p99_max_ms > 1 {
            return Err(WorkflowError::Validation(format!(
                "W1 P99 max {}ms exceeds maximum 1ms",
                self.w1_p99_max_ms
            )));
        }
        if self.c1_p99_max_ms > 500 {
            return Err(WorkflowError::Validation(format!(
                "C1 P99 max {}ms exceeds maximum 500ms",
                self.c1_p99_max_ms
            )));
        }
        Ok(())
    }
}

impl Default for SloConfig {
    fn default() -> Self {
        Self {
            r1_p99_max_ns: 2,
            w1_p99_max_ms: 1,
            c1_p99_max_ms: 500,
            window_size_seconds: 60,
        }
    }
}

/// Promotion gate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionConfig {
    /// Target environment
    pub environment: Environment,
    /// Feature flags
    pub feature_flags: Vec<String>,
    /// Auto-rollback enabled
    pub auto_rollback_enabled: bool,
    /// SLO threshold for promotion (0.0-1.0)
    pub slo_threshold: f64,
    /// Rollback window (seconds)
    pub rollback_window_seconds: u64,
}

impl PromotionConfig {
    /// Validate promotion configuration
    pub fn validate(&self) -> WorkflowResult<()> {
        if !(0.0..=1.0).contains(&self.slo_threshold) {
            return Err(WorkflowError::Validation(format!(
                "SLO threshold {} must be between 0.0 and 1.0",
                self.slo_threshold
            )));
        }
        Ok(())
    }
}

/// Environment type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    /// Development environment
    Development,
    /// Staging environment
    Staging,
    /// Production environment
    Production,
}

/// Admission strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdmissionStrategy {
    /// Allow all
    AllowAll,
    /// Deny all
    DenyAll,
    /// Require SPIFFE identity
    RequireSpiffe,
    /// Require KMS key
    RequireKms,
}
