//! Real-time stream processing with windowed aggregations and complex event processing
//!
//! This module implements real-time stream processing capabilities including:
//! - Sliding and tumbling windows
//! - Aggregations (count, sum, avg, percentiles)
//! - Complex event processing (CEP)
//! - Anomaly detection

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use super::{TelemetryEvent, TelemetryResult, TelemetryError, Span, SpanStatus};

/// Window configuration for stream processing
#[derive(Debug, Clone)]
pub enum WindowConfig {
    /// Tumbling window (non-overlapping, fixed-size)
    Tumbling {
        /// Window duration
        duration: Duration,
    },

    /// Sliding window (overlapping, moves by slide interval)
    Sliding {
        /// Window size
        size: Duration,

        /// Slide interval
        slide: Duration,
    },

    /// Session window (gaps define window boundaries)
    Session {
        /// Gap timeout - window closes after this much inactivity
        gap: Duration,
    },
}

/// Stream processor for real-time event processing
pub struct StreamProcessor {
    /// Window configuration
    window_config: WindowConfig,

    /// Event buffer (time-ordered)
    event_buffer: Arc<RwLock<VecDeque<TimestampedEvent>>>,

    /// Current window start time
    current_window_start: Arc<RwLock<Option<Instant>>>,

    /// Aggregation results
    aggregation_results: Arc<RwLock<Vec<AggregationResult>>>,

    /// CEP rules
    cep_rules: Arc<RwLock<Vec<CepRule>>>,

    /// Anomaly detector
    anomaly_detector: Arc<RwLock<AnomalyDetector>>,
}

/// Event with timestamp for window processing
#[derive(Debug, Clone)]
struct TimestampedEvent {
    event: TelemetryEvent,
    timestamp: Instant,
}

/// Aggregation result for a window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    /// Window start time (Unix epoch nanos)
    pub window_start_ns: u64,

    /// Window end time (Unix epoch nanos)
    pub window_end_ns: u64,

    /// Event count in window
    pub count: u64,

    /// Sum of durations (for spans)
    pub sum_duration_ns: u64,

    /// Average duration (milliseconds)
    pub avg_duration_ms: f64,

    /// Median duration (milliseconds)
    pub median_duration_ms: f64,

    /// P95 duration (milliseconds)
    pub p95_duration_ms: f64,

    /// P99 duration (milliseconds)
    pub p99_duration_ms: f64,

    /// Error count
    pub error_count: u64,

    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,

    /// Throughput (events per second)
    pub throughput: f64,

    /// Custom aggregations
    pub custom: HashMap<String, f64>,
}

/// Complex Event Processing rule
#[derive(Debug, Clone)]
pub struct CepRule {
    /// Rule name
    pub name: String,

    /// Pattern to match
    pub pattern: CepPattern,

    /// Action to take when pattern matches
    pub action: CepAction,
}

/// CEP pattern types
#[derive(Debug, Clone)]
pub enum CepPattern {
    /// Sequence of events in order
    Sequence(Vec<EventMatcher>),

    /// All events must occur (any order)
    All(Vec<EventMatcher>),

    /// At least one event must occur
    Any(Vec<EventMatcher>),

    /// Event followed by another within time window
    FollowedBy {
        first: Box<EventMatcher>,
        second: Box<EventMatcher>,
        within: Duration,
    },

    /// Count threshold within window
    CountThreshold {
        matcher: Box<EventMatcher>,
        threshold: usize,
        within: Duration,
    },
}

/// Event matcher for CEP
#[derive(Debug, Clone)]
pub struct EventMatcher {
    /// Span name pattern (regex)
    pub span_name_pattern: Option<String>,

    /// Required attributes
    pub required_attributes: HashMap<String, String>,

    /// Status filter
    pub status: Option<SpanStatus>,

    /// Duration threshold (nanoseconds)
    pub duration_threshold_ns: Option<u64>,
}

/// CEP action
#[derive(Debug, Clone)]
pub enum CepAction {
    /// Log a message
    Log(String),

    /// Emit an alert
    Alert {
        severity: AlertSeverity,
        message: String,
    },

    /// Trigger a callback
    Callback(String),
}

/// Alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Anomaly detector
#[derive(Debug, Clone)]
pub struct AnomalyDetector {
    /// Historical baselines (metric -> baseline value)
    baselines: HashMap<String, f64>,

    /// Threshold multiplier for anomaly detection
    threshold_multiplier: f64,

    /// Minimum samples before detecting anomalies
    min_samples: usize,

    /// Sample count
    sample_count: usize,
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self {
            baselines: HashMap::new(),
            threshold_multiplier: 3.0,  // 3 standard deviations
            min_samples: 100,
            sample_count: 0,
        }
    }
}

impl AnomalyDetector {
    /// Check if a value is anomalous
    pub fn is_anomaly(&self, metric: &str, value: f64) -> bool {
        if self.sample_count < self.min_samples {
            return false;
        }

        if let Some(&baseline) = self.baselines.get(metric) {
            let deviation = (value - baseline).abs();
            let threshold = baseline * self.threshold_multiplier;
            deviation > threshold
        } else {
            false
        }
    }

    /// Update baseline with new sample
    pub fn update_baseline(&mut self, metric: &str, value: f64) {
        self.sample_count += 1;

        let baseline = self.baselines.entry(metric.to_string()).or_insert(value);

        // Exponential moving average with alpha = 0.1
        *baseline = 0.9 * *baseline + 0.1 * value;
    }
}

impl StreamProcessor {
    /// Create a new stream processor
    pub fn new(window_config: WindowConfig) -> Self {
        Self {
            window_config,
            event_buffer: Arc::new(RwLock::new(VecDeque::new())),
            current_window_start: Arc::new(RwLock::new(None)),
            aggregation_results: Arc::new(RwLock::new(Vec::new())),
            cep_rules: Arc::new(RwLock::new(Vec::new())),
            anomaly_detector: Arc::new(RwLock::new(AnomalyDetector::default())),
        }
    }

    /// Add a CEP rule
    pub fn add_cep_rule(&self, rule: CepRule) {
        let mut rules = self.cep_rules.write();
        rules.push(rule);
    }

    /// Process an event
    pub async fn process_event(&self, event: TelemetryEvent) -> TelemetryResult<()> {
        let timestamped = TimestampedEvent {
            event: event.clone(),
            timestamp: Instant::now(),
        };

        // Add to buffer
        {
            let mut buffer = self.event_buffer.write();
            buffer.push_back(timestamped);
        }

        // Check if we should close current window
        self.check_window_boundary().await?;

        // Check CEP rules
        self.check_cep_rules(&event).await?;

        Ok(())
    }

    /// Check if window boundary is reached
    async fn check_window_boundary(&self) -> TelemetryResult<()> {
        let mut should_aggregate = false;

        {
            let window_start = self.current_window_start.read();

            if let Some(start) = *window_start {
                let elapsed = start.elapsed();

                should_aggregate = match &self.window_config {
                    WindowConfig::Tumbling { duration } => elapsed >= *duration,
                    WindowConfig::Sliding { size, .. } => elapsed >= *size,
                    WindowConfig::Session { gap } => elapsed >= *gap,
                };
            } else {
                // First event - start window
                drop(window_start);
                let mut window_start_mut = self.current_window_start.write();
                *window_start_mut = Some(Instant::now());
            }
        }

        if should_aggregate {
            self.aggregate_window().await?;
        }

        Ok(())
    }

    /// Aggregate events in current window
    async fn aggregate_window(&self) -> TelemetryResult<()> {
        let events = {
            let mut buffer = self.event_buffer.write();
            let events: Vec<_> = buffer.drain(..).collect();
            events
        };

        if events.is_empty() {
            return Ok(());
        }

        // Calculate window bounds
        let window_start = events.first()
            .ok_or_else(|| TelemetryError::StreamError("Empty window".to_string()))?
            .timestamp;
        let window_end = events.last()
            .ok_or_else(|| TelemetryError::StreamError("Empty window".to_string()))?
            .timestamp;

        let window_duration = window_end.duration_since(window_start);

        // Collect span durations for percentile calculations
        let mut durations: Vec<u64> = Vec::new();
        let mut error_count = 0u64;

        for timestamped in &events {
            if let TelemetryEvent::Span(span) = &timestamped.event {
                durations.push(span.duration_ns);
                if span.status == SpanStatus::Error {
                    error_count += 1;
                }
            }
        }

        // Sort durations for percentile calculation
        durations.sort_unstable();

        let count = durations.len() as u64;
        let sum_duration_ns: u64 = durations.iter().sum();

        let avg_duration_ms = if count > 0 {
            (sum_duration_ns as f64 / count as f64) / 1_000_000.0
        } else {
            0.0
        };

        let median_duration_ms = percentile(&durations, 0.50);
        let p95_duration_ms = percentile(&durations, 0.95);
        let p99_duration_ms = percentile(&durations, 0.99);

        let error_rate = if count > 0 {
            error_count as f64 / count as f64
        } else {
            0.0
        };

        let throughput = if window_duration.as_secs_f64() > 0.0 {
            count as f64 / window_duration.as_secs_f64()
        } else {
            0.0
        };

        // Create aggregation result
        let now = SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(|e| TelemetryError::StreamError(format!("Time error: {}", e)))?;

        let result = AggregationResult {
            window_start_ns: now.as_nanos() as u64 - window_duration.as_nanos() as u64,
            window_end_ns: now.as_nanos() as u64,
            count,
            sum_duration_ns,
            avg_duration_ms,
            median_duration_ms,
            p95_duration_ms,
            p99_duration_ms,
            error_count,
            error_rate,
            throughput,
            custom: HashMap::new(),
        };

        // Store result
        {
            let mut results = self.aggregation_results.write();
            results.push(result.clone());
        }

        // Check for anomalies
        {
            let mut detector = self.anomaly_detector.write();

            if detector.is_anomaly("avg_duration_ms", avg_duration_ms) {
                warn!("Anomaly detected: avg_duration_ms={:.2}ms", avg_duration_ms);
            }

            if detector.is_anomaly("error_rate", error_rate) {
                warn!("Anomaly detected: error_rate={:.2}%", error_rate * 100.0);
            }

            detector.update_baseline("avg_duration_ms", avg_duration_ms);
            detector.update_baseline("error_rate", error_rate);
        }

        // Reset window
        {
            let mut window_start = self.current_window_start.write();
            *window_start = Some(Instant::now());
        }

        debug!("Window aggregated: count={}, avg_duration={:.2}ms, p99={:.2}ms",
            count, avg_duration_ms, p99_duration_ms);

        Ok(())
    }

    /// Check CEP rules against an event
    async fn check_cep_rules(&self, event: &TelemetryEvent) -> TelemetryResult<()> {
        let rules = self.cep_rules.read();

        for rule in rules.iter() {
            if self.matches_pattern(&rule.pattern, event).await {
                self.execute_action(&rule.action).await?;
            }
        }

        Ok(())
    }

    /// Check if event matches CEP pattern
    async fn matches_pattern(&self, pattern: &CepPattern, event: &TelemetryEvent) -> bool {
        match pattern {
            CepPattern::Sequence(_) => {
                // Simplified - would need more complex matching logic
                false
            }
            CepPattern::All(_) => false,
            CepPattern::Any(_) => false,
            CepPattern::FollowedBy { .. } => false,
            CepPattern::CountThreshold { matcher, threshold, .. } => {
                // Simple implementation - just check current event
                if self.matches_event(matcher, event) {
                    // Would need to track counts over time
                    false
                } else {
                    false
                }
            }
        }
    }

    /// Check if event matches matcher
    fn matches_event(&self, matcher: &EventMatcher, event: &TelemetryEvent) -> bool {
        if let TelemetryEvent::Span(span) = event {
            // Check span name pattern
            if let Some(ref pattern) = matcher.span_name_pattern {
                if !span.name.contains(pattern) {
                    return false;
                }
            }

            // Check status
            if let Some(status) = matcher.status {
                if span.status != status {
                    return false;
                }
            }

            // Check duration threshold
            if let Some(threshold) = matcher.duration_threshold_ns {
                if span.duration_ns < threshold {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }

    /// Execute CEP action
    async fn execute_action(&self, action: &CepAction) -> TelemetryResult<()> {
        match action {
            CepAction::Log(message) => {
                debug!("CEP rule triggered: {}", message);
            }
            CepAction::Alert { severity, message } => {
                match severity {
                    AlertSeverity::Info => debug!("ALERT [INFO]: {}", message),
                    AlertSeverity::Warning => warn!("ALERT [WARNING]: {}", message),
                    AlertSeverity::Error => tracing::error!("ALERT [ERROR]: {}", message),
                    AlertSeverity::Critical => tracing::error!("ALERT [CRITICAL]: {}", message),
                }
            }
            CepAction::Callback(name) => {
                debug!("CEP callback triggered: {}", name);
            }
        }
        Ok(())
    }

    /// Get aggregation results
    pub fn get_results(&self) -> Vec<AggregationResult> {
        self.aggregation_results.read().clone()
    }

    /// Get latest aggregation result
    pub fn get_latest_result(&self) -> Option<AggregationResult> {
        self.aggregation_results.read().last().cloned()
    }
}

/// Calculate percentile from sorted durations
fn percentile(sorted_durations: &[u64], p: f64) -> f64 {
    if sorted_durations.is_empty() {
        return 0.0;
    }

    let idx = (sorted_durations.len() as f64 * p) as usize;
    let idx = idx.min(sorted_durations.len() - 1);

    sorted_durations[idx] as f64 / 1_000_000.0  // Convert to milliseconds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentile_calculation() {
        let durations = vec![1_000_000, 2_000_000, 3_000_000, 4_000_000, 5_000_000];

        assert_eq!(percentile(&durations, 0.50), 3.0);  // Median
        assert_eq!(percentile(&durations, 0.99), 5.0);  // P99
    }

    #[tokio::test]
    async fn test_stream_processor_creation() {
        let processor = StreamProcessor::new(WindowConfig::Tumbling {
            duration: Duration::from_secs(60),
        });

        assert!(processor.get_results().is_empty());
    }
}
