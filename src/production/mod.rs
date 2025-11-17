// KNHK Production Module - Fortune 500 Enterprise Runtime
// Phase 5: Complete production platform with all subsystems integrated
// This module provides the production-grade runtime for real-world deployments

pub mod platform;
pub mod persistence;
pub mod observability;
pub mod monitoring;
pub mod recovery;
pub mod scaling;
pub mod learning;
pub mod cost_tracking;

pub use platform::{ProductionPlatform, PlatformConfig, PlatformState, WorkflowState};
pub use persistence::{PersistenceLayer, ReceiptRecord};
pub use observability::{ObservabilityLayer, Telemetry};
pub use monitoring::{MonitoringLayer, SLATracker};
pub use recovery::{RecoveryManager, StateSnapshot};
pub use scaling::{ScalingManager, ClusterNode};
pub use learning::{LearningEngine, PatternRecognition};
pub use cost_tracking::{CostTracker, ResourceUsage};

/// Production deployment configuration
#[derive(Debug, Clone)]
pub struct ProductionConfig {
    pub platform: PlatformConfig,
    pub enable_persistence: bool,
    pub enable_observability: bool,
    pub enable_monitoring: bool,
    pub enable_recovery: bool,
    pub enable_scaling: bool,
    pub enable_learning: bool,
    pub enable_cost_tracking: bool,
}

impl Default for ProductionConfig {
    fn default() -> Self {
        Self {
            platform: PlatformConfig::default(),
            enable_persistence: true,
            enable_observability: true,
            enable_monitoring: true,
            enable_recovery: true,
            enable_scaling: true,
            enable_learning: true,
            enable_cost_tracking: true,
        }
    }
}

/// Production readiness checklist
pub struct ReadinessCheck {
    pub persistence_ready: bool,
    pub observability_ready: bool,
    pub monitoring_ready: bool,
    pub recovery_ready: bool,
    pub scaling_ready: bool,
    pub learning_ready: bool,
    pub cost_tracking_ready: bool,
    pub sla_configured: bool,
    pub budget_configured: bool,
    pub alerts_configured: bool,
}

impl ReadinessCheck {
    pub fn is_ready(&self) -> bool {
        self.persistence_ready &&
        self.observability_ready &&
        self.monitoring_ready &&
        self.recovery_ready &&
        self.scaling_ready &&
        self.learning_ready &&
        self.cost_tracking_ready &&
        self.sla_configured &&
        self.budget_configured &&
        self.alerts_configured
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_config_default() {
        let config = ProductionConfig::default();
        assert!(config.enable_persistence);
        assert!(config.enable_observability);
        assert!(config.enable_monitoring);
    }

    #[test]
    fn test_readiness_check() {
        let check = ReadinessCheck {
            persistence_ready: true,
            observability_ready: true,
            monitoring_ready: true,
            recovery_ready: true,
            scaling_ready: true,
            learning_ready: true,
            cost_tracking_ready: true,
            sla_configured: true,
            budget_configured: true,
            alerts_configured: true,
        };
        assert!(check.is_ready());
    }
}