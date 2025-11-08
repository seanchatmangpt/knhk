//! SLO (Service Level Objective) tracking and enforcement
//!
//! Provides SLO metrics tracking and compliance checking for Fortune 5 deployments.
//!
//! # Runtime Classes
//!
//! - **R1**: Hot path operations (≤2ns P99)
//! - **W1**: Warm path operations (≤1ms P99)
//! - **C1**: Cold path operations (≤500ms P99)

use crate::error::WorkflowResult;
use crate::integration::fortune5::config::SloConfig;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Runtime class for SLO tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeClass {
    /// R1: Hot path (≤2ns P99)
    R1,
    /// W1: Warm path (≤1ms P99)
    W1,
    /// C1: Cold path (≤500ms P99)
    C1,
}

/// SLO metrics
#[derive(Debug, Default)]
pub struct SloMetrics {
    /// R1 latency samples (nanoseconds)
    pub r1_samples: Vec<u64>,
    /// W1 latency samples (milliseconds)
    pub w1_samples: Vec<u64>,
    /// C1 latency samples (milliseconds)
    pub c1_samples: Vec<u64>,
}

impl SloMetrics {
    /// Record SLO metric
    pub fn record_metric(&mut self, runtime_class: RuntimeClass, latency_ns: u64) {
        match runtime_class {
            RuntimeClass::R1 => {
                self.r1_samples.push(latency_ns);
                // Keep only last 1000 samples for memory efficiency
                if self.r1_samples.len() > 1000 {
                    self.r1_samples.remove(0);
                }
            }
            RuntimeClass::W1 => {
                let latency_ms = latency_ns / 1_000_000;
                self.w1_samples.push(latency_ms);
                if self.w1_samples.len() > 1000 {
                    self.w1_samples.remove(0);
                }
            }
            RuntimeClass::C1 => {
                let latency_ms = latency_ns / 1_000_000;
                self.c1_samples.push(latency_ms);
                if self.c1_samples.len() > 1000 {
                    self.c1_samples.remove(0);
                }
            }
        }
    }

    /// Check SLO compliance
    pub fn check_compliance(&self, slo_config: &SloConfig) -> bool {
        // Calculate P99 for each runtime class
        let r1_p99 = Self::calculate_p99(&self.r1_samples);
        let w1_p99 = self.w1_samples.iter().max().copied().unwrap_or(0) as u64;
        let c1_p99 = self.c1_samples.iter().max().copied().unwrap_or(0) as u64;

        let r1_compliant = r1_p99 <= slo_config.r1_p99_max_ns;
        let w1_compliant = w1_p99 <= slo_config.w1_p99_max_ms;
        let c1_compliant = c1_p99 <= slo_config.c1_p99_max_ms;

        r1_compliant && w1_compliant && c1_compliant
    }

    /// Calculate P99 percentile
    fn calculate_p99(samples: &[u64]) -> u64 {
        if samples.is_empty() {
            return 0;
        }
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let index = ((sorted.len() as f64) * 0.99).ceil() as usize - 1;
        sorted
            .get(index.min(sorted.len() - 1))
            .copied()
            .unwrap_or(0)
    }
}

/// SLO manager
pub struct SloManager {
    config: Arc<SloConfig>,
    metrics: Arc<RwLock<SloMetrics>>,
}

impl SloManager {
    /// Create new SLO manager
    pub fn new(config: SloConfig) -> Self {
        Self {
            config: Arc::new(config),
            metrics: Arc::new(RwLock::new(SloMetrics::default())),
        }
    }

    /// Record SLO metric
    pub async fn record_metric(&self, runtime_class: RuntimeClass, latency_ns: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.record_metric(runtime_class, latency_ns);
    }

    /// Check SLO compliance
    pub async fn check_compliance(&self) -> bool {
        let metrics = self.metrics.read().await;
        metrics.check_compliance(&self.config)
    }

    /// Get current P99 metrics
    pub async fn get_p99_metrics(&self) -> (u64, u64, u64) {
        let metrics = self.metrics.read().await;
        let r1_p99 = SloMetrics::calculate_p99(&metrics.r1_samples);
        let w1_p99 = metrics.w1_samples.iter().max().copied().unwrap_or(0) as u64;
        let c1_p99 = metrics.c1_samples.iter().max().copied().unwrap_or(0) as u64;
        (r1_p99, w1_p99, c1_p99)
    }
}

