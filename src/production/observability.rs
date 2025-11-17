// KNHK Observability Layer - Complete Instrumentation
// Phase 5: Production-grade telemetry with OpenTelemetry, Prometheus, and Jaeger
// Provides full visibility into system behavior for Fortune 500 operations

use std::collections::HashMap;
use std::sync::{Arc, RwLock, atomic::{AtomicU64, AtomicUsize, Ordering}};
use std::time::{Duration, Instant, SystemTime};
use opentelemetry::{global, KeyValue, Context};
use opentelemetry::trace::{Tracer, TracerProvider, Span, SpanKind, Status};
use opentelemetry::metrics::{Counter, Histogram, Meter, MeterProvider, UpDownCounter};
use opentelemetry_sdk::trace as sdktrace;
use opentelemetry_sdk::metrics as sdkmetrics;
use opentelemetry_sdk::Resource;
use opentelemetry_otlp::{WithExportConfig, ExportConfig};
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION, SERVICE_INSTANCE_ID};
use prometheus::{Encoder, TextEncoder, Registry};
use tracing::{info, warn, error, debug, instrument};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use dashmap::DashMap;

const SERVICE_NAME_VALUE: &str = "knhk-production";
const SERVICE_VERSION_VALUE: &str = "5.0.0";
const METRICS_FLUSH_INTERVAL: Duration = Duration::from_secs(10);
const TRACE_SAMPLING_RATE: f64 = 1.0; // 100% for production monitoring
const SPAN_BUFFER_SIZE: usize = 10000;
const METRIC_BUFFER_SIZE: usize = 10000;

/// Telemetry data for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Telemetry {
    pub workflow_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation: String,
    pub start_time: SystemTime,
    pub duration: Duration,
    pub status: TelemetryStatus,
    pub attributes: HashMap<String, String>,
    pub events: Vec<TelemetryEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TelemetryStatus {
    Ok,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub name: String,
    pub timestamp: SystemTime,
    pub attributes: HashMap<String, String>,
}

/// Complete observability implementation
pub struct ObservabilityLayer {
    // OpenTelemetry providers
    tracer_provider: sdktrace::TracerProvider,
    meter_provider: sdkmetrics::SdkMeterProvider,

    // Prometheus registry
    prometheus_registry: Registry,

    // Metrics
    workflow_counter: Counter<u64>,
    workflow_duration: Histogram<f64>,
    workflow_errors: Counter<u64>,
    active_workflows: UpDownCounter<i64>,

    step_counter: Counter<u64>,
    step_duration: Histogram<f64>,
    step_errors: Counter<u64>,

    receipt_counter: Counter<u64>,
    receipt_size: Histogram<u64>,

    // Custom metrics
    latency_percentiles: Arc<RwLock<LatencyPercentiles>>,
    error_rates: Arc<RwLock<ErrorRates>>,
    throughput: Arc<RwLock<ThroughputMetrics>>,

    // Active spans tracking
    active_spans: Arc<DashMap<String, SpanHandle>>,

    // Telemetry buffer for export
    telemetry_buffer: Arc<RwLock<Vec<Telemetry>>>,
    telemetry_tx: mpsc::UnboundedSender<Telemetry>,
    telemetry_rx: Option<mpsc::UnboundedReceiver<Telemetry>>,

    // Configuration
    endpoint: Option<String>,
    service_instance_id: String,

    // Metrics
    total_spans: Arc<AtomicU64>,
    total_events: Arc<AtomicU64>,
    total_metrics: Arc<AtomicU64>,
}

struct SpanHandle {
    span: Box<dyn Span>,
    start_time: Instant,
    workflow_id: String,
    operation: String,
}

#[derive(Debug, Clone, Default)]
struct LatencyPercentiles {
    p50: f64,
    p90: f64,
    p95: f64,
    p99: f64,
    p999: f64,
    samples: Vec<f64>,
}

#[derive(Debug, Clone, Default)]
struct ErrorRates {
    total: u64,
    errors: u64,
    by_type: HashMap<String, u64>,
    window_start: Option<Instant>,
}

#[derive(Debug, Clone, Default)]
struct ThroughputMetrics {
    requests_per_second: f64,
    bytes_per_second: f64,
    window_start: Option<Instant>,
    request_count: u64,
    byte_count: u64,
}

impl ObservabilityLayer {
    /// Initialize observability with OpenTelemetry
    pub fn new(endpoint: Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing observability layer");

        let service_instance_id = format!("knhk-{}", uuid::Uuid::new_v4());

        // Create resource with service information
        let resource = Resource::new(vec![
            KeyValue::new(SERVICE_NAME, SERVICE_NAME_VALUE),
            KeyValue::new(SERVICE_VERSION, SERVICE_VERSION_VALUE),
            KeyValue::new(SERVICE_INSTANCE_ID, service_instance_id.clone()),
        ]);

        // Configure trace exporter (OTLP)
        let trace_config = sdktrace::config()
            .with_sampler(sdktrace::Sampler::TraceIdRatioBased(TRACE_SAMPLING_RATE))
            .with_resource(resource.clone());

        let tracer_provider = if let Some(ref endpoint) = endpoint {
            let exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint.clone())
                .with_timeout(Duration::from_secs(10));

            opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(exporter)
                .with_trace_config(trace_config)
                .install_batch(opentelemetry_sdk::runtime::Tokio)?
        } else {
            // Fallback to stdout exporter for development
            sdktrace::TracerProvider::builder()
                .with_config(trace_config)
                .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
                .build()
        };

        // Configure metrics exporter
        let meter_provider = if let Some(ref endpoint) = endpoint {
            let exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint.clone());

            opentelemetry_otlp::new_pipeline()
                .metrics(opentelemetry_sdk::runtime::Tokio)
                .with_exporter(exporter)
                .with_resource(resource)
                .with_period(METRICS_FLUSH_INTERVAL)
                .build()?
        } else {
            opentelemetry_sdk::metrics::MeterProvider::builder()
                .with_resource(resource)
                .with_reader(
                    opentelemetry_sdk::metrics::PeriodicReader::builder(
                        opentelemetry_stdout::MetricsExporter::default(),
                        opentelemetry_sdk::runtime::Tokio,
                    ).build()
                )
                .build()
        };

        // Set global providers
        global::set_tracer_provider(tracer_provider.clone());
        global::set_meter_provider(meter_provider.clone());

        // Get meter for creating instruments
        let meter = global::meter("knhk");

        // Create metrics instruments
        let workflow_counter = meter.u64_counter("knhk.workflow.count")
            .with_description("Total number of workflows processed")
            .init();

        let workflow_duration = meter.f64_histogram("knhk.workflow.duration")
            .with_description("Workflow execution duration in seconds")
            .with_unit("s")
            .init();

        let workflow_errors = meter.u64_counter("knhk.workflow.errors")
            .with_description("Total number of workflow errors")
            .init();

        let active_workflows = meter.i64_up_down_counter("knhk.workflow.active")
            .with_description("Number of currently active workflows")
            .init();

        let step_counter = meter.u64_counter("knhk.step.count")
            .with_description("Total number of workflow steps executed")
            .init();

        let step_duration = meter.f64_histogram("knhk.step.duration")
            .with_description("Step execution duration in seconds")
            .with_unit("s")
            .init();

        let step_errors = meter.u64_counter("knhk.step.errors")
            .with_description("Total number of step errors")
            .init();

        let receipt_counter = meter.u64_counter("knhk.receipt.count")
            .with_description("Total number of receipts generated")
            .init();

        let receipt_size = meter.u64_histogram("knhk.receipt.size")
            .with_description("Size of receipts in bytes")
            .with_unit("By")
            .init();

        // Create Prometheus registry
        let prometheus_registry = Registry::new();

        // Create telemetry channel
        let (telemetry_tx, telemetry_rx) = mpsc::unbounded_channel();

        Ok(Self {
            tracer_provider,
            meter_provider,
            prometheus_registry,
            workflow_counter,
            workflow_duration,
            workflow_errors,
            active_workflows,
            step_counter,
            step_duration,
            step_errors,
            receipt_counter,
            receipt_size,
            latency_percentiles: Arc::new(RwLock::new(LatencyPercentiles::default())),
            error_rates: Arc::new(RwLock::new(ErrorRates::default())),
            throughput: Arc::new(RwLock::new(ThroughputMetrics::default())),
            active_spans: Arc::new(DashMap::new()),
            telemetry_buffer: Arc::new(RwLock::new(Vec::new())),
            telemetry_tx,
            telemetry_rx: Some(telemetry_rx),
            endpoint,
            service_instance_id,
            total_spans: Arc::new(AtomicU64::new(0)),
            total_events: Arc::new(AtomicU64::new(0)),
            total_metrics: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Start observability services
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting observability services");

        // Initialize tracing subscriber
        let telemetry_layer = tracing_opentelemetry::layer()
            .with_tracer(global::tracer("knhk"));

        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(telemetry_layer)
            .try_init()
            .ok(); // Ignore if already initialized

        // Start telemetry processor
        self.start_telemetry_processor();

        // Start metrics aggregator
        self.start_metrics_aggregator();

        info!("Observability services started");
        Ok(())
    }

    /// Record workflow submission
    #[instrument(skip(self))]
    pub async fn record_workflow_submitted(&self, workflow_id: &str, descriptor: &str) {
        let tracer = global::tracer("knhk");
        let mut span = tracer
            .span_builder("workflow.submit")
            .with_kind(SpanKind::Server)
            .with_attributes(vec![
                KeyValue::new("workflow.id", workflow_id.to_string()),
                KeyValue::new("workflow.descriptor", descriptor.to_string()),
            ])
            .start(&tracer);

        span.set_status(Status::Ok);
        span.end();

        self.workflow_counter.add(&Context::current(), 1, &[
            KeyValue::new("status", "submitted"),
        ]);

        self.total_spans.fetch_add(1, Ordering::Relaxed);
    }

    /// Record workflow start
    #[instrument(skip(self))]
    pub async fn record_workflow_start(&self, workflow_id: &str) {
        let tracer = global::tracer("knhk");
        let span = tracer
            .span_builder("workflow.execute")
            .with_kind(SpanKind::Internal)
            .with_attributes(vec![
                KeyValue::new("workflow.id", workflow_id.to_string()),
            ])
            .start(&tracer);

        self.active_workflows.add(&Context::current(), 1, &[]);

        // Store active span
        self.active_spans.insert(
            workflow_id.to_string(),
            SpanHandle {
                span: Box::new(span),
                start_time: Instant::now(),
                workflow_id: workflow_id.to_string(),
                operation: "workflow.execute".to_string(),
            }
        );
    }

    /// Record workflow step completion
    #[instrument(skip(self))]
    pub async fn record_step_completion(
        &self,
        workflow_id: &str,
        step_index: usize,
        elapsed: Duration,
    ) {
        let tracer = global::tracer("knhk");
        let mut span = tracer
            .span_builder("step.execute")
            .with_kind(SpanKind::Internal)
            .with_attributes(vec![
                KeyValue::new("workflow.id", workflow_id.to_string()),
                KeyValue::new("step.index", step_index as i64),
            ])
            .start(&tracer);

        span.set_status(Status::Ok);
        span.end();

        self.step_counter.add(&Context::current(), 1, &[
            KeyValue::new("status", "completed"),
        ]);

        self.step_duration.record(&Context::current(), elapsed.as_secs_f64(), &[]);

        self.total_spans.fetch_add(1, Ordering::Relaxed);
    }

    /// Record workflow completion
    #[instrument(skip(self))]
    pub async fn record_workflow_completion(&self, workflow_id: &str, elapsed: Duration) {
        if let Some((_, handle)) = self.active_spans.remove(workflow_id) {
            let mut span = handle.span;
            span.set_status(Status::Ok);
            span.end();
        }

        self.active_workflows.add(&Context::current(), -1, &[]);

        self.workflow_duration.record(&Context::current(), elapsed.as_secs_f64(), &[
            KeyValue::new("status", "completed"),
        ]);

        // Update latency percentiles
        self.update_latency_percentiles(elapsed.as_millis() as f64);

        // Update throughput
        self.update_throughput(1, 0);
    }

    /// Record workflow error
    #[instrument(skip(self))]
    pub async fn record_workflow_error(&self, workflow_id: &str, error: &str) {
        if let Some((_, handle)) = self.active_spans.remove(workflow_id) {
            let mut span = handle.span;
            span.set_status(Status::error(error));
            span.end();
        }

        self.workflow_errors.add(&Context::current(), 1, &[
            KeyValue::new("error", error.to_string()),
        ]);

        self.active_workflows.add(&Context::current(), -1, &[]);

        // Update error rates
        self.update_error_rates(error);
    }

    /// Record receipt generation
    #[instrument(skip(self))]
    pub async fn record_receipt(&self, workflow_id: &str, size_bytes: u64) {
        self.receipt_counter.add(&Context::current(), 1, &[
            KeyValue::new("workflow.id", workflow_id.to_string()),
        ]);

        self.receipt_size.record(&Context::current(), size_bytes, &[]);

        self.total_metrics.fetch_add(2, Ordering::Relaxed);
    }

    /// Update latency percentiles
    fn update_latency_percentiles(&self, latency_ms: f64) {
        let mut percentiles = self.latency_percentiles.write().unwrap();
        percentiles.samples.push(latency_ms);

        // Keep only last 10000 samples
        if percentiles.samples.len() > 10000 {
            percentiles.samples.remove(0);
        }

        // Calculate percentiles
        let mut sorted = percentiles.samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = sorted.len();
        if len > 0 {
            percentiles.p50 = sorted[len * 50 / 100];
            percentiles.p90 = sorted[len * 90 / 100];
            percentiles.p95 = sorted[len * 95 / 100];
            percentiles.p99 = sorted[len * 99 / 100];
            percentiles.p999 = sorted[len * 999 / 1000];
        }
    }

    /// Update error rates
    fn update_error_rates(&self, error: &str) {
        let mut rates = self.error_rates.write().unwrap();
        rates.errors += 1;
        rates.total += 1;

        *rates.by_type.entry(error.to_string()).or_insert(0) += 1;

        // Reset window every minute
        if rates.window_start.is_none() ||
           rates.window_start.unwrap().elapsed() > Duration::from_secs(60) {
            rates.window_start = Some(Instant::now());
            rates.total = 0;
            rates.errors = 0;
            rates.by_type.clear();
        }
    }

    /// Update throughput metrics
    fn update_throughput(&self, requests: u64, bytes: u64) {
        let mut throughput = self.throughput.write().unwrap();
        throughput.request_count += requests;
        throughput.byte_count += bytes;

        // Calculate rates every second
        if throughput.window_start.is_none() ||
           throughput.window_start.unwrap().elapsed() >= Duration::from_secs(1) {
            let elapsed = throughput.window_start
                .map(|s| s.elapsed().as_secs_f64())
                .unwrap_or(1.0);

            throughput.requests_per_second = throughput.request_count as f64 / elapsed;
            throughput.bytes_per_second = throughput.byte_count as f64 / elapsed;

            throughput.window_start = Some(Instant::now());
            throughput.request_count = 0;
            throughput.byte_count = 0;
        }
    }

    /// Start telemetry processor
    fn start_telemetry_processor(&self) {
        let buffer = self.telemetry_buffer.clone();
        let mut rx = self.telemetry_rx.take().unwrap();

        tokio::spawn(async move {
            while let Some(telemetry) = rx.recv().await {
                let mut buf = buffer.write().unwrap();
                buf.push(telemetry);

                // Keep buffer size limited
                if buf.len() > SPAN_BUFFER_SIZE {
                    buf.remove(0);
                }
            }
        });
    }

    /// Start metrics aggregator
    fn start_metrics_aggregator(&self) {
        let latency = self.latency_percentiles.clone();
        let errors = self.error_rates.clone();
        let throughput_metrics = self.throughput.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));

            loop {
                interval.tick().await;

                // Log current metrics
                let latency = latency.read().unwrap();
                let errors = errors.read().unwrap();
                let throughput = throughput_metrics.read().unwrap();

                info!(
                    "Metrics - Latency p50: {:.2}ms, p99: {:.2}ms | \
                     Errors: {}/{} ({:.2}%) | \
                     Throughput: {:.2} req/s",
                    latency.p50, latency.p99,
                    errors.errors, errors.total,
                    if errors.total > 0 {
                        (errors.errors as f64 / errors.total as f64) * 100.0
                    } else {
                        0.0
                    },
                    throughput.requests_per_second
                );
            }
        });
    }

    /// Get Prometheus metrics
    pub fn get_prometheus_metrics(&self) -> Result<String, Box<dyn std::error::Error>> {
        let encoder = TextEncoder::new();
        let metric_families = self.prometheus_registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// Get telemetry statistics
    pub fn get_stats(&self) -> ObservabilityStats {
        ObservabilityStats {
            total_spans: self.total_spans.load(Ordering::Relaxed),
            total_events: self.total_events.load(Ordering::Relaxed),
            total_metrics: self.total_metrics.load(Ordering::Relaxed),
            active_spans: self.active_spans.len(),
            latency_p50: self.latency_percentiles.read().unwrap().p50,
            latency_p99: self.latency_percentiles.read().unwrap().p99,
            error_rate: {
                let rates = self.error_rates.read().unwrap();
                if rates.total > 0 {
                    (rates.errors as f64 / rates.total as f64) * 100.0
                } else {
                    0.0
                }
            },
            throughput_rps: self.throughput.read().unwrap().requests_per_second,
        }
    }

    /// Shutdown observability
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Shutting down observability layer");

        // Flush all pending spans
        for entry in self.active_spans.iter() {
            let mut span = entry.value().span.clone();
            span.end();
        }

        // Shutdown providers
        self.tracer_provider.shutdown()?;
        self.meter_provider.shutdown()?;

        info!("Observability layer shutdown complete");
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityStats {
    pub total_spans: u64,
    pub total_events: u64,
    pub total_metrics: u64,
    pub active_spans: usize,
    pub latency_p50: f64,
    pub latency_p99: f64,
    pub error_rate: f64,
    pub throughput_rps: f64,
}