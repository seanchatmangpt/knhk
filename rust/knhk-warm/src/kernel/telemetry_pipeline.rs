// kernel/telemetry_pipeline.rs - Receipt streaming and metrics aggregation
// Phase 3: Telemetry pipeline with buffering and rate limiting
// DOCTRINE: Covenant 6 (Observations Are Sacred) - All telemetry must be preserved

use crossbeam::channel::{bounded, unbounded, Receiver, Sender, TryRecvError};
use dashmap::DashMap;
use flate2::write::GzEncoder;
use flate2::Compression;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, span, trace, warn, Level};

/// Telemetry receipt for processed items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryReceipt {
    pub id: String,
    pub timestamp: u64,
    pub execution_time_us: u64,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
    pub status: ReceiptStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReceiptStatus {
    Success,
    Error(String),
    Timeout,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Bool(bool),
    Array(Vec<AttributeValue>),
}

/// Aggregated metrics
#[derive(Debug, Clone)]
pub struct AggregatedMetrics {
    pub name: String,
    pub value: f64,
    pub count: u64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
    pub tags: HashMap<String, String>,
    pub window_start: Instant,
    pub window_end: Instant,
}

/// Event for correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedEvent {
    pub id: String,
    pub timestamp: u64,
    pub event_type: String,
    pub trace_id: String,
    pub correlation_id: String,
    pub data: HashMap<String, serde_json::Value>,
    pub related_events: Vec<String>,
}

/// Trace context for propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub trace_flags: u8,
    pub trace_state: String,
    pub baggage: HashMap<String, String>,
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceContext {
    pub fn new() -> Self {
        Self {
            trace_id: generate_trace_id(),
            span_id: generate_span_id(),
            trace_flags: 1, // Sampled
            trace_state: String::new(),
            baggage: HashMap::new(),
        }
    }

    pub fn child_span(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: generate_span_id(),
            trace_flags: self.trace_flags,
            trace_state: self.trace_state.clone(),
            baggage: self.baggage.clone(),
        }
    }
}

/// Rate limiter for telemetry
pub struct RateLimiter {
    tokens: AtomicU64,
    max_tokens: u64,
    refill_rate: u64,
    last_refill: Mutex<Instant>,
}

impl RateLimiter {
    pub fn new(max_tokens: u64, refill_rate: u64) -> Self {
        Self {
            tokens: AtomicU64::new(max_tokens),
            max_tokens,
            refill_rate,
            last_refill: Mutex::new(Instant::now()),
        }
    }

    pub fn try_acquire(&self, tokens: u64) -> bool {
        self.refill();

        let mut current = self.tokens.load(Ordering::Acquire);
        loop {
            if current < tokens {
                return false;
            }

            match self.tokens.compare_exchange_weak(
                current,
                current - tokens,
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => return true,
                Err(x) => current = x,
            }
        }
    }

    fn refill(&self) {
        let mut last_refill = self.last_refill.lock();
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill);

        if elapsed >= Duration::from_secs(1) {
            let tokens_to_add = (elapsed.as_secs() * self.refill_rate).min(self.max_tokens);
            self.tokens.fetch_add(tokens_to_add, Ordering::Release);
            *last_refill = now;
        }
    }
}

/// Buffered telemetry stream
pub struct TelemetryBuffer {
    receipts: Arc<Mutex<VecDeque<TelemetryReceipt>>>,
    metrics: Arc<DashMap<String, AggregatedMetrics>>,
    events: Arc<Mutex<VecDeque<CorrelatedEvent>>>,
    max_buffer_size: usize,
    compression_threshold: usize,
    dropped_items: AtomicU64,
}

impl TelemetryBuffer {
    pub fn new(max_size: usize, compression_threshold: usize) -> Self {
        Self {
            receipts: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            metrics: Arc::new(DashMap::new()),
            events: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            max_buffer_size: max_size,
            compression_threshold,
            dropped_items: AtomicU64::new(0),
        }
    }

    pub fn add_receipt(&self, receipt: TelemetryReceipt) -> bool {
        let mut receipts = self.receipts.lock();

        if receipts.len() >= self.max_buffer_size {
            self.dropped_items.fetch_add(1, Ordering::Relaxed);
            receipts.pop_front(); // Drop oldest
        }

        receipts.push_back(receipt);
        true
    }

    pub fn add_metric(&self, name: String, value: f64, tags: HashMap<String, String>) {
        self.metrics
            .entry(name.clone())
            .and_modify(|m| {
                m.count += 1;
                m.sum += value;
                m.min = m.min.min(value);
                m.max = m.max.max(value);
                m.value = m.sum / m.count as f64;
            })
            .or_insert_with(|| AggregatedMetrics {
                name,
                value,
                count: 1,
                min: value,
                max: value,
                sum: value,
                tags,
                window_start: Instant::now(),
                window_end: Instant::now() + Duration::from_secs(60),
            });
    }

    pub fn flush(
        &self,
    ) -> (
        Vec<TelemetryReceipt>,
        Vec<AggregatedMetrics>,
        Vec<CorrelatedEvent>,
    ) {
        let mut receipts = self.receipts.lock();
        let mut events = self.events.lock();

        let flushed_receipts: Vec<_> = receipts.drain(..).collect();
        let flushed_events: Vec<_> = events.drain(..).collect();

        let flushed_metrics: Vec<_> = self
            .metrics
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        self.metrics.clear();

        (flushed_receipts, flushed_metrics, flushed_events)
    }

    pub fn compress_if_needed(&self) -> Option<Vec<u8>> {
        let receipts = self.receipts.lock();

        if receipts.len() < self.compression_threshold {
            return None;
        }

        // Compress receipts
        let data = serde_json::to_vec(&receipts.as_slices()).ok()?;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&data).ok()?;
        encoder.finish().ok()
    }
}

/// Event correlator for linking related events
pub struct EventCorrelator {
    correlation_map: Arc<DashMap<String, Vec<String>>>,
    trace_map: Arc<DashMap<String, Vec<CorrelatedEvent>>>,
    window_size: Duration,
}

impl EventCorrelator {
    pub fn new(window_size: Duration) -> Self {
        Self {
            correlation_map: Arc::new(DashMap::new()),
            trace_map: Arc::new(DashMap::new()),
            window_size,
        }
    }

    pub fn correlate(&self, event: CorrelatedEvent) {
        // Add to trace map
        self.trace_map
            .entry(event.trace_id.clone())
            .or_default()
            .push(event.clone());

        // Add to correlation map
        self.correlation_map
            .entry(event.correlation_id.clone())
            .or_default()
            .push(event.id.clone());
    }

    pub fn get_correlated_events(&self, correlation_id: &str) -> Vec<String> {
        self.correlation_map
            .get(correlation_id)
            .map(|entry| entry.value().clone())
            .unwrap_or_default()
    }

    pub fn get_trace_events(&self, trace_id: &str) -> Vec<CorrelatedEvent> {
        self.trace_map
            .get(trace_id)
            .map(|entry| entry.value().clone())
            .unwrap_or_default()
    }

    pub fn cleanup_old_events(&self, older_than: Instant) {
        let cutoff = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs()
            .saturating_sub(older_than.elapsed().as_secs());

        self.trace_map.retain(|_, events| {
            events.retain(|e| e.timestamp > cutoff);
            !events.is_empty()
        });
    }
}

/// Main telemetry pipeline
pub struct TelemetryPipeline {
    buffer: Arc<TelemetryBuffer>,
    rate_limiter: Arc<RateLimiter>,
    correlator: Arc<EventCorrelator>,
    stream_tx: Sender<TelemetryBatch>,
    stream_rx: Receiver<TelemetryBatch>,
    stats: Arc<PipelineStats>,
    running: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
pub struct TelemetryBatch {
    pub receipts: Vec<TelemetryReceipt>,
    pub metrics: Vec<AggregatedMetrics>,
    pub events: Vec<CorrelatedEvent>,
    pub batch_id: String,
    pub timestamp: u64,
}

#[derive(Debug)]
pub struct PipelineStats {
    pub receipts_processed: AtomicU64,
    pub metrics_aggregated: AtomicU64,
    pub events_correlated: AtomicU64,
    pub batches_sent: AtomicU64,
    pub bytes_sent: AtomicU64,
    pub rate_limited_items: AtomicU64,
}

impl TelemetryPipeline {
    pub fn new(buffer_size: usize, rate_limit: u64, correlation_window: Duration) -> Self {
        let (tx, rx) = bounded(100);

        Self {
            buffer: Arc::new(TelemetryBuffer::new(buffer_size, buffer_size / 2)),
            rate_limiter: Arc::new(RateLimiter::new(rate_limit, rate_limit)),
            correlator: Arc::new(EventCorrelator::new(correlation_window)),
            stream_tx: tx,
            stream_rx: rx,
            stats: Arc::new(PipelineStats {
                receipts_processed: AtomicU64::new(0),
                metrics_aggregated: AtomicU64::new(0),
                events_correlated: AtomicU64::new(0),
                batches_sent: AtomicU64::new(0),
                bytes_sent: AtomicU64::new(0),
                rate_limited_items: AtomicU64::new(0),
            }),
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Process a receipt through the pipeline
    pub fn process_receipt(&self, receipt: TelemetryReceipt) -> Result<(), String> {
        // Check rate limit
        if !self.rate_limiter.try_acquire(1) {
            self.stats
                .rate_limited_items
                .fetch_add(1, Ordering::Relaxed);
            return Err("Rate limited".to_string());
        }

        // Add to buffer
        self.buffer.add_receipt(receipt);
        self.stats
            .receipts_processed
            .fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Process a metric through the pipeline
    pub fn process_metric(&self, name: String, value: f64, tags: HashMap<String, String>) {
        if !self.rate_limiter.try_acquire(1) {
            self.stats
                .rate_limited_items
                .fetch_add(1, Ordering::Relaxed);
            return;
        }

        self.buffer.add_metric(name, value, tags);
        self.stats
            .metrics_aggregated
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Process an event with correlation
    pub fn process_event(&self, event: CorrelatedEvent) {
        if !self.rate_limiter.try_acquire(1) {
            self.stats
                .rate_limited_items
                .fetch_add(1, Ordering::Relaxed);
            return;
        }

        self.correlator.correlate(event.clone());
        self.buffer.events.lock().push_back(event);
        self.stats.events_correlated.fetch_add(1, Ordering::Relaxed);
    }

    /// Create and send a batch
    pub fn flush_batch(&self) -> Result<(), String> {
        let (receipts, metrics, events) = self.buffer.flush();

        if receipts.is_empty() && metrics.is_empty() && events.is_empty() {
            return Ok(());
        }

        let batch = TelemetryBatch {
            receipts,
            metrics,
            events,
            batch_id: generate_batch_id(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::from_secs(0))
                .as_secs(),
        };

        // Estimate batch size
        let batch_size = estimate_batch_size(&batch);
        self.stats
            .bytes_sent
            .fetch_add(batch_size as u64, Ordering::Relaxed);

        self.stream_tx
            .try_send(batch)
            .map_err(|_| "Channel full".to_string())?;

        self.stats.batches_sent.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Start automatic batching
    pub fn start_auto_batching(&self, interval: Duration) -> Arc<AtomicBool> {
        let running = Arc::clone(&self.running);
        let pipeline = Arc::new(self.clone_refs());

        std::thread::spawn(move || {
            while running.load(Ordering::Acquire) {
                std::thread::sleep(interval);

                if let Err(e) = pipeline.flush_batch() {
                    warn!("Failed to flush batch: {}", e);
                }

                // Cleanup old correlation data
                pipeline
                    .correlator
                    .cleanup_old_events(Instant::now() - Duration::from_secs(300));
            }
        });

        Arc::clone(&self.running)
    }

    /// Receive batches from the pipeline
    pub fn receive_batch(&self) -> Result<TelemetryBatch, TryRecvError> {
        self.stream_rx.try_recv()
    }

    /// Get pipeline statistics
    pub fn get_stats(&self) -> PipelineStatistics {
        PipelineStatistics {
            receipts_processed: self.stats.receipts_processed.load(Ordering::Relaxed),
            metrics_aggregated: self.stats.metrics_aggregated.load(Ordering::Relaxed),
            events_correlated: self.stats.events_correlated.load(Ordering::Relaxed),
            batches_sent: self.stats.batches_sent.load(Ordering::Relaxed),
            bytes_sent: self.stats.bytes_sent.load(Ordering::Relaxed),
            rate_limited_items: self.stats.rate_limited_items.load(Ordering::Relaxed),
            dropped_items: self.buffer.dropped_items.load(Ordering::Relaxed),
        }
    }

    /// Shutdown the pipeline
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::Release);

        // Final flush
        let _ = self.flush_batch();
    }

    fn clone_refs(&self) -> Self {
        Self {
            buffer: Arc::clone(&self.buffer),
            rate_limiter: Arc::clone(&self.rate_limiter),
            correlator: Arc::clone(&self.correlator),
            stream_tx: self.stream_tx.clone(),
            stream_rx: self.stream_rx.clone(),
            stats: Arc::clone(&self.stats),
            running: Arc::clone(&self.running),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStatistics {
    pub receipts_processed: u64,
    pub metrics_aggregated: u64,
    pub events_correlated: u64,
    pub batches_sent: u64,
    pub bytes_sent: u64,
    pub rate_limited_items: u64,
    pub dropped_items: u64,
}

/// Context propagation for distributed tracing
pub struct ContextPropagator {
    extractors: HashMap<String, Box<dyn ContextExtractor>>,
    injectors: HashMap<String, Box<dyn ContextInjector>>,
}

trait ContextExtractor: Send + Sync {
    fn extract(&self, carrier: &dyn std::any::Any) -> Option<TraceContext>;
}

trait ContextInjector: Send + Sync {
    fn inject(&self, context: &TraceContext, carrier: &mut dyn std::any::Any);
}

impl Default for ContextPropagator {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextPropagator {
    pub fn new() -> Self {
        Self {
            extractors: HashMap::new(),
            injectors: HashMap::new(),
        }
    }

    pub fn extract<T: 'static>(&self, format: &str, carrier: &T) -> Option<TraceContext> {
        self.extractors
            .get(format)?
            .extract(carrier as &dyn std::any::Any)
    }

    pub fn inject<T: 'static>(&self, format: &str, context: &TraceContext, carrier: &mut T) {
        if let Some(injector) = self.injectors.get(format) {
            injector.inject(context, carrier as &mut dyn std::any::Any);
        }
    }
}

// Helper functions
fn generate_trace_id() -> String {
    format!("{:032x}", rand::random::<u128>())
}

fn generate_span_id() -> String {
    format!("{:016x}", rand::random::<u64>())
}

fn generate_batch_id() -> String {
    format!("batch-{}", uuid::Uuid::new_v4())
}

fn estimate_batch_size(batch: &TelemetryBatch) -> usize {
    // Rough estimation
    let receipts_size = batch.receipts.len() * std::mem::size_of::<TelemetryReceipt>();
    let metrics_size = batch.metrics.len() * std::mem::size_of::<AggregatedMetrics>();
    let events_size = batch.events.len() * std::mem::size_of::<CorrelatedEvent>();

    receipts_size + metrics_size + events_size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(10, 5);

        // Should allow initial tokens
        assert!(limiter.try_acquire(5));
        assert!(limiter.try_acquire(5));

        // Should block when exhausted
        assert!(!limiter.try_acquire(1));

        // Wait for refill
        std::thread::sleep(Duration::from_secs(2));
        assert!(limiter.try_acquire(5));
    }

    #[test]
    fn test_telemetry_buffer() {
        let buffer = TelemetryBuffer::new(10, 5);

        for i in 0..15 {
            let receipt = TelemetryReceipt {
                id: format!("receipt-{}", i),
                timestamp: i as u64,
                execution_time_us: 100,
                trace_id: "test-trace".to_string(),
                span_id: format!("span-{}", i),
                parent_span_id: None,
                attributes: HashMap::new(),
                status: ReceiptStatus::Success,
            };
            buffer.add_receipt(receipt);
        }

        let (receipts, _, _) = buffer.flush();
        assert_eq!(receipts.len(), 10); // Max size enforced
        assert_eq!(buffer.dropped_items.load(Ordering::Relaxed), 5);
    }

    #[test]
    fn test_event_correlator() {
        let correlator = EventCorrelator::new(Duration::from_secs(60));

        let event1 = CorrelatedEvent {
            id: "event-1".to_string(),
            timestamp: 100,
            event_type: "test".to_string(),
            trace_id: "trace-1".to_string(),
            correlation_id: "corr-1".to_string(),
            data: HashMap::new(),
            related_events: vec![],
        };

        let event2 = CorrelatedEvent {
            id: "event-2".to_string(),
            timestamp: 101,
            event_type: "test".to_string(),
            trace_id: "trace-1".to_string(),
            correlation_id: "corr-1".to_string(),
            data: HashMap::new(),
            related_events: vec![],
        };

        correlator.correlate(event1);
        correlator.correlate(event2);

        let correlated = correlator.get_correlated_events("corr-1");
        assert_eq!(correlated.len(), 2);

        let trace_events = correlator.get_trace_events("trace-1");
        assert_eq!(trace_events.len(), 2);
    }

    #[test]
    fn test_telemetry_pipeline() {
        let pipeline = TelemetryPipeline::new(100, 1000, Duration::from_secs(60));

        let receipt = TelemetryReceipt {
            id: "test-receipt".to_string(),
            timestamp: 100,
            execution_time_us: 50,
            trace_id: "test-trace".to_string(),
            span_id: "test-span".to_string(),
            parent_span_id: None,
            attributes: HashMap::new(),
            status: ReceiptStatus::Success,
        };

        assert!(pipeline.process_receipt(receipt).is_ok());

        pipeline.process_metric(
            "test.metric".to_string(),
            42.0,
            HashMap::from([("tag".to_string(), "value".to_string())]),
        );

        let stats = pipeline.get_stats();
        assert_eq!(stats.receipts_processed, 1);
        assert_eq!(stats.metrics_aggregated, 1);
    }

    #[test]
    fn test_trace_context() {
        let context = TraceContext::new();
        assert!(!context.trace_id.is_empty());
        assert!(!context.span_id.is_empty());

        let child = context.child_span();
        assert_eq!(child.trace_id, context.trace_id);
        assert_ne!(child.span_id, context.span_id);
    }

    #[test]
    fn test_metrics_aggregation() {
        let buffer = TelemetryBuffer::new(100, 50);

        for i in 0..10 {
            buffer.add_metric("test.metric".to_string(), i as f64, HashMap::new());
        }

        let (_, metrics, _) = buffer.flush();
        assert_eq!(metrics.len(), 1);

        let metric = &metrics[0];
        assert_eq!(metric.count, 10);
        assert_eq!(metric.min, 0.0);
        assert_eq!(metric.max, 9.0);
        assert_eq!(metric.sum, 45.0);
    }
}
