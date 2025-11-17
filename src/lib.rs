// KNHK - Knowledge Navigation & Hypothesis Kinetics
// Phase 5: Production-Ready Fortune 500 Platform
// Complete implementation with all 6 covenants and production subsystems

pub mod autonomic;
pub mod production;

// Fortune 500 RevOps Avatar System
pub mod avatars;
pub mod knhk_client;
pub mod scenarios;
pub mod results;

// Re-export key types for convenience
pub use autonomic::{
    Covenant, Receipt, Descriptor, Rule, Pattern,
    O, Sigma, Q, Pi, MAPEK, ChatmanConstant,
};

pub use production::{
    ProductionPlatform, PlatformConfig, PlatformState,
    PersistenceLayer, ReceiptStore,
    ObservabilityLayer, Telemetry,
    MonitoringLayer, SLATracker,
    RecoveryManager, StateSnapshot,
    ScalingManager, ClusterNode,
    LearningEngine, PatternRecognition,
    CostTracker, ResourceUsage,
};

// Re-export RevOps types
pub use avatars::{Avatar, Decision, AuthorityLevel, SLA};
pub use scenarios::DealScenario;
pub use results::{ComprehensiveResults, ScenarioResult};

/// KNHK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Production deployment targets
pub const SLA_TARGET: f64 = 99.99; // 99.99% uptime
pub const LATENCY_TARGET_MS: u64 = 100; // P50 latency
pub const THROUGHPUT_TARGET_RPS: f64 = 1000.0; // Requests per second
pub const COST_REDUCTION_TARGET: f64 = 40.0; // 40% cost reduction vs legacy

/// Initialize KNHK for production
pub async fn initialize_production() -> Result<ProductionPlatform, Box<dyn std::error::Error>> {
    let config = PlatformConfig::default();
    let mut platform = ProductionPlatform::new(config)?;
    platform.start().await?;
    Ok(platform)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_sla_targets() {
        assert_eq!(SLA_TARGET, 99.99);
        assert_eq!(LATENCY_TARGET_MS, 100);
        assert_eq!(THROUGHPUT_TARGET_RPS, 1000.0);
        assert_eq!(COST_REDUCTION_TARGET, 40.0);
    }
}