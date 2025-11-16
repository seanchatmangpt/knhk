//! System configuration types

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Storage backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackend {
    /// In-memory storage (for testing)
    InMemory,

    /// RocksDB persistent storage
    RocksDb {
        /// Path to database directory
        path: String,
    },

    /// PostgreSQL storage
    PostgreSQL {
        /// Connection string
        connection_string: String,
    },
}

/// Autonomous loop configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopConfiguration {
    /// Interval between evolution cycles
    pub cycle_interval: Duration,

    /// Maximum number of proposals per cycle
    pub max_proposals: usize,

    /// Maximum change rate (proposals/hour)
    pub max_change_rate: f64,

    /// Failure threshold for self-healing (0.0-1.0)
    pub failure_threshold: f64,

    /// Auto-promote validated changes
    pub auto_promote: bool,
}

impl Default for LoopConfiguration {
    fn default() -> Self {
        Self {
            cycle_interval: Duration::from_secs(crate::DEFAULT_CYCLE_INTERVAL_SECS),
            max_proposals: 10,
            max_change_rate: 1.0,
            failure_threshold: 0.5,
            auto_promote: true,
        }
    }
}

/// Complete system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Storage backend
    pub storage: StorageBackend,

    /// Autonomous loop configuration
    pub loop_config: LoopConfiguration,

    /// Enable τ-axis verification
    pub verify_time_axis: bool,

    /// Enable μ-axis verification
    pub verify_mapping_axis: bool,

    /// Enable Γ-axis verification
    pub verify_glue_axis: bool,

    /// Enable OpenTelemetry
    pub enable_telemetry: bool,

    /// OTLP endpoint for telemetry
    pub otlp_endpoint: Option<String>,

    /// Tick budget for hot path operations
    pub max_hot_path_ticks: u64,

    /// Tick budget for promotion operations
    pub max_promotion_ticks: u64,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            storage: StorageBackend::InMemory,
            loop_config: LoopConfiguration::default(),
            verify_time_axis: true,
            verify_mapping_axis: true,
            verify_glue_axis: true,
            enable_telemetry: true,
            otlp_endpoint: None,
            max_hot_path_ticks: crate::MAX_HOT_PATH_TICKS,
            max_promotion_ticks: crate::MAX_PROMOTION_TICKS,
        }
    }
}

impl SystemConfig {
    /// Create a configuration suitable for testing
    pub fn for_testing() -> Self {
        Self {
            storage: StorageBackend::InMemory,
            loop_config: LoopConfiguration {
                cycle_interval: Duration::from_millis(100),
                max_proposals: 5,
                max_change_rate: 10.0,
                failure_threshold: 0.8,
                auto_promote: false,
            },
            verify_time_axis: true,
            verify_mapping_axis: true,
            verify_glue_axis: true,
            enable_telemetry: false,
            otlp_endpoint: None,
            max_hot_path_ticks: 100, // Generous for tests
            max_promotion_ticks: 200,
        }
    }

    /// Create a production configuration
    pub fn for_production(otlp_endpoint: String) -> Self {
        Self {
            storage: StorageBackend::RocksDb {
                path: "/var/knhk/store".to_string(),
            },
            loop_config: LoopConfiguration::default(),
            verify_time_axis: true,
            verify_mapping_axis: true,
            verify_glue_axis: true,
            enable_telemetry: true,
            otlp_endpoint: Some(otlp_endpoint),
            max_hot_path_ticks: crate::MAX_HOT_PATH_TICKS,
            max_promotion_ticks: crate::MAX_PROMOTION_TICKS,
        }
    }
}
