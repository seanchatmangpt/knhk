//! Fortune 5 Integration Implementation
//!
//! Main integration logic for Fortune 5 enterprise features.

use crate::error::{WorkflowError, WorkflowResult};
use crate::integration::fortune5::config::*;
use crate::integration::fortune5::slo::{RuntimeClass, SloManager};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Promotion gate state
#[derive(Debug, Clone)]
struct PromotionState {
    /// Current environment
    environment: Environment,
    /// Active feature flags
    feature_flags: HashSet<String>,
    /// Last rollback time (Unix timestamp)
    last_rollback_time: Option<u64>,
}

/// Fortune 5 integration manager
pub struct Fortune5Integration {
    config: Fortune5Config,
    slo_manager: Option<Arc<SloManager>>,
    promotion_state: Arc<RwLock<PromotionState>>,
}

impl Fortune5Integration {
    /// Create new Fortune 5 integration
    pub fn new(config: Fortune5Config) -> Self {
        // Validate SLO config if present
        if let Some(ref slo_config) = config.slo {
            if let Err(e) = slo_config.validate() {
                // Log error but don't fail - use defaults
                tracing::warn!("Invalid SLO config: {}, using defaults", e);
            }
        }

        let slo_manager = config
            .slo
            .as_ref()
            .map(|slo| Arc::new(SloManager::new(slo.clone())));

        let promotion_state = Arc::new(RwLock::new(PromotionState {
            environment: config
                .promotion
                .as_ref()
                .map(|p| p.environment)
                .unwrap_or(Environment::Development),
            feature_flags: config
                .promotion
                .as_ref()
                .map(|p| p.feature_flags.iter().cloned().collect())
                .unwrap_or_default(),
            last_rollback_time: None,
        }));

        Self {
            config,
            slo_manager,
            promotion_state,
        }
    }

    /// Record SLO metric
    pub async fn record_slo_metric(&self, runtime_class: RuntimeClass, latency_ns: u64) {
        if let Some(ref slo_manager) = self.slo_manager {
            slo_manager.record_metric(runtime_class, latency_ns).await;
        }
    }

    /// Check SLO compliance
    pub async fn check_slo_compliance(&self) -> WorkflowResult<bool> {
        let Some(ref slo_manager) = self.slo_manager else {
            return Ok(true); // No SLO configured, always compliant
        };

        Ok(slo_manager.check_compliance().await)
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
                let now_secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                    .as_secs();
                let elapsed = now_secs.saturating_sub(last_rollback);
                if elapsed < promotion_config.rollback_window_seconds {
                    return Ok(false); // Still in rollback window
                }
            }
        }

        // Check environment-specific rules
        match promotion_config.environment {
            Environment::Staging => Ok(true),     // Staging: always allow
            Environment::Development => Ok(true), // Development: always allow
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
        state.feature_flags.contains(feature)
    }
}
