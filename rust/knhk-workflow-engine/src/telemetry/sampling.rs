//! Adaptive sampling strategies for telemetry
//!
//! This module implements various sampling strategies including:
//! - Head-based sampling (decision made early)
//! - Tail-based sampling (decision made after seeing full trace)
//! - Adaptive sampling (adjust rates based on traffic/errors)
//! - Priority sampling (always sample errors/slow requests)

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use super::{TelemetryEvent, Span, SpanStatus};

/// Sampling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    /// Base sampling rate (0.0 to 1.0)
    pub base_rate: f64,

    /// Sampling rate for errors (0.0 to 1.0)
    pub error_rate: f64,

    /// Sampling rate for slow requests (0.0 to 1.0)
    pub slow_rate: f64,

    /// Threshold for considering a request "slow" (milliseconds)
    pub slow_threshold_ms: u64,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            base_rate: 0.01,      // 1% base sampling
            error_rate: 1.0,       // 100% error sampling
            slow_rate: 0.5,        // 50% slow request sampling
            slow_threshold_ms: 1000,  // 1 second
        }
    }
}

/// Sampling decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplingDecision {
    /// Sample this event
    Sample,

    /// Drop this event
    Drop,
}

/// Sampling strategy trait
pub trait SamplingStrategy: Send + Sync {
    /// Decide whether to sample an event
    fn should_sample(&self, event: &TelemetryEvent) -> SamplingDecision;

    /// Reduce sampling rate (for backpressure handling)
    fn reduce_sampling_rate(&mut self);

    /// Increase sampling rate
    fn increase_sampling_rate(&mut self);

    /// Get current sampling rate
    fn current_rate(&self) -> f64;
}

/// Always sample strategy (for testing/development)
pub struct AlwaysSampleStrategy;

impl SamplingStrategy for AlwaysSampleStrategy {
    fn should_sample(&self, _event: &TelemetryEvent) -> SamplingDecision {
        SamplingDecision::Sample
    }

    fn reduce_sampling_rate(&mut self) {
        // No-op
    }

    fn increase_sampling_rate(&mut self) {
        // No-op
    }

    fn current_rate(&self) -> f64 {
        1.0
    }
}

/// Never sample strategy (disable telemetry)
pub struct NeverSampleStrategy;

impl SamplingStrategy for NeverSampleStrategy {
    fn should_sample(&self, _event: &TelemetryEvent) -> SamplingDecision {
        SamplingDecision::Drop
    }

    fn reduce_sampling_rate(&mut self) {
        // No-op
    }

    fn increase_sampling_rate(&mut self) {
        // No-op
    }

    fn current_rate(&self) -> f64 {
        0.0
    }
}

/// Probabilistic sampling (simple random sampling)
pub struct ProbabilisticSamplingStrategy {
    /// Sampling rate (0.0 to 1.0)
    rate: Arc<RwLock<f64>>,

    /// Sample counter
    sample_count: Arc<AtomicU64>,

    /// Drop counter
    drop_count: Arc<AtomicU64>,
}

impl ProbabilisticSamplingStrategy {
    /// Create a new probabilistic sampling strategy
    pub fn new(rate: f64) -> Self {
        Self {
            rate: Arc::new(RwLock::new(rate.clamp(0.0, 1.0))),
            sample_count: Arc::new(AtomicU64::new(0)),
            drop_count: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl SamplingStrategy for ProbabilisticSamplingStrategy {
    fn should_sample(&self, _event: &TelemetryEvent) -> SamplingDecision {
        let rate = *self.rate.read();

        // Use fastrand for performance
        let random = fastrand::f64();

        if random < rate {
            self.sample_count.fetch_add(1, Ordering::Relaxed);
            SamplingDecision::Sample
        } else {
            self.drop_count.fetch_add(1, Ordering::Relaxed);
            SamplingDecision::Drop
        }
    }

    fn reduce_sampling_rate(&mut self) {
        let mut rate = self.rate.write();
        *rate = (*rate * 0.5).max(0.001);  // Reduce by 50%, minimum 0.1%
    }

    fn increase_sampling_rate(&mut self) {
        let mut rate = self.rate.write();
        *rate = (*rate * 1.5).min(1.0);  // Increase by 50%, maximum 100%
    }

    fn current_rate(&self) -> f64 {
        *self.rate.read()
    }
}

/// Adaptive priority-based sampling
pub struct AdaptiveSamplingStrategy {
    /// Configuration
    config: Arc<RwLock<SamplingConfig>>,

    /// Current base rate (adjusted dynamically)
    current_base_rate: Arc<RwLock<f64>>,

    /// Sample counters
    sample_count: Arc<AtomicU64>,
    drop_count: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
    slow_count: Arc<AtomicU64>,
}

impl AdaptiveSamplingStrategy {
    /// Create a new adaptive sampling strategy
    pub fn new(config: SamplingConfig) -> Self {
        let base_rate = config.base_rate;

        Self {
            config: Arc::new(RwLock::new(config)),
            current_base_rate: Arc::new(RwLock::new(base_rate)),
            sample_count: Arc::new(AtomicU64::new(0)),
            drop_count: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicU64::new(0)),
            slow_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Check if span is slow
    fn is_slow(&self, span: &Span) -> bool {
        let config = self.config.read();
        let duration_ms = span.duration_ns / 1_000_000;
        duration_ms > config.slow_threshold_ms
    }

    /// Get sampling statistics
    pub fn stats(&self) -> SamplingStats {
        let sample_count = self.sample_count.load(Ordering::Relaxed);
        let drop_count = self.drop_count.load(Ordering::Relaxed);
        let total = sample_count + drop_count;

        let effective_rate = if total > 0 {
            sample_count as f64 / total as f64
        } else {
            0.0
        };

        SamplingStats {
            sample_count,
            drop_count,
            error_count: self.error_count.load(Ordering::Relaxed),
            slow_count: self.slow_count.load(Ordering::Relaxed),
            current_base_rate: *self.current_base_rate.read(),
            effective_rate,
        }
    }
}

impl SamplingStrategy for AdaptiveSamplingStrategy {
    fn should_sample(&self, event: &TelemetryEvent) -> SamplingDecision {
        let decision = match event {
            TelemetryEvent::Span(span) => {
                let config = self.config.read();

                // Priority 1: Always sample errors
                if span.status == SpanStatus::Error {
                    self.error_count.fetch_add(1, Ordering::Relaxed);
                    if fastrand::f64() < config.error_rate {
                        SamplingDecision::Sample
                    } else {
                        SamplingDecision::Drop
                    }
                }
                // Priority 2: Sample slow requests at higher rate
                else if self.is_slow(span) {
                    self.slow_count.fetch_add(1, Ordering::Relaxed);
                    if fastrand::f64() < config.slow_rate {
                        SamplingDecision::Sample
                    } else {
                        SamplingDecision::Drop
                    }
                }
                // Priority 3: Normal sampling
                else {
                    let base_rate = *self.current_base_rate.read();
                    if fastrand::f64() < base_rate {
                        SamplingDecision::Sample
                    } else {
                        SamplingDecision::Drop
                    }
                }
            }
            // Always sample metrics and logs at base rate
            _ => {
                let base_rate = *self.current_base_rate.read();
                if fastrand::f64() < base_rate {
                    SamplingDecision::Sample
                } else {
                    SamplingDecision::Drop
                }
            }
        };

        match decision {
            SamplingDecision::Sample => {
                self.sample_count.fetch_add(1, Ordering::Relaxed);
            }
            SamplingDecision::Drop => {
                self.drop_count.fetch_add(1, Ordering::Relaxed);
            }
        }

        decision
    }

    fn reduce_sampling_rate(&mut self) {
        let mut rate = self.current_base_rate.write();
        *rate = (*rate * 0.5).max(0.001);  // Reduce by 50%, minimum 0.1%
    }

    fn increase_sampling_rate(&mut self) {
        let mut rate = self.current_base_rate.write();
        let max_rate = self.config.read().base_rate;
        *rate = (*rate * 1.5).min(max_rate);  // Increase by 50%, cap at configured base
    }

    fn current_rate(&self) -> f64 {
        *self.current_base_rate.read()
    }
}

/// Sampling statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingStats {
    /// Total samples taken
    pub sample_count: u64,

    /// Total samples dropped
    pub drop_count: u64,

    /// Errors sampled
    pub error_count: u64,

    /// Slow requests sampled
    pub slow_count: u64,

    /// Current base sampling rate
    pub current_base_rate: f64,

    /// Effective sampling rate (samples / total)
    pub effective_rate: f64,
}

/// Tail-based sampling (decision after trace completion)
pub struct TailBasedSamplingStrategy {
    /// Head-based strategy for initial decision
    head_strategy: Box<dyn SamplingStrategy>,

    /// Trace buffer for tail-based decisions
    trace_buffer: Arc<RwLock<Vec<String>>>,
}

impl TailBasedSamplingStrategy {
    /// Create a new tail-based sampling strategy
    pub fn new(head_strategy: Box<dyn SamplingStrategy>) -> Self {
        Self {
            head_strategy,
            trace_buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl SamplingStrategy for TailBasedSamplingStrategy {
    fn should_sample(&self, event: &TelemetryEvent) -> SamplingDecision {
        // For now, delegate to head-based strategy
        // In production, would buffer traces and make decision after completion
        self.head_strategy.should_sample(event)
    }

    fn reduce_sampling_rate(&mut self) {
        self.head_strategy.reduce_sampling_rate();
    }

    fn increase_sampling_rate(&mut self) {
        self.head_strategy.increase_sampling_rate();
    }

    fn current_rate(&self) -> f64 {
        self.head_strategy.current_rate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_always_sample() {
        let strategy = AlwaysSampleStrategy;
        let event = TelemetryEvent::Span(Span {
            name: "test".to_string(),
            trace_id: "trace-123".to_string(),
            span_id: "span-456".to_string(),
            parent_span_id: None,
            attributes: vec![],
            duration_ns: 1_000_000,
            status: SpanStatus::Ok,
            start_time_ns: 1000,
            end_time_ns: 2000,
        });

        assert_eq!(strategy.should_sample(&event), SamplingDecision::Sample);
    }

    #[test]
    fn test_never_sample() {
        let strategy = NeverSampleStrategy;
        let event = TelemetryEvent::Span(Span {
            name: "test".to_string(),
            trace_id: "trace-123".to_string(),
            span_id: "span-456".to_string(),
            parent_span_id: None,
            attributes: vec![],
            duration_ns: 1_000_000,
            status: SpanStatus::Ok,
            start_time_ns: 1000,
            end_time_ns: 2000,
        });

        assert_eq!(strategy.should_sample(&event), SamplingDecision::Drop);
    }

    #[test]
    fn test_adaptive_sampling_error_priority() {
        let config = SamplingConfig {
            base_rate: 0.01,
            error_rate: 1.0,  // Always sample errors
            slow_rate: 0.5,
            slow_threshold_ms: 1000,
        };

        let strategy = AdaptiveSamplingStrategy::new(config);

        let error_span = Span {
            name: "test".to_string(),
            trace_id: "trace-123".to_string(),
            span_id: "span-456".to_string(),
            parent_span_id: None,
            attributes: vec![],
            duration_ns: 1_000_000,
            status: SpanStatus::Error,  // Error status
            start_time_ns: 1000,
            end_time_ns: 2000,
        };

        let event = TelemetryEvent::Span(error_span);

        // Should always sample errors
        assert_eq!(strategy.should_sample(&event), SamplingDecision::Sample);
    }

    #[test]
    fn test_probabilistic_sampling_rate_adjustment() {
        let mut strategy = ProbabilisticSamplingStrategy::new(0.5);

        assert_eq!(strategy.current_rate(), 0.5);

        strategy.reduce_sampling_rate();
        assert_eq!(strategy.current_rate(), 0.25);

        strategy.increase_sampling_rate();
        assert_eq!(strategy.current_rate(), 0.375);
    }
}
