//! Fortune 5 Enterprise Integration
//!
//! Provides SPIFFE/SPIRE, KMS, multi-region, SLO, and promotion gate integration
//! for Fortune 5 enterprise deployments.

use crate::error::{WorkflowError, WorkflowResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Fortune 5 configuration
#[derive(Debug, Clone)]
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

/// SPIFFE/SPIRE configuration
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct KmsConfig {
    /// KMS provider type
    pub provider: KmsProvider,
    /// Provider-specific configuration
    pub provider_config: HashMap<String, String>,
    /// Key rotation interval (hours, must be ≤24)
    pub rotation_interval_hours: u32,
}

/// KMS provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KmsProvider {
    /// AWS KMS
    Aws,
    /// Azure Key Vault
    Azure,
    /// HashiCorp Vault
    Vault,
}

/// Multi-region configuration
#[derive(Debug, Clone)]
pub struct MultiRegionConfig {
    /// Current region
    pub region: String,
    /// Primary region
    pub primary_region: Option<String>,
    /// Cross-region sync enabled
    pub cross_region_sync_enabled: bool,
    /// Receipt sync endpoints (other regions)
    pub receipt_sync_endpoints: Vec<String>,
    /// Quorum threshold (0.0-1.0)
    pub quorum_threshold: f64,
}

/// SLO configuration
#[derive(Debug, Clone)]
pub struct SloConfig {
    /// R1 P99 max nanoseconds (≤2ns)
    pub r1_p99_max_ns: u64,
    /// W1 P99 max milliseconds (≤1ms)
    pub w1_p99_max_ms: u64,
    /// C1 P99 max milliseconds (≤500ms)
    pub c1_p99_max_ms: u64,
    /// Admission strategy
    pub admission_strategy: AdmissionStrategy,
}

/// Admission strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionStrategy {
    /// Strict: reject if SLO cannot be met
    Strict,
    /// Degrade: allow but mark as degraded
    Degrade,
}

/// Promotion gate configuration
#[derive(Debug, Clone)]
pub struct PromotionConfig {
    /// Environment type
    pub environment: Environment,
    /// Feature flags
    pub feature_flags: Vec<String>,
    /// Auto-rollback enabled
    pub auto_rollback_enabled: bool,
    /// SLO threshold for rollback (0.0-1.0)
    pub slo_threshold: f64,
    /// Rollback window (seconds)
    pub rollback_window_seconds: u64,
}

/// Environment type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    /// Canary deployment with traffic percentage
    Canary { traffic_percent: f64 },
    /// Staging environment
    Staging,
    /// Production environment
    Production,
}

/// Fortune 5 integration manager
pub struct Fortune5Integration {
    config: Fortune5Config,
    /// SLO metrics tracking
    slo_metrics: Arc<RwLock<SloMetrics>>,
    /// Promotion gate state
    promotion_state: Arc<RwLock<PromotionState>>,
}

/// SLO metrics
#[derive(Debug, Default)]
struct SloMetrics {
    /// R1 latency samples (nanoseconds)
    r1_samples: Vec<u64>,
    /// W1 latency samples (milliseconds)
    w1_samples: Vec<u64>,
    /// C1 latency samples (milliseconds)
    c1_samples: Vec<u64>,
}

/// Promotion gate state
#[derive(Debug, Clone)]
struct PromotionState {
    /// Current environment
    environment: Environment,
    /// Active feature flags
    feature_flags: Vec<String>,
    /// SLO compliance status
    slo_compliant: bool,
    /// Last rollback time
    last_rollback_time: Option<std::time::Instant>,
}

impl Fortune5Integration {
    /// Create new Fortune 5 integration
    pub fn new(config: Fortune5Config) -> Self {
        let promotion_state = PromotionState {
            environment: config
                .promotion
                .as_ref()
                .map(|p| p.environment)
                .unwrap_or(Environment::Production),
            feature_flags: config
                .promotion
                .as_ref()
                .map(|p| p.feature_flags.clone())
                .unwrap_or_default(),
            slo_compliant: true,
            last_rollback_time: None,
        };

        Self {
            config,
            slo_metrics: Arc::new(RwLock::new(SloMetrics::default())),
            promotion_state: Arc::new(RwLock::new(promotion_state)),
        }
    }

    /// Validate KMS rotation interval (must be ≤24 hours)
    pub fn validate_kms_config(config: &KmsConfig) -> WorkflowResult<()> {
        if config.rotation_interval_hours > 24 {
            return Err(WorkflowError::Configuration(
                format!(
                    "KMS rotation interval {}h exceeds maximum 24h",
                    config.rotation_interval_hours
                )
                .into(),
            ));
        }
        Ok(())
    }

    /// Validate SLO configuration
    pub fn validate_slo_config(config: &SloConfig) -> WorkflowResult<()> {
        if config.r1_p99_max_ns > 2 {
            return Err(WorkflowError::Configuration(
                format!("R1 P99 max {}ns exceeds maximum 2ns", config.r1_p99_max_ns).into(),
            ));
        }
        if config.w1_p99_max_ms > 1 {
            return Err(WorkflowError::Configuration(
                format!("W1 P99 max {}ms exceeds maximum 1ms", config.w1_p99_max_ms).into(),
            ));
        }
        if config.c1_p99_max_ms > 500 {
            return Err(WorkflowError::Configuration(
                format!(
                    "C1 P99 max {}ms exceeds maximum 500ms",
                    config.c1_p99_max_ms
                )
                .into(),
            ));
        }
        Ok(())
    }

    /// Record SLO metric
    pub async fn record_slo_metric(&self, runtime_class: RuntimeClass, latency_ns: u64) {
        let mut metrics = self.slo_metrics.write().await;
        match runtime_class {
            RuntimeClass::R1 => {
                metrics.r1_samples.push(latency_ns);
                // Keep only last 1000 samples
                if metrics.r1_samples.len() > 1000 {
                    metrics.r1_samples.remove(0);
                }
            }
            RuntimeClass::W1 => {
                let latency_ms = latency_ns / 1_000_000;
                metrics.w1_samples.push(latency_ms);
                if metrics.w1_samples.len() > 1000 {
                    metrics.w1_samples.remove(0);
                }
            }
            RuntimeClass::C1 => {
                let latency_ms = latency_ns / 1_000_000;
                metrics.c1_samples.push(latency_ms);
                if metrics.c1_samples.len() > 1000 {
                    metrics.c1_samples.remove(0);
                }
            }
        }
    }

    /// Check SLO compliance
    pub async fn check_slo_compliance(&self) -> WorkflowResult<bool> {
        let Some(ref slo_config) = self.config.slo else {
            return Ok(true); // No SLO configured, always compliant
        };

        let metrics = self.slo_metrics.read().await;

        // Calculate P99 for each runtime class
        let r1_p99 = Self::calculate_p99(&metrics.r1_samples);
        let w1_p99 = metrics.w1_samples.iter().max().copied().unwrap_or(0) as u64;
        let c1_p99 = metrics.c1_samples.iter().max().copied().unwrap_or(0) as u64;

        let r1_compliant = r1_p99 <= slo_config.r1_p99_max_ns;
        let w1_compliant = w1_p99 <= slo_config.w1_p99_max_ms;
        let c1_compliant = c1_p99 <= slo_config.c1_p99_max_ms;

        Ok(r1_compliant && w1_compliant && c1_compliant)
    }

    /// Calculate P99 percentile
    fn calculate_p99(samples: &[u64]) -> u64 {
        if samples.is_empty() {
            return 0;
        }
        let mut sorted = samples.to_vec();
        sorted.sort();
        let index = (sorted.len() as f64 * 0.99).ceil() as usize - 1;
        sorted
            .get(index.min(sorted.len() - 1))
            .copied()
            .unwrap_or(0)
    }

    /// Check if promotion gate allows execution
    pub async fn check_promotion_gate(&self) -> WorkflowResult<bool> {
        let Some(ref promotion_config) = self.config.promotion else {
            return Ok(true); // No promotion gate configured, always allow
        };

        let state = self.promotion_state.read().await;

        // Check SLO compliance
        let slo_compliant = self.check_slo_compliance().await?;
        if !slo_compliant && promotion_config.auto_rollback_enabled {
            // Check if rollback window has passed
            if let Some(last_rollback) = state.last_rollback_time {
                let elapsed = last_rollback.elapsed().as_secs();
                if elapsed < promotion_config.rollback_window_seconds {
                    return Ok(false); // Still in rollback window
                }
            }
        }

        // Check environment-specific rules
        match promotion_config.environment {
            Environment::Canary { traffic_percent } => {
                // Canary: allow based on traffic percentage
                let random = fastrand::f64();
                Ok(random < traffic_percent / 100.0)
            }
            Environment::Staging => Ok(true), // Staging: always allow
            Environment::Production => Ok(slo_compliant), // Production: require SLO compliance
        }
    }

    /// Get current environment
    pub async fn get_environment(&self) -> Environment {
        let state = self.promotion_state.read().await;
        state.environment
    }

    /// Check if feature flag is enabled
    pub async fn is_feature_enabled(&self, feature: &str) -> bool {
        let state = self.promotion_state.read().await;
        state.feature_flags.contains(&feature.to_string())
    }
}

/// Runtime class for SLO tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeClass {
    /// R1: Hot path (≤2ns P99)
    R1,
    /// W1: Warm path (≤1ms P99)
    W1,
    /// C1: Cold path (≤500ms P99)
    C1,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kms_validation() {
        let config = KmsConfig {
            provider: KmsProvider::Aws,
            provider_config: HashMap::new(),
            rotation_interval_hours: 25, // Invalid: >24
        };
        assert!(Fortune5Integration::validate_kms_config(&config).is_err());

        let config = KmsConfig {
            provider: KmsProvider::Aws,
            provider_config: HashMap::new(),
            rotation_interval_hours: 24, // Valid
        };
        assert!(Fortune5Integration::validate_kms_config(&config).is_ok());
    }

    #[tokio::test]
    async fn test_slo_validation() {
        let config = SloConfig {
            r1_p99_max_ns: 3, // Invalid: >2
            w1_p99_max_ms: 1,
            c1_p99_max_ms: 500,
            admission_strategy: AdmissionStrategy::Strict,
        };
        assert!(Fortune5Integration::validate_slo_config(&config).is_err());

        let config = SloConfig {
            r1_p99_max_ns: 2,
            w1_p99_max_ms: 1,
            c1_p99_max_ms: 500,
            admission_strategy: AdmissionStrategy::Strict,
        };
        assert!(Fortune5Integration::validate_slo_config(&config).is_ok());
    }

    #[tokio::test]
    async fn test_slo_tracking() {
        let config = Fortune5Config::default();
        let integration = Fortune5Integration::new(config);

        // Record some metrics
        integration.record_slo_metric(RuntimeClass::R1, 1).await;
        integration.record_slo_metric(RuntimeClass::R1, 2).await;
        integration.record_slo_metric(RuntimeClass::R1, 3).await;

        // Check compliance (no SLO configured, should be compliant)
        assert!(integration
            .check_slo_compliance()
            .await
            .expect("Fortune5 integration should never fail"));
    }

    #[tokio::test]
    async fn test_promotion_gate() {
        let config = Fortune5Config {
            promotion: Some(PromotionConfig {
                environment: Environment::Staging,
                feature_flags: vec!["test-feature".to_string()],
                auto_rollback_enabled: false,
                slo_threshold: 0.95,
                rollback_window_seconds: 300,
            }),
            ..Default::default()
        };
        let integration = Fortune5Integration::new(config);

        // Staging should always allow
        assert!(integration
            .check_promotion_gate()
            .await
            .expect("Fortune5 integration should never fail"));
        assert!(integration.is_feature_enabled("test-feature").await);
        assert!(!integration.is_feature_enabled("other-feature").await);
    }
}
