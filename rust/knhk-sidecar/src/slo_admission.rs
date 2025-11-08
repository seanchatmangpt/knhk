// knhk-sidecar: SLO-based admission control for Fortune 5
// Enforces R1/W1/C1 runtime class SLOs

use crate::error::{SidecarError, SidecarResult};
use std::collections::VecDeque;
use std::time::Duration;
use tracing::{error, info, warn};

/// SLO configuration for Fortune 5
#[derive(Debug, Clone)]
pub struct SloConfig {
    /// R1 (Hot) p99 max latency in nanoseconds (default: 2ns)
    pub r1_p99_max_ns: u64,
    /// W1 (Warm) p99 max latency in milliseconds (default: 1ms)
    pub w1_p99_max_ms: u64,
    /// C1 (Cold) p99 max latency in milliseconds (default: 500ms)
    pub c1_p99_max_ms: u64,
    /// Admission strategy
    pub admission_strategy: AdmissionStrategy,
}

impl Default for SloConfig {
    fn default() -> Self {
        Self {
            r1_p99_max_ns: 2,   // Fortune 5 requirement: ≤2ns
            w1_p99_max_ms: 1,   // Fortune 5 requirement: ≤1ms
            c1_p99_max_ms: 500, // Fortune 5 requirement: ≤500ms
            admission_strategy: AdmissionStrategy::Strict,
        }
    }
}

impl SloConfig {
    /// Validate SLO configuration
    pub fn validate(&self) -> SidecarResult<()> {
        if self.r1_p99_max_ns > 2 {
            return Err(SidecarError::config_error(format!(
                "R1 p99 max {}ns exceeds Fortune 5 requirement of 2ns",
                self.r1_p99_max_ns
            )));
        }

        if self.w1_p99_max_ms > 1 {
            return Err(SidecarError::config_error(format!(
                "W1 p99 max {}ms exceeds Fortune 5 requirement of 1ms",
                self.w1_p99_max_ms
            )));
        }

        if self.c1_p99_max_ms > 500 {
            return Err(SidecarError::config_error(format!(
                "C1 p99 max {}ms exceeds Fortune 5 requirement of 500ms",
                self.c1_p99_max_ms
            )));
        }

        Ok(())
    }
}

/// Admission strategy
#[derive(Debug, Clone, Copy)]
pub enum AdmissionStrategy {
    /// Strict: Reject if SLO cannot be met
    Strict,
    /// Degrade: Park to lower tier if SLO cannot be met
    Degrade,
}

/// Runtime class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeClass {
    /// R1: Hot path (≤2ns, ≤8 ticks)
    R1,
    /// W1: Warm path (≤1ms)
    W1,
    /// C1: Cold path (≤500ms)
    C1,
}

impl RuntimeClass {
    /// Get SLO for this runtime class
    pub fn get_slo(&self, config: &SloConfig) -> Duration {
        match self {
            RuntimeClass::R1 => Duration::from_nanos(config.r1_p99_max_ns),
            RuntimeClass::W1 => Duration::from_millis(config.w1_p99_max_ms),
            RuntimeClass::C1 => Duration::from_millis(config.c1_p99_max_ms),
        }
    }

    /// Get next lower tier (for degradation)
    pub fn lower_tier(&self) -> Option<RuntimeClass> {
        match self {
            RuntimeClass::R1 => Some(RuntimeClass::W1),
            RuntimeClass::W1 => Some(RuntimeClass::C1),
            RuntimeClass::C1 => None,
        }
    }
}

/// SLO-based admission controller
///
/// Enforces Fortune 5 SLO requirements for R1/W1/C1 runtime classes.
pub struct SloAdmissionController {
    config: SloConfig,
    metrics: AdmissionMetrics,
    /// Historical latency tracking for accurate estimation
    latency_history: std::collections::VecDeque<(RuntimeClass, Duration)>,
    max_history_size: usize,
}

impl SloAdmissionController {
    /// Create new SLO admission controller
    pub fn new(config: SloConfig) -> SidecarResult<Self> {
        config.validate()?;

        Ok(Self {
            config,
            metrics: AdmissionMetrics::default(),
            latency_history: VecDeque::with_capacity(1000),
            max_history_size: 1000,
        })
    }

    /// Record actual latency for a runtime class
    ///
    /// This is called after request processing to track actual performance.
    pub fn record_latency(&mut self, class: RuntimeClass, latency: Duration) {
        self.latency_history.push_back((class, latency));
        if self.latency_history.len() > self.max_history_size {
            self.latency_history.pop_front();
        }
    }

    /// Estimate latency for a runtime class based on historical data
    ///
    /// Returns p99 latency estimate based on historical measurements.
    pub fn estimate_latency(&self, class: RuntimeClass) -> Duration {
        // Filter history for this class
        let mut latencies: Vec<Duration> = self
            .latency_history
            .iter()
            .filter_map(|(c, d)| if *c == class { Some(*d) } else { None })
            .collect();

        if latencies.is_empty() {
            // No history, return SLO as conservative estimate
            return class.get_slo(&self.config);
        }

        // Calculate p99 latency
        latencies.sort();
        let p99_index = (latencies.len() as f64 * 0.99).ceil() as usize - 1;
        let p99_index = p99_index.min(latencies.len() - 1);

        latencies[p99_index]
    }

    /// Check if request can be admitted to runtime class
    ///
    /// If estimated_latency is None, uses historical data to estimate.
    ///
    /// Returns:
    /// - Ok(Some(class)): Admitted to specified or degraded class
    /// - Ok(None): Rejected (SLO cannot be met)
    /// - Err: Error during admission check
    pub fn check_admission(
        &mut self,
        requested_class: RuntimeClass,
        estimated_latency: Option<Duration>,
    ) -> SidecarResult<Option<RuntimeClass>> {
        // Use provided estimate or calculate from history
        let estimated = estimated_latency.unwrap_or_else(|| self.estimate_latency(requested_class));
        let slo = requested_class.get_slo(&self.config);

        // Check if requested class can meet SLO
        if estimated <= slo {
            self.metrics.admitted(requested_class);
            return Ok(Some(requested_class));
        }

        // SLO cannot be met in requested class
        match self.config.admission_strategy {
            AdmissionStrategy::Strict => {
                self.metrics.rejected(requested_class);
                warn!(
                    "Request rejected: {} cannot meet SLO (estimated: {:?}, required: {:?})",
                    format!("{:?}", requested_class),
                    estimated_latency,
                    slo
                );
                Ok(None)
            }
            AdmissionStrategy::Degrade => {
                // Try lower tier
                if let Some(lower_tier) = requested_class.lower_tier() {
                    let lower_slo = lower_tier.get_slo(&self.config);
                    if estimated <= lower_slo {
                        self.metrics.degraded(requested_class, lower_tier);
                        info!(
                            "Request degraded from {:?} to {:?} to meet SLO",
                            requested_class, lower_tier
                        );
                        return Ok(Some(lower_tier));
                    }
                }

                // Cannot meet SLO even in lowest tier
                self.metrics.rejected(requested_class);
                error!(
                    "Request rejected: Cannot meet SLO in any tier (estimated: {:?})",
                    estimated_latency
                );
                Ok(None)
            }
        }
    }

    /// Get admission metrics
    pub fn get_metrics(&self) -> &AdmissionMetrics {
        &self.metrics
    }
}

/// Admission metrics
#[derive(Debug, Clone, Default)]
pub struct AdmissionMetrics {
    pub r1_admitted: u64,
    pub r1_rejected: u64,
    pub r1_degraded_to_w1: u64,
    pub w1_admitted: u64,
    pub w1_rejected: u64,
    pub w1_degraded_to_c1: u64,
    pub c1_admitted: u64,
    pub c1_rejected: u64,
}

impl AdmissionMetrics {
    fn admitted(&mut self, class: RuntimeClass) {
        match class {
            RuntimeClass::R1 => self.r1_admitted += 1,
            RuntimeClass::W1 => self.w1_admitted += 1,
            RuntimeClass::C1 => self.c1_admitted += 1,
        }
    }

    fn rejected(&mut self, class: RuntimeClass) {
        match class {
            RuntimeClass::R1 => self.r1_rejected += 1,
            RuntimeClass::W1 => self.w1_rejected += 1,
            RuntimeClass::C1 => self.c1_rejected += 1,
        }
    }

    fn degraded(&mut self, from: RuntimeClass, to: RuntimeClass) {
        match (from, to) {
            (RuntimeClass::R1, RuntimeClass::W1) => self.r1_degraded_to_w1 += 1,
            (RuntimeClass::W1, RuntimeClass::C1) => self.w1_degraded_to_c1 += 1,
            _ => {} // Other degradations not tracked
        }
    }
}
