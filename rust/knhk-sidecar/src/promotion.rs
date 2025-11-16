// knhk-sidecar: Formal promotion gates for Fortune 5
// Canary, staging, production promotion with automatic rollback

use crate::error::{SidecarError, SidecarResult};
use crate::slo_admission::SloAdmissionController;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use tracing::{debug, error, info, warn};

/// Deployment environment
#[derive(Debug, Clone, Copy, PartialEq)]
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
            if !(0.0..=100.0).contains(&traffic_percent) {
                return Err(SidecarError::config_error(format!(
                    "Canary traffic percent must be between 0.0 and 100.0, got {}",
                    traffic_percent
                )));
            }
        }

        Ok(())
    }
}

/// Request routing decision
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    /// Target environment for this request
    pub target_environment: Environment,
    /// Whether this request is part of canary traffic
    pub is_canary: bool,
    /// Feature flags enabled for this request
    pub enabled_features: Vec<String>,
    /// Routing reason
    pub reason: String,
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
    /// Metrics for canary analysis
    canary_metrics: CanaryMetrics,
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
            canary_metrics: CanaryMetrics::default(),
        })
    }

    /// Route request based on canary configuration
    ///
    /// Uses deterministic hashing to ensure consistent routing for the same request ID
    pub fn route_request(&mut self, request_id: &str) -> RoutingDecision {
        match self.config.environment {
            Environment::Canary { traffic_percent } => {
                // Deterministic routing based on request ID hash
                let mut hasher = DefaultHasher::new();
                request_id.hash(&mut hasher);
                let hash = hasher.finish();

                // Convert hash to percentage (0-100)
                let request_percent = (hash % 100) as f64;

                let is_canary = request_percent < traffic_percent;

                if is_canary {
                    self.canary_metrics.record_canary_request();
                    debug!("Request {} routed to canary (hash {}%)", request_id, request_percent);

                    RoutingDecision {
                        target_environment: Environment::Canary { traffic_percent },
                        is_canary: true,
                        enabled_features: self.get_enabled_features(),
                        reason: format!(
                            "Canary routing: {}% < {}% threshold",
                            request_percent, traffic_percent
                        ),
                    }
                } else {
                    self.canary_metrics.record_production_request();
                    debug!("Request {} routed to production (hash {}%)", request_id, request_percent);

                    RoutingDecision {
                        target_environment: Environment::Production,
                        is_canary: false,
                        enabled_features: Vec::new(), // Production uses stable features only
                        reason: format!(
                            "Production routing: {}% >= {}% canary threshold",
                            request_percent, traffic_percent
                        ),
                    }
                }
            }
            Environment::Staging => {
                // All staging traffic gets new features
                self.canary_metrics.record_staging_request();

                RoutingDecision {
                    target_environment: Environment::Staging,
                    is_canary: false,
                    enabled_features: self.get_enabled_features(),
                    reason: "Staging environment: all features enabled".to_string(),
                }
            }
            Environment::Production => {
                // Production uses stable features only
                self.canary_metrics.record_production_request();

                RoutingDecision {
                    target_environment: Environment::Production,
                    is_canary: false,
                    enabled_features: self.get_stable_features(),
                    reason: "Production environment: stable features only".to_string(),
                }
            }
        }
    }

    /// Get list of enabled features
    fn get_enabled_features(&self) -> Vec<String> {
        self.feature_flags
            .iter()
            .filter(|(_, enabled)| **enabled)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get list of stable features (for production)
    fn get_stable_features(&self) -> Vec<String> {
        // In production, only features marked as stable are enabled
        // This would check a stability marker in real implementation
        Vec::new()
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

        // Check canary-specific SLO if in canary mode
        if let Environment::Canary { .. } = self.config.environment {
            let canary_compliance = self.canary_metrics.get_slo_compliance();

            if canary_compliance < self.config.slo_threshold {
                error!(
                    "Canary SLO compliance {} below threshold {}, triggering rollback",
                    canary_compliance, self.config.slo_threshold
                );

                if self.config.auto_rollback_enabled {
                    self.trigger_rollback(format!(
                        "Canary SLO violation: {} < {}",
                        canary_compliance, self.config.slo_threshold
                    ))?;
                }
                return Ok(false);
            }
        }

        // Check overall SLO compliance
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

    /// Monitor canary deployment health
    pub fn monitor_canary_health(&mut self) -> CanaryHealth {
        if let Environment::Canary { traffic_percent } = self.config.environment {
            let metrics = &self.canary_metrics;

            // Calculate error rates
            let canary_error_rate = metrics.get_canary_error_rate();
            let production_error_rate = metrics.get_production_error_rate();

            // Calculate latency differences
            let canary_p99 = metrics.canary_p99_latency;
            let production_p99 = metrics.production_p99_latency;

            let health = CanaryHealth {
                traffic_percent,
                canary_requests: metrics.canary_requests,
                production_requests: metrics.production_requests,
                canary_error_rate,
                production_error_rate,
                canary_p99_latency: canary_p99,
                production_p99_latency: production_p99,
                health_score: self.calculate_health_score(
                    canary_error_rate,
                    production_error_rate,
                    canary_p99,
                    production_p99,
                ),
                recommendation: self.get_canary_recommendation(
                    canary_error_rate,
                    production_error_rate,
                ),
            };

            // Auto-rollback if canary is unhealthy
            if health.health_score < 0.8 && self.config.auto_rollback_enabled {
                warn!("Canary health score {} below threshold, considering rollback", health.health_score);

                if let Err(e) = self.trigger_rollback(format!(
                    "Canary health degradation: score {}",
                    health.health_score
                )) {
                    error!("Failed to trigger rollback: {}", e);
                }
            }

            health
        } else {
            CanaryHealth::not_applicable()
        }
    }

    /// Calculate canary health score (0.0 - 1.0)
    fn calculate_health_score(
        &self,
        canary_error_rate: f64,
        production_error_rate: f64,
        canary_p99: std::time::Duration,
        production_p99: std::time::Duration,
    ) -> f64 {
        let mut score = 1.0;

        // Penalize if canary has higher error rate
        if canary_error_rate > production_error_rate * 1.1 {
            score -= (canary_error_rate - production_error_rate).min(0.5);
        }

        // Penalize if canary has worse latency
        if canary_p99 > production_p99 {
            let latency_ratio = canary_p99.as_millis() as f64 / production_p99.as_millis().max(1) as f64;
            score -= ((latency_ratio - 1.0) * 0.2).min(0.3);
        }

        score.max(0.0)
    }

    /// Get recommendation for canary deployment
    fn get_canary_recommendation(&self, canary_error_rate: f64, production_error_rate: f64) -> String {
        if canary_error_rate > production_error_rate * 2.0 {
            "ROLLBACK: Canary error rate significantly higher than production".to_string()
        } else if canary_error_rate > production_error_rate * 1.5 {
            "MONITOR: Canary showing elevated error rate".to_string()
        } else if canary_error_rate <= production_error_rate {
            "HEALTHY: Canary performing as well or better than production".to_string()
        } else {
            "OBSERVE: Canary performance within acceptable range".to_string()
        }
    }

    /// Trigger automatic rollback
    fn trigger_rollback(&mut self, reason: String) -> SidecarResult<()> {
        info!("Triggering automatic rollback: {}", reason);

        // Disable all feature flags
        for feature in self.feature_flags.keys().cloned().collect::<Vec<_>>() {
            self.disable_feature(feature);
        }

        // Reset to production environment
        let previous_env = self.config.environment;
        self.config.environment = Environment::Production;

        self.record_rollback(
            format!("{:?}", previous_env),
            reason
        );

        // Reset canary metrics
        self.canary_metrics = CanaryMetrics::default();

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

        // Check canary health before promotion
        if let Environment::Canary { .. } = current {
            let health = self.monitor_canary_health();
            if health.health_score < 0.9 {
                return Err(SidecarError::config_error(format!(
                    "Cannot promote: Canary health score {} below 0.9 threshold",
                    health.health_score
                )));
            }
        }

        info!("Promoting from {:?} to {:?}", current, target);
        self.config.environment = target;

        // Reset canary metrics when leaving canary
        if let Environment::Canary { .. } = current {
            self.canary_metrics = CanaryMetrics::default();
        }

        Ok(())
    }

    /// Record request outcome for SLO tracking
    pub fn record_request_outcome(
        &mut self,
        request_id: &str,
        success: bool,
        latency: std::time::Duration,
    ) {
        let decision = self.route_request(request_id);

        if decision.is_canary {
            if success {
                self.canary_metrics.canary_successes += 1;
            } else {
                self.canary_metrics.canary_errors += 1;
            }

            // Update P99 latency (simplified - real implementation would use histogram)
            if latency > self.canary_metrics.canary_p99_latency {
                self.canary_metrics.canary_p99_latency = latency;
            }
        } else {
            if success {
                self.canary_metrics.production_successes += 1;
            } else {
                self.canary_metrics.production_errors += 1;
            }

            if latency > self.canary_metrics.production_p99_latency {
                self.canary_metrics.production_p99_latency = latency;
            }
        }
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

/// Canary deployment metrics
#[derive(Debug, Clone, Default)]
struct CanaryMetrics {
    canary_requests: u64,
    canary_successes: u64,
    canary_errors: u64,
    canary_p99_latency: std::time::Duration,

    production_requests: u64,
    production_successes: u64,
    production_errors: u64,
    production_p99_latency: std::time::Duration,

    staging_requests: u64,
}

impl CanaryMetrics {
    fn record_canary_request(&mut self) {
        self.canary_requests += 1;
    }

    fn record_production_request(&mut self) {
        self.production_requests += 1;
    }

    fn record_staging_request(&mut self) {
        self.staging_requests += 1;
    }

    fn get_canary_error_rate(&self) -> f64 {
        if self.canary_requests == 0 {
            return 0.0;
        }
        self.canary_errors as f64 / self.canary_requests as f64
    }

    fn get_production_error_rate(&self) -> f64 {
        if self.production_requests == 0 {
            return 0.0;
        }
        self.production_errors as f64 / self.production_requests as f64
    }

    fn get_slo_compliance(&self) -> f64 {
        let total = self.canary_requests + self.canary_errors;
        if total == 0 {
            return 1.0;
        }
        self.canary_successes as f64 / total as f64
    }
}

/// Canary deployment health report
#[derive(Debug, Clone)]
pub struct CanaryHealth {
    pub traffic_percent: f64,
    pub canary_requests: u64,
    pub production_requests: u64,
    pub canary_error_rate: f64,
    pub production_error_rate: f64,
    pub canary_p99_latency: std::time::Duration,
    pub production_p99_latency: std::time::Duration,
    pub health_score: f64,
    pub recommendation: String,
}

impl CanaryHealth {
    fn not_applicable() -> Self {
        Self {
            traffic_percent: 0.0,
            canary_requests: 0,
            production_requests: 0,
            canary_error_rate: 0.0,
            production_error_rate: 0.0,
            canary_p99_latency: std::time::Duration::ZERO,
            production_p99_latency: std::time::Duration::ZERO,
            health_score: 1.0,
            recommendation: "N/A - Not in canary mode".to_string(),
        }
    }
}