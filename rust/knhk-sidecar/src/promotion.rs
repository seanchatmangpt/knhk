// knhk-sidecar: Formal promotion gates for Fortune 5
// Canary, staging, production promotion with automatic rollback

use crate::error::{SidecarError, SidecarResult};
use crate::slo_admission::SloAdmissionController;
use std::collections::HashMap;
use tracing::{error, info, warn};

/// Deployment environment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    /// Canary deployment (percentage of traffic)
    Canary { traffic_percent: f64 },
    /// Staging environment
    Staging,
    /// Production environment
    Production,
}

impl Environment {
    pub fn name(&self) -> &'static str {
        match self {
            Environment::Canary { .. } => "canary",
            Environment::Staging => "staging",
            Environment::Production => "production",
        }
    }
}

/// Promotion configuration
#[derive(Debug, Clone)]
pub struct PromotionConfig {
    /// Current environment
    pub environment: Environment,
    /// Feature flags
    pub feature_flags: Vec<String>,
    /// Enable automatic rollback on SLO violations
    pub auto_rollback_enabled: bool,
    /// SLO threshold for rollback (default: 0.95 = 95%)
    pub slo_threshold: f64,
    /// Rollback window (duration to monitor before rollback)
    pub rollback_window_seconds: u64,
}

impl Default for PromotionConfig {
    fn default() -> Self {
        Self {
            environment: Environment::Production,
            feature_flags: Vec::new(),
            auto_rollback_enabled: true,
            slo_threshold: 0.95,
            rollback_window_seconds: 300, // 5 minutes
        }
    }
}

impl PromotionConfig {
    /// Validate promotion configuration
    pub fn validate(&self) -> SidecarResult<()> {
        if self.slo_threshold < 0.0 || self.slo_threshold > 1.0 {
            return Err(SidecarError::config_error(format!(
                "SLO threshold must be between 0.0 and 1.0, got {}",
                self.slo_threshold
            )));
        }

        if let Environment::Canary { traffic_percent } = self.environment {
            if traffic_percent < 0.0 || traffic_percent > 100.0 {
                return Err(SidecarError::config_error(format!(
                    "Canary traffic percent must be between 0.0 and 100.0, got {}",
                    traffic_percent
                )));
            }
        }

        Ok(())
    }
}

/// Promotion gate manager
///
/// Manages formal promotion gates (canary → staging → production)
/// with automatic rollback on SLO violations.
pub struct PromotionGateManager {
    config: PromotionConfig,
    slo_controller: SloAdmissionController,
    feature_flags: HashMap<String, bool>,
    rollback_history: Vec<RollbackEvent>,
}

impl PromotionGateManager {
    /// Create new promotion gate manager
    pub fn new(
        config: PromotionConfig,
        slo_controller: SloAdmissionController,
    ) -> SidecarResult<Self> {
        config.validate()?;

        let mut feature_flags = HashMap::new();
        for flag in &config.feature_flags {
            feature_flags.insert(flag.clone(), true);
        }

        Ok(Self {
            config,
            slo_controller,
            feature_flags,
            rollback_history: Vec::new(),
        })
    }

    /// Check if feature flag is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.feature_flags.get(feature).copied().unwrap_or(false)
    }

    /// Enable feature flag
    pub fn enable_feature(&mut self, feature: String) {
        self.feature_flags.insert(feature.clone(), true);
        info!("Feature flag enabled: {}", feature);
    }

    /// Disable feature flag (triggers rollback if in canary/staging)
    pub fn disable_feature(&mut self, feature: String) {
        self.feature_flags.insert(feature.clone(), false);
        info!("Feature flag disabled: {}", feature);

        if self.config.auto_rollback_enabled {
            self.record_rollback(feature, "Feature flag disabled".to_string());
        }
    }

    /// Check SLO compliance and trigger rollback if needed
    pub fn check_slo_compliance(&mut self) -> SidecarResult<bool> {
        let metrics = self.slo_controller.get_metrics();

        // Calculate SLO compliance rate
        let total_requests = metrics.r1_admitted
            + metrics.r1_rejected
            + metrics.w1_admitted
            + metrics.w1_rejected
            + metrics.c1_admitted
            + metrics.c1_rejected;

        if total_requests == 0 {
            return Ok(true); // No requests yet, cannot determine compliance
        }

        let admitted = metrics.r1_admitted + metrics.w1_admitted + metrics.c1_admitted;
        let compliance_rate = admitted as f64 / total_requests as f64;

        if compliance_rate < self.config.slo_threshold {
            if self.config.auto_rollback_enabled {
                error!(
                    "SLO compliance {} below threshold {}, triggering rollback",
                    compliance_rate, self.config.slo_threshold
                );
                self.trigger_rollback(format!(
                    "SLO compliance {} below threshold {}",
                    compliance_rate, self.config.slo_threshold
                ))?;
            }
            return Ok(false);
        }

        Ok(true)
    }

    /// Trigger automatic rollback
    fn trigger_rollback(&mut self, reason: String) -> SidecarResult<()> {
        info!("Triggering automatic rollback: {}", reason);

        // Disable all feature flags
        for feature in self.feature_flags.keys().cloned().collect::<Vec<_>>() {
            self.disable_feature(feature);
        }

        self.record_rollback("all".to_string(), reason);
        Ok(())
    }

    /// Record rollback event
    fn record_rollback(&mut self, feature: String, reason: String) {
        let event = RollbackEvent {
            feature,
            reason,
            timestamp: std::time::SystemTime::now(),
            environment: self.config.environment,
        };
        self.rollback_history.push(event);
    }

    /// Get rollback history
    pub fn get_rollback_history(&self) -> &[RollbackEvent] {
        &self.rollback_history
    }

    /// Promote to next environment
    pub fn promote(&mut self, target: Environment) -> SidecarResult<()> {
        // Check if promotion is valid
        let current = self.config.environment;
        match (current, target) {
            (Environment::Canary { .. }, Environment::Staging) => {
                // Canary → Staging: OK
            }
            (Environment::Staging, Environment::Production) => {
                // Staging → Production: OK
            }
            (Environment::Canary { .. }, Environment::Production) => {
                // Canary → Production: Skip staging (allowed but not recommended)
                warn!("Promoting directly from canary to production (skipping staging)");
            }
            _ => {
                return Err(SidecarError::config_error(format!(
                    "Invalid promotion from {:?} to {:?}",
                    current, target
                )));
            }
        }

        // Check SLO compliance before promotion
        if !self.check_slo_compliance()? {
            return Err(SidecarError::config_error(
                "Cannot promote: SLO compliance below threshold".to_string(),
            ));
        }

        info!("Promoting from {:?} to {:?}", current, target);
        self.config.environment = target;
        Ok(())
    }
}

/// Rollback event
#[derive(Debug, Clone)]
pub struct RollbackEvent {
    pub feature: String,
    pub reason: String,
    pub timestamp: std::time::SystemTime,
    pub environment: Environment,
}
