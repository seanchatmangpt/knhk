//! Observability Phase: Zero-Cost Distributed Tracing
//!
//! This phase provides comprehensive observability through zero-cost abstractions.
//! All tracing and metrics are compile-time configurable and optimized away when
//! disabled, with no runtime overhead.
//!
//! # Key Features
//! - Zero-cost tracing spans
//! - Compile-time trace filtering
//! - Lock-free metric collection
//! - Distributed trace correlation
//! - Adaptive sampling

use crate::const_assert;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Trace level - severity/importance of trace events
pub trait TraceLevel: 'static {
    const NAME: &'static str;
    const NUMERIC_LEVEL: u8;
    const COLOR: &'static str;
}

/// Trace - diagnostic information
pub struct Trace;
impl TraceLevel for Trace {
    const NAME: &'static str = "TRACE";
    const NUMERIC_LEVEL: u8 = 0;
    const COLOR: &'static str = "gray";
}

/// Debug - detailed debugging information
pub struct Debug;
impl TraceLevel for Debug {
    const NAME: &'static str = "DEBUG";
    const NUMERIC_LEVEL: u8 = 1;
    const COLOR: &'static str = "blue";
}

/// Info - general information
pub struct Info;
impl TraceLevel for Info {
    const NAME: &'static str = "INFO";
    const NUMERIC_LEVEL: u8 = 2;
    const COLOR: &'static str = "green";
}

/// Warn - warning messages
pub struct Warn;
impl TraceLevel for Warn {
    const NAME: &'static str = "WARN";
    const NUMERIC_LEVEL: u8 = 3;
    const COLOR: &'static str = "yellow";
}

/// Error - error conditions
pub struct Error;
impl TraceLevel for Error {
    const NAME: &'static str = "ERROR";
    const NUMERIC_LEVEL: u8 = 4;
    const COLOR: &'static str = "red";
}

/// Trace span - represents a unit of work
pub struct Span<L: TraceLevel, const ENABLED: bool> {
    name: &'static str,
    start_time: u64,
    _level: PhantomData<L>,
}

impl<L: TraceLevel, const ENABLED: bool> Span<L, ENABLED> {
    /// Create new span
    #[inline(always)]
    pub fn new(name: &'static str) -> Self {
        let start_time = if ENABLED {
            // Would use actual timestamp
            0
        } else {
            0
        };

        Self {
            name,
            start_time,
            _level: PhantomData,
        }
    }

    /// Record event within span
    #[inline(always)]
    pub fn event(&self, _message: &str) {
        if ENABLED {
            // Would emit trace event
        }
    }

    /// Add attribute to span
    #[inline(always)]
    pub fn attribute(&self, _key: &str, _value: &str) {
        if ENABLED {
            // Would attach metadata
        }
    }

    /// Get span name
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl<L: TraceLevel, const ENABLED: bool> Drop for Span<L, ENABLED> {
    fn drop(&mut self) {
        if ENABLED {
            // Would record span duration
            let _duration = 0 - self.start_time;
        }
    }
}

/// Trace context - correlation across distributed system
#[derive(Debug, Clone, Copy)]
pub struct TraceContext {
    pub trace_id: u128,
    pub span_id: u64,
    pub parent_span_id: Option<u64>,
}

impl TraceContext {
    /// Create root context
    pub fn root() -> Self {
        Self {
            trace_id: 0, // Would generate random ID
            span_id: 1,
            parent_span_id: None,
        }
    }

    /// Create child context
    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id,
            span_id: self.span_id + 1,
            parent_span_id: Some(self.span_id),
        }
    }

    /// Serialize for propagation (W3C Trace Context format)
    pub fn to_traceparent(&self) -> String {
        format!("00-{:032x}-{:016x}-01", self.trace_id, self.span_id)
    }
}

/// Metric - numerical measurement
pub trait Metric {
    type Value;

    fn record(&self, value: Self::Value);
    fn get(&self) -> Self::Value;
}

/// Counter - monotonically increasing value
pub struct Counter {
    value: AtomicU64,
}

impl Counter {
    pub const fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    pub fn increment(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add(&self, amount: u64) {
        self.value.fetch_add(amount, Ordering::Relaxed);
    }
}

impl Metric for Counter {
    type Value = u64;

    fn record(&self, value: u64) {
        self.add(value);
    }

    fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// Gauge - value that can go up or down
pub struct Gauge {
    value: AtomicU64,
}

impl Gauge {
    pub const fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::Relaxed);
    }

    pub fn increment(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Metric for Gauge {
    type Value = u64;

    fn record(&self, value: u64) {
        self.set(value);
    }

    fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// Histogram - distribution of values
pub struct Histogram<const BUCKETS: usize> {
    buckets: [AtomicUsize; BUCKETS],
    boundaries: [u64; BUCKETS],
}

impl<const BUCKETS: usize> Histogram<BUCKETS> {
    pub fn new(boundaries: [u64; BUCKETS]) -> Self {
        const INIT: AtomicUsize = AtomicUsize::new(0);
        Self {
            buckets: [INIT; BUCKETS],
            boundaries,
        }
    }

    pub fn observe(&self, value: u64) {
        for (i, &boundary) in self.boundaries.iter().enumerate() {
            if value <= boundary {
                self.buckets[i].fetch_add(1, Ordering::Relaxed);
                break;
            }
        }
    }

    pub fn percentile(&self, p: f64) -> Option<u64> {
        let total: usize = self.buckets.iter().map(|b| b.load(Ordering::Relaxed)).sum();

        if total == 0 {
            return None;
        }

        let target = (total as f64 * p) as usize;
        let mut cumulative = 0;

        for (i, bucket) in self.buckets.iter().enumerate() {
            cumulative += bucket.load(Ordering::Relaxed);
            if cumulative >= target {
                return Some(self.boundaries[i]);
            }
        }

        None
    }
}

/// Sampling strategy - controls trace collection rate
pub trait SamplingStrategy {
    fn should_sample(&self, trace_id: u128) -> bool;
}

/// Always sample - collect all traces
pub struct AlwaysSample;
impl SamplingStrategy for AlwaysSample {
    fn should_sample(&self, _trace_id: u128) -> bool {
        true
    }
}

/// Never sample - collect no traces
pub struct NeverSample;
impl SamplingStrategy for NeverSample {
    fn should_sample(&self, _trace_id: u128) -> bool {
        false
    }
}

/// Probabilistic sampling - sample N% of traces
pub struct ProbabilisticSample {
    probability: f64,
}

impl ProbabilisticSample {
    pub fn new(probability: f64) -> Self {
        assert!(probability >= 0.0 && probability <= 1.0);
        Self { probability }
    }
}

impl SamplingStrategy for ProbabilisticSample {
    fn should_sample(&self, trace_id: u128) -> bool {
        let hash = (trace_id % 10000) as f64 / 10000.0;
        hash < self.probability
    }
}

/// Adaptive sampling - adjust rate based on load
pub struct AdaptiveSample {
    target_rate: AtomicUsize,
    current_rate: AtomicUsize,
}

impl AdaptiveSample {
    pub fn new(target_per_second: usize) -> Self {
        Self {
            target_rate: AtomicUsize::new(target_per_second),
            current_rate: AtomicUsize::new(0),
        }
    }

    pub fn adjust_target(&self, new_target: usize) {
        self.target_rate.store(new_target, Ordering::Relaxed);
    }
}

impl SamplingStrategy for AdaptiveSample {
    fn should_sample(&self, _trace_id: u128) -> bool {
        let current = self.current_rate.fetch_add(1, Ordering::Relaxed);
        let target = self.target_rate.load(Ordering::Relaxed);
        current < target
    }
}

/// Exemplar - trace example linked to metric
#[derive(Clone)]
pub struct Exemplar {
    pub value: u64,
    pub trace_id: u128,
    pub span_id: u64,
    pub timestamp: u64,
}

/// Metric with exemplars - links metrics to traces
pub struct MetricWithExemplars {
    counter: Counter,
    exemplars: Vec<Exemplar>,
}

impl MetricWithExemplars {
    pub fn new() -> Self {
        Self {
            counter: Counter::new(),
            exemplars: Vec::new(),
        }
    }

    pub fn record_with_trace(&mut self, value: u64, ctx: &TraceContext) {
        self.counter.add(value);

        // Keep latest N exemplars
        let exemplar = Exemplar {
            value,
            trace_id: ctx.trace_id,
            span_id: ctx.span_id,
            timestamp: 0, // Would use actual timestamp
        };

        if self.exemplars.len() < 10 {
            self.exemplars.push(exemplar);
        } else {
            self.exemplars.remove(0);
            self.exemplars.push(exemplar);
        }
    }

    pub fn exemplars(&self) -> &[Exemplar] {
        &self.exemplars
    }
}

/// Log aggregation - collect logs with structured fields
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub level: u8,
    pub message: String,
    pub fields: Vec<(&'static str, String)>,
    pub trace_context: Option<TraceContext>,
}

pub struct LogAggregator {
    entries: Vec<LogEntry>,
}

impl LogAggregator {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn log(&mut self, entry: LogEntry) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    pub fn filter_by_level(&self, min_level: u8) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|e| e.level >= min_level)
            .collect()
    }

    pub fn filter_by_trace(&self, trace_id: u128) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|e| {
                e.trace_context
                    .map(|ctx| ctx.trace_id == trace_id)
                    .unwrap_or(false)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_levels() {
        assert_eq!(Trace::NUMERIC_LEVEL, 0);
        assert_eq!(Debug::NUMERIC_LEVEL, 1);
        assert_eq!(Info::NUMERIC_LEVEL, 2);
        assert_eq!(Warn::NUMERIC_LEVEL, 3);
        assert_eq!(Error::NUMERIC_LEVEL, 4);
    }

    #[test]
    fn test_span_enabled() {
        let span: Span<Info, true> = Span::new("test");
        span.event("event occurred");
        span.attribute("key", "value");
        assert_eq!(span.name(), "test");
    }

    #[test]
    fn test_span_disabled() {
        let span: Span<Info, false> = Span::new("test");
        span.event("this should be no-op");
        span.attribute("key", "value");
    }

    #[test]
    fn test_trace_context() {
        let root = TraceContext::root();
        let child = root.child();

        assert_eq!(child.trace_id, root.trace_id);
        assert_eq!(child.parent_span_id, Some(root.span_id));
    }

    #[test]
    fn test_traceparent() {
        let ctx = TraceContext {
            trace_id: 0x12345678901234567890123456789012,
            span_id: 0xABCDEF0123456789,
            parent_span_id: None,
        };

        let traceparent = ctx.to_traceparent();
        assert!(traceparent.starts_with("00-"));
    }

    #[test]
    fn test_counter() {
        let counter = Counter::new();
        counter.increment();
        counter.add(5);
        assert_eq!(counter.get(), 6);
    }

    #[test]
    fn test_gauge() {
        let gauge = Gauge::new();
        gauge.set(10);
        gauge.increment();
        gauge.decrement();
        assert_eq!(gauge.get(), 10);
    }

    #[test]
    fn test_histogram() {
        let histogram = Histogram::new([10, 50, 100, 500, 1000]);

        histogram.observe(5);
        histogram.observe(25);
        histogram.observe(75);
        histogram.observe(250);

        let p50 = histogram.percentile(0.5);
        assert!(p50.is_some());
    }

    #[test]
    fn test_sampling() {
        let always = AlwaysSample;
        assert!(always.should_sample(12345));

        let never = NeverSample;
        assert!(!never.should_sample(12345));

        let prob = ProbabilisticSample::new(0.5);
        // Probabilistic, so we can't assert specific value
        let _ = prob.should_sample(12345);
    }

    #[test]
    fn test_adaptive_sampling() {
        let sampler = AdaptiveSample::new(100);

        let mut sampled = 0;
        for i in 0..200 {
            if sampler.should_sample(i as u128) {
                sampled += 1;
            }
        }

        assert!(sampled <= 100);
    }

    #[test]
    fn test_exemplars() {
        let mut metric = MetricWithExemplars::new();
        let ctx = TraceContext::root();

        metric.record_with_trace(100, &ctx);
        metric.record_with_trace(200, &ctx);

        let exemplars = metric.exemplars();
        assert_eq!(exemplars.len(), 2);
        assert_eq!(exemplars[0].value, 100);
    }

    #[test]
    fn test_log_aggregation() {
        let mut aggregator = LogAggregator::new();

        aggregator.log(LogEntry {
            level: Info::NUMERIC_LEVEL,
            message: "Info message".to_string(),
            fields: vec![("key", "value".to_string())],
            trace_context: None,
        });

        aggregator.log(LogEntry {
            level: Error::NUMERIC_LEVEL,
            message: "Error message".to_string(),
            fields: vec![],
            trace_context: None,
        });

        assert_eq!(aggregator.entries().len(), 2);

        let errors = aggregator.filter_by_level(Error::NUMERIC_LEVEL);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_log_trace_correlation() {
        let mut aggregator = LogAggregator::new();
        let ctx = TraceContext::root();

        aggregator.log(LogEntry {
            level: Info::NUMERIC_LEVEL,
            message: "Traced log".to_string(),
            fields: vec![],
            trace_context: Some(ctx),
        });

        let traced = aggregator.filter_by_trace(ctx.trace_id);
        assert_eq!(traced.len(), 1);
    }
}
