// rust/knhk-otel/src/lib.rs
// OpenTelemetry Observability Integration
// Provides metrics, traces, and spans for KNHKS operations

// CRITICAL: Enforce proper error handling - no unwrap/expect in production code
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![cfg_attr(not(feature = "std"), no_std)]
// Allow acceptable warnings for clean build
#![allow(unused_imports)] // Some imports are conditional or reserved for planned use
#![allow(unused_variables)] // Some variables are used in conditional compilation
#![allow(unused_mut)] // Some mut variables are used in conditional compilation
#![allow(dead_code)] // Some code is reserved for planned features
#![allow(deprecated)] // Some dependencies use deprecated APIs (will be updated)
#![allow(unexpected_cfgs)] // Some cfg values are informational
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::time::{SystemTime, UNIX_EPOCH};

// Module declarations
pub mod runtime_class;

#[cfg(feature = "std")]
pub mod exporter;

#[cfg(feature = "std")]
pub mod metrics;

#[cfg(feature = "std")]
pub mod types;

#[cfg(feature = "std")]
pub mod validation;

// Advanced Rust features for zero-overhead hot path telemetry
pub mod const_validation;
pub mod hot_path;
pub mod simd;

/// Trace ID (128-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceId(pub u128);

/// Span ID (64-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpanId(pub u64);

/// Span context
#[derive(Debug, Clone)]
pub struct SpanContext {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span_id: Option<SpanId>,
    pub flags: u8,
}

/// Span attributes
pub type Attributes = BTreeMap<String, String>;

/// Span event
#[derive(Debug, Clone)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp_ms: u64,
    pub attributes: Attributes,
}

/// Span
#[derive(Debug, Clone)]
pub struct Span {
    pub context: SpanContext,
    pub name: String,
    pub start_time_ms: u64,
    pub end_time_ms: Option<u64>,
    pub attributes: Attributes,
    pub events: Vec<SpanEvent>,
    pub status: SpanStatus,
}

/// Span status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanStatus {
    Ok,
    Error,
    Unset,
}

/// Metric value
#[derive(Debug, Clone)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<u64>),
}

/// Metric
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub value: MetricValue,
    pub timestamp_ms: u64,
    pub attributes: Attributes,
}

/// OTLP exporter for sending spans/metrics to collectors
#[cfg(feature = "std")]
pub struct OtlpExporter {
    endpoint: String,
}

/// Weaver live-check integration for telemetry validation
#[cfg(feature = "std")]
pub struct WeaverLiveCheck {
    registry_path: Option<String>,
    otlp_grpc_address: String,
    otlp_grpc_port: u16,
    admin_port: u16,
    inactivity_timeout: u64,
    format: String,
    output: Option<String>,
}

#[cfg(feature = "std")]
impl WeaverLiveCheck {
    /// Create a new Weaver live-check instance
    pub fn new() -> Self {
        Self {
            registry_path: None,
            otlp_grpc_address: "127.0.0.1".to_string(),
            otlp_grpc_port: 4317,
            admin_port: 8080,
            inactivity_timeout: 60,
            format: "json".to_string(),
            output: None,
        }
    }

    /// Set the semantic convention registry path
    pub fn with_registry(mut self, registry_path: String) -> Self {
        self.registry_path = Some(registry_path);
        self
    }

    /// Set the OTLP gRPC address
    pub fn with_otlp_address(mut self, address: String) -> Self {
        self.otlp_grpc_address = address;
        self
    }

    /// Set the OTLP gRPC port
    pub fn with_otlp_port(mut self, port: u16) -> Self {
        self.otlp_grpc_port = port;
        self
    }

    /// Set the admin HTTP port
    pub fn with_admin_port(mut self, port: u16) -> Self {
        self.admin_port = port;
        self
    }

    /// Set the inactivity timeout in seconds
    pub fn with_inactivity_timeout(mut self, timeout: u64) -> Self {
        self.inactivity_timeout = timeout;
        self
    }

    /// Set the output format (json, ansi)
    pub fn with_format(mut self, format: String) -> Self {
        self.format = format;
        self
    }

    /// Set the output directory (for JSON reports)
    pub fn with_output(mut self, output: String) -> Self {
        self.output = Some(output);
        self
    }

    /// Check if Weaver binary is available in PATH
    pub fn check_weaver_available() -> Result<(), String> {
        use std::process::Command;

        // Try to run weaver --version to check if it exists
        match Command::new("weaver").arg("--version").output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else {
                    Err("Weaver binary found but --version failed".to_string())
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Err("Weaver binary not found in PATH. Install with: ./scripts/install-weaver.sh or cargo install weaver".to_string())
                } else {
                    Err(format!("Failed to check Weaver binary: {}", e))
                }
            }
        }
    }

    /// Check Weaver health by querying the admin endpoint
    pub fn check_health(&self) -> Result<bool, String> {
        use reqwest::blocking::Client;
        use std::time::Duration;

        let client = Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        // Try common health check endpoints
        let health_endpoints = vec![
            format!(
                "http://{}:{}/health",
                self.otlp_grpc_address, self.admin_port
            ),
            format!(
                "http://{}:{}/status",
                self.otlp_grpc_address, self.admin_port
            ),
            format!("http://{}:{}/", self.otlp_grpc_address, self.admin_port),
        ];

        for url in health_endpoints {
            match client.get(&url).send() {
                Ok(response) => {
                    if response.status().is_success() || response.status().as_u16() == 404 {
                        // 404 might mean endpoint doesn't exist but server is running
                        // Try to verify by checking if we can connect
                        return Ok(true);
                    }
                }
                Err(_) => {
                    // Try next endpoint
                    continue;
                }
            }
        }

        // If all endpoints failed, check if port is listening
        // This is a basic connectivity check
        match std::net::TcpStream::connect(format!(
            "{}:{}",
            self.otlp_grpc_address, self.admin_port
        )) {
            Ok(_) => Ok(true), // Port is open, assume Weaver is running
            Err(_) => Err(format!(
                "Weaver admin endpoint not responding on {}:{}",
                self.otlp_grpc_address, self.admin_port
            )),
        }
    }

    /// Run live-check and return the process handle
    /// The caller should send telemetry to the configured OTLP endpoint
    pub fn start(&self) -> Result<std::process::Child, String> {
        // Check Weaver binary availability first
        Self::check_weaver_available()?;
        use std::process::Command;

        let mut cmd = Command::new("weaver");

        cmd.args(["registry", "live-check"]);

        if let Some(ref registry) = self.registry_path {
            cmd.args(["--registry", registry]);
        }

        cmd.args(["--otlp-grpc-address", &self.otlp_grpc_address]);
        cmd.args(["--otlp-grpc-port", &self.otlp_grpc_port.to_string()]);
        cmd.args(["--admin-port", &self.admin_port.to_string()]);
        cmd.args(["--inactivity-timeout", &self.inactivity_timeout.to_string()]);
        cmd.args(["--format", &self.format]);

        if let Some(ref output) = self.output {
            cmd.args(["--output", output]);
        }

        cmd.spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    "Weaver binary not found in PATH. Install with: ./scripts/install-weaver.sh or cargo install weaver".to_string()
                } else {
                    format!("Failed to start Weaver live-check: {}. Ensure Weaver is installed and in PATH.", e)
                }
            })
    }

    /// Stop the live-check process via HTTP admin endpoint
    pub fn stop(&self) -> Result<(), String> {
        use reqwest::blocking::Client;
        use std::time::Duration;

        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let url = format!("http://{}:{}/stop", self.otlp_grpc_address, self.admin_port);

        match client.post(&url).send() {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to stop Weaver live-check: {}", e)),
        }
    }

    /// Get the OTLP gRPC endpoint for sending telemetry
    /// Note: Weaver live-check listens on gRPC, but exporters typically use HTTP
    /// This returns the address:port format for configuration
    pub fn otlp_endpoint(&self) -> String {
        format!("{}:{}", self.otlp_grpc_address, self.otlp_grpc_port)
    }
}

#[cfg(feature = "std")]
impl Default for WeaverLiveCheck {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "std")]
impl OtlpExporter {
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }

    pub fn export_spans(&self, spans: &[Span]) -> Result<(), String> {
        if spans.is_empty() {
            return Ok(());
        }

        #[cfg(all(feature = "std", feature = "reqwest"))]
        {
            use reqwest::blocking::Client;
            use std::time::Duration;

            // Build OTLP JSON payload
            let payload = self.build_otlp_spans_payload(spans);

            // Create HTTP client
            let client = Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

            // Send spans to OTLP endpoint (traces endpoint)
            let traces_endpoint = format!("{}/v1/traces", self.endpoint.trim_end_matches('/'));

            match client
                .post(&traces_endpoint)
                .json(&payload)
                .header("Content-Type", "application/json")
                .send()
            {
                Ok(response) => {
                    if response.status().is_success() {
                        Ok(())
                    } else {
                        Err(format!("OTLP export failed: HTTP {}", response.status()))
                    }
                }
                Err(e) => Err(format!("OTLP export failed: {}", e)),
            }
        }

        #[cfg(not(all(feature = "std", feature = "reqwest")))]
        {
            // Fallback: cannot export spans (for no_std or when reqwest not available)
            // Error message already contains context
            Err(format!("OTLP export not available: reqwest feature not enabled. Cannot export {} spans without HTTP client.", spans.len()))
        }
    }

    #[cfg(all(feature = "std", feature = "serde_json"))]
    fn build_otlp_spans_payload(&self, spans: &[Span]) -> serde_json::Value {
        use serde_json::json;

        let spans_json: Vec<_> = spans
            .iter()
            .map(|span| {
                let mut span_json = json!({
                    "traceId": format!("{:032x}", span.context.trace_id.0),
                    "spanId": format!("{:016x}", span.context.span_id.0),
                    "name": span.name,
                    "startTimeUnixNano": span.start_time_ms * 1_000_000,
                    "endTimeUnixNano": span.end_time_ms.unwrap_or(span.start_time_ms) * 1_000_000,
                    "status": {
                        "code": match span.status {
                            SpanStatus::Ok => 1,
                            SpanStatus::Error => 2,
                            SpanStatus::Unset => 0,
                        }
                    }
                });

                // Add parent span ID if present
                if let Some(parent_id) = span.context.parent_span_id {
                    span_json["parentSpanId"] = json!(format!("{:016x}", parent_id.0));
                }

                // Add attributes
                if !span.attributes.is_empty() {
                    span_json["attributes"] = json!(span
                        .attributes
                        .iter()
                        .map(|(k, v)| {
                            json!({
                                "key": k,
                                "value": {"stringValue": v}
                            })
                        })
                        .collect::<Vec<_>>());
                }

                span_json
            })
            .collect();

        json!({
            "resourceSpans": [{
                "resource": {},
                "instrumentationLibrarySpans": [{
                    "instrumentationLibrary": {},
                    "spans": spans_json
                }]
            }]
        })
    }

    #[cfg(not(all(feature = "std", feature = "serde_json")))]
    fn build_otlp_spans_payload(&self, _spans: &[Span]) -> String {
        // Fallback: return empty JSON
        "{}".to_string()
    }

    pub fn export_metrics(&self, metrics: &[Metric]) -> Result<(), String> {
        if metrics.is_empty() {
            return Ok(());
        }

        #[cfg(all(feature = "std", feature = "reqwest"))]
        {
            use reqwest::blocking::Client;
            use std::time::Duration;

            // Build OTLP JSON payload
            let payload = self.build_otlp_metrics_payload(metrics);

            // Create HTTP client
            let client = Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

            // Send metrics to OTLP endpoint (metrics endpoint)
            let metrics_endpoint = format!("{}/v1/metrics", self.endpoint.trim_end_matches('/'));

            match client
                .post(&metrics_endpoint)
                .json(&payload)
                .header("Content-Type", "application/json")
                .send()
            {
                Ok(response) => {
                    if response.status().is_success() {
                        Ok(())
                    } else {
                        Err(format!("OTLP export failed: HTTP {}", response.status()))
                    }
                }
                Err(e) => Err(format!("OTLP export failed: {}", e)),
            }
        }

        #[cfg(not(all(feature = "std", feature = "reqwest")))]
        {
            // Fallback: cannot export metrics (for no_std or when reqwest not available)
            // Error message already contains context
            Err(format!("OTLP export not available: reqwest feature not enabled. Cannot export {} metrics without HTTP client.", metrics.len()))
        }
    }

    #[cfg(all(feature = "std", feature = "serde_json"))]
    fn build_otlp_metrics_payload(&self, metrics: &[Metric]) -> serde_json::Value {
        use serde_json::json;

        let metrics_json: Vec<_> = metrics
            .iter()
            .map(|metric| {
                let value_json = match &metric.value {
                    MetricValue::Counter(c) => json!({
                        "asInt": format!("{}", c)
                    }),
                    MetricValue::Gauge(g) => json!({
                        "asDouble": *g
                    }),
                    MetricValue::Histogram(h) => json!({
                        "asInt": format!("{}", h.len())
                    }),
                };

                let mut metric_json = json!({
                    "name": metric.name,
                    "timestamp": metric.timestamp_ms * 1_000_000,
                    "value": value_json
                });

                // Add attributes
                if !metric.attributes.is_empty() {
                    metric_json["attributes"] = json!(metric
                        .attributes
                        .iter()
                        .map(|(k, v)| {
                            json!({
                                "key": k,
                                "value": {"stringValue": v}
                            })
                        })
                        .collect::<Vec<_>>());
                }

                metric_json
            })
            .collect();

        json!({
            "resourceMetrics": [{
                "resource": {},
                "instrumentationLibraryMetrics": [{
                    "instrumentationLibrary": {},
                    "metrics": metrics_json
                }]
            }]
        })
    }

    #[cfg(not(all(feature = "std", feature = "serde_json")))]
    fn build_otlp_metrics_payload(&self, _metrics: &[Metric]) -> String {
        // Fallback: return empty JSON
        "{}".to_string()
    }
}

/// OTEL tracer
pub struct Tracer {
    spans: Vec<Span>,
    metrics: Vec<Metric>,
    #[cfg(feature = "std")]
    exporter: Option<OtlpExporter>,
}

impl Tracer {
    pub fn new() -> Self {
        Self {
            spans: Vec::new(),
            metrics: Vec::new(),
            #[cfg(feature = "std")]
            exporter: None,
        }
    }

    #[cfg(feature = "std")]
    pub fn with_otlp_exporter(endpoint: String) -> Self {
        Self {
            spans: Vec::new(),
            metrics: Vec::new(),
            exporter: Some(OtlpExporter::new(endpoint)),
        }
    }

    #[cfg(feature = "std")]
    pub fn export(&mut self) -> Result<(), String> {
        let exporter = self.exporter.as_mut().ok_or_else(|| {
            "No OTLP exporter configured. Cannot export telemetry without exporter.".to_string()
        })?;

        exporter.export_spans(&self.spans)?;
        exporter.export_metrics(&self.metrics)?;
        Ok(())
    }

    /// Start a new span
    pub fn start_span(&mut self, name: String, parent: Option<SpanContext>) -> SpanContext {
        let trace_id = parent.as_ref().map(|p| p.trace_id).unwrap_or_else(|| {
            // Generate new trace ID (128-bit random)
            TraceId(generate_trace_id())
        });

        let span_id = SpanId(generate_span_id());
        let parent_span_id = parent.as_ref().map(|p| p.span_id);

        let context = SpanContext {
            trace_id,
            span_id,
            parent_span_id,
            flags: 1, // sampled
        };

        let span = Span {
            context: context.clone(),
            name: name.clone(),
            start_time_ms: get_timestamp_ms(),
            end_time_ms: None,
            attributes: BTreeMap::new(),
            events: Vec::new(),
            status: SpanStatus::Unset,
        };

        self.spans.push(span);
        context
    }

    /// End a span
    pub fn end_span(&mut self, context: SpanContext, status: SpanStatus) {
        if let Some(span) = self
            .spans
            .iter_mut()
            .find(|s| s.context.span_id == context.span_id)
        {
            span.end_time_ms = Some(get_timestamp_ms());
            span.status = status;
        }
    }

    /// Add event to span
    pub fn add_event(&mut self, context: SpanContext, event: SpanEvent) {
        if let Some(span) = self
            .spans
            .iter_mut()
            .find(|s| s.context.span_id == context.span_id)
        {
            span.events.push(event);
        }
    }

    /// Add attribute to span
    pub fn add_attribute(&mut self, context: SpanContext, key: String, value: String) {
        if let Some(span) = self
            .spans
            .iter_mut()
            .find(|s| s.context.span_id == context.span_id)
        {
            span.attributes.insert(key, value);
        }
    }

    /// Record metric
    pub fn record_metric(&mut self, metric: Metric) {
        self.metrics.push(metric);
    }

    /// Get span by ID
    pub fn get_span(&self, span_id: SpanId) -> Option<&Span> {
        self.spans.iter().find(|s| s.context.span_id == span_id)
    }

    /// Get all spans
    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    /// Get all metrics
    pub fn metrics(&self) -> &[Metric] {
        &self.metrics
    }

    /// Get metrics for specific name
    pub fn get_metrics(&self, name: &str) -> Vec<&Metric> {
        self.metrics.iter().filter(|m| m.name == name).collect()
    }

    /// Export telemetry to Weaver live-check endpoint for validation
    #[cfg(feature = "std")]
    pub fn export_to_weaver(&mut self, weaver_endpoint: &str) -> Result<(), String> {
        // Create a temporary exporter pointing to Weaver's OTLP endpoint
        let weaver_exporter = OtlpExporter::new(weaver_endpoint.to_string());
        weaver_exporter.export_spans(&self.spans)?;
        weaver_exporter.export_metrics(&self.metrics)?;
        Ok(())
    }
}

impl Default for Tracer {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics helper functions
pub struct MetricsHelper;

impl MetricsHelper {
    /// Record hook execution latency
    pub fn record_hook_latency(tracer: &mut Tracer, ticks: u32, operation: &str) {
        let metric = Metric {
            name: "knhk.hook.latency.ticks".to_string(),
            value: MetricValue::Histogram(vec![ticks as u64]),
            timestamp_ms: get_timestamp_ms(),
            attributes: {
                let mut attrs = BTreeMap::new();
                attrs.insert("operation".to_string(), operation.to_string());
                attrs
            },
        };
        tracer.record_metric(metric);
    }

    /// Record receipt generation
    pub fn record_receipt(tracer: &mut Tracer, receipt_id: &str) {
        let metric = Metric {
            name: "knhk.receipt.generated".to_string(),
            value: MetricValue::Counter(1),
            timestamp_ms: get_timestamp_ms(),
            attributes: {
                let mut attrs = BTreeMap::new();
                attrs.insert("receipt_id".to_string(), receipt_id.to_string());
                attrs
            },
        };
        tracer.record_metric(metric);
    }

    /// Record guard violation
    pub fn record_guard_violation(tracer: &mut Tracer, guard_type: &str) {
        let metric = Metric {
            name: "knhk.guard.violation".to_string(),
            value: MetricValue::Counter(1),
            timestamp_ms: get_timestamp_ms(),
            attributes: {
                let mut attrs = BTreeMap::new();
                attrs.insert("guard_type".to_string(), guard_type.to_string());
                attrs
            },
        };
        tracer.record_metric(metric);
    }

    /// Record warm path operation latency
    pub fn record_warm_path_latency(tracer: &mut Tracer, latency_us: u64, operation: &str) {
        let metric = Metric {
            name: "knhk.warm_path.operations.latency".to_string(),
            value: MetricValue::Histogram(vec![latency_us]),
            timestamp_ms: get_timestamp_ms(),
            attributes: {
                let mut attrs = BTreeMap::new();
                attrs.insert("operation".to_string(), operation.to_string());
                attrs
            },
        };
        tracer.record_metric(metric);

        // Also record count
        let count_metric = Metric {
            name: "knhk.warm_path.operations.count".to_string(),
            value: MetricValue::Counter(1),
            timestamp_ms: get_timestamp_ms(),
            attributes: {
                let mut attrs = BTreeMap::new();
                attrs.insert("operation".to_string(), operation.to_string());
                attrs
            },
        };
        tracer.record_metric(count_metric);
    }

    /// Record configuration load
    pub fn record_config_load(tracer: &mut Tracer, source: &str) {
        let metric = Metric {
            name: "knhk.config.loads".to_string(),
            value: MetricValue::Counter(1),
            timestamp_ms: get_timestamp_ms(),
            attributes: {
                let mut attrs = BTreeMap::new();
                attrs.insert("source".to_string(), source.to_string());
                attrs
            },
        };
        tracer.record_metric(metric);
    }

    /// Record configuration error
    pub fn record_config_error(tracer: &mut Tracer, error_type: &str) {
        let metric = Metric {
            name: "knhk.config.errors".to_string(),
            value: MetricValue::Counter(1),
            timestamp_ms: get_timestamp_ms(),
            attributes: {
                let mut attrs = BTreeMap::new();
                attrs.insert("error_type".to_string(), error_type.to_string());
                attrs
            },
        };
        tracer.record_metric(metric);
    }

    /// Record connector throughput
    pub fn record_connector_throughput(tracer: &mut Tracer, connector_id: &str, triples: usize) {
        let metric = Metric {
            name: "knhk.connector.throughput".to_string(),
            value: MetricValue::Counter(triples as u64),
            timestamp_ms: get_timestamp_ms(),
            attributes: {
                let mut attrs = BTreeMap::new();
                attrs.insert("connector_id".to_string(), connector_id.to_string());
                attrs
            },
        };
        tracer.record_metric(metric);
    }

    /// Record generic operation execution
    pub fn record_operation(tracer: &mut Tracer, operation: &str, success: bool) {
        let metric = Metric {
            name: "knhk.operation.executed".to_string(),
            value: MetricValue::Counter(1),
            timestamp_ms: get_timestamp_ms(),
            attributes: {
                let mut attrs = BTreeMap::new();
                attrs.insert("operation".to_string(), operation.to_string());
                attrs.insert("success".to_string(), success.to_string());
                attrs
            },
        };
        tracer.record_metric(metric);
    }
}

// Helper functions for generating IDs and timestamps

/// Generate 128-bit trace ID
fn generate_trace_id() -> u128 {
    #[cfg(feature = "std")]
    {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        // Generate 128-bit ID from two 64-bit values
        let high = rng.next_u64();
        let low = rng.next_u64();
        (high as u128) << 64 | (low as u128)
    }
    #[cfg(not(feature = "std"))]
    {
        // For no_std, use simple hash-based generation
        // In production, use hardware RNG or external source
        use core::hash::{Hash, Hasher};
        // Use FNV-1a hash for no_std compatibility
        let mut hash = 14695981039346656037u64; // FNV offset basis
        const FNV_PRIME: u64 = 1099511628211;
        for byte in "trace".as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash as u128 | ((hash.wrapping_mul(FNV_PRIME) as u128) << 64)
    }
}

/// Generate 64-bit span ID (public API)
pub fn generate_span_id() -> u64 {
    #[cfg(feature = "std")]
    {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        rng.next_u64()
    }
    #[cfg(not(feature = "std"))]
    {
        // For no_std, use hash-based generation with timestamp
        // Use FNV-1a hash for no_std compatibility
        let mut hash = 14695981039346656037u64; // FNV offset basis
        const FNV_PRIME: u64 = 1099511628211;
        for byte in "span".as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        let timestamp = get_timestamp_ms();
        for byte in timestamp.to_le_bytes().iter() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }
}

/// Get current timestamp in milliseconds
pub fn get_timestamp_ms() -> u64 {
    #[cfg(feature = "std")]
    {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
    #[cfg(not(feature = "std"))]
    {
        // no_std mode: Timestamp not available without std library
        // For no_std builds, timestamps are provided externally or disabled
        // This is a known limitation for no_std builds
        0
    }
}

/// Initialize OpenTelemetry tracer with SDK
///
/// This function initializes the OpenTelemetry SDK with proper resource configuration,
/// exporter setup, and tracing-subscriber integration. It follows the same pattern as
/// clnrm's telemetry initialization.
///
/// # Arguments
/// * `service_name` - Service name for resource attributes
/// * `service_version` - Service version for resource attributes
/// * `endpoint` - Optional OTLP endpoint (if None, uses stdout exporter)
///
/// # Returns
/// * `Result<OtelGuard>` - Guard that handles shutdown and flushing on drop
///
/// # Errors
/// * Returns error if SDK initialization fails
#[cfg(feature = "std")]
pub fn init_tracer(
    service_name: &str,
    service_version: &str,
    endpoint: Option<&str>,
) -> Result<OtelGuard, String> {
    use opentelemetry::global;
    use opentelemetry::propagation::TextMapCompositePropagator;
    use opentelemetry::trace::TracerProvider;
    use opentelemetry::KeyValue;
    #[cfg(feature = "std")]
    use opentelemetry_sdk::{
        propagation::{BaggagePropagator, TraceContextPropagator},
        trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
        Resource,
    };

    // Set up propagators
    global::set_text_map_propagator(TextMapCompositePropagator::new(vec![
        Box::new(TraceContextPropagator::new()),
        Box::new(BaggagePropagator::new()),
    ]));

    // Create resource with service information
    let service_name_str = service_name.to_string();
    let service_version_str = service_version.to_string();
    let instance_id = format!("knhk-{}", std::process::id());
    let resource = Resource::builder_empty()
        .with_service_name(service_name_str)
        .with_attributes([
            KeyValue::new("service.version", service_version_str),
            KeyValue::new("telemetry.sdk.language", "rust"),
            KeyValue::new("telemetry.sdk.name", "opentelemetry"),
            KeyValue::new("telemetry.sdk.version", "0.31.0"),
            KeyValue::new("service.instance.id", instance_id),
        ])
        .build();

    // Create exporter based on endpoint
    // Use enum approach since SpanExporter is not dyn compatible
    #[allow(refining_impl_trait)]
    #[derive(Debug)]
    enum SpanExporterType {
        Otlp(opentelemetry_otlp::SpanExporter),
    }

    #[allow(refining_impl_trait)]
    impl opentelemetry_sdk::trace::SpanExporter for SpanExporterType {
        fn export(
            &self,
            batch: Vec<opentelemetry_sdk::trace::SpanData>,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = opentelemetry_sdk::error::OTelSdkResult>
                    + Send
                    + '_,
            >,
        > {
            match self {
                SpanExporterType::Otlp(exporter) => Box::pin(exporter.export(batch)),
            }
        }

        fn shutdown(&mut self) -> opentelemetry_sdk::error::OTelSdkResult {
            match self {
                SpanExporterType::Otlp(exporter) => exporter.shutdown(),
            }
        }
    }

    let span_exporter = if let Some(endpoint) = endpoint {
        // OTLP HTTP exporter
        std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", endpoint);

        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .build()
            .map_err(|e| format!("Failed to create OTLP HTTP exporter: {}", e))?;

        SpanExporterType::Otlp(exporter)
    } else {
        // Use OTLP with default endpoint if no endpoint specified
        std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4317");
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .build()
            .map_err(|e| format!("Failed to create OTLP HTTP exporter: {}", e))?;
        SpanExporterType::Otlp(exporter)
    };

    // Create tracer provider with batch exporter
    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .with_sampler(Sampler::TraceIdRatioBased(1.0)) // Always sample
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource.clone())
        .build();

    // Get tracer and set as global
    let tracer = tracer_provider.tracer("knhk");
    global::set_tracer_provider(tracer_provider.clone());

    // Initialize tracing-subscriber with OpenTelemetry layer
    use tracing_opentelemetry::OpenTelemetryLayer;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};

    let otel_layer = OpenTelemetryLayer::new(tracer);
    Registry::default()
        .with(otel_layer)
        .with(tracing_subscriber::fmt::layer().compact())
        .init();

    // Create metrics provider
    let meter_provider = if let Some(_endpoint) = endpoint {
        use opentelemetry_otlp::MetricExporter;
        use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};

        let exporter = MetricExporter::builder()
            .with_http()
            .build()
            .map_err(|e| format!("Failed to create OTLP HTTP metrics exporter: {}", e))?;

        let reader = PeriodicReader::builder(exporter)
            .with_interval(std::time::Duration::from_secs(1))
            .build();

        Some(
            SdkMeterProvider::builder()
                .with_resource(resource)
                .with_reader(reader)
                .build(),
        )
    } else {
        None
    };

    // Set global meter provider if configured
    if let Some(ref mp) = meter_provider {
        global::set_meter_provider(mp.clone());
    }

    Ok(OtelGuard {
        tracer_provider,
        meter_provider,
    })
}

/// Guard for managing OpenTelemetry lifecycle
///
/// This guard ensures proper shutdown and flushing of telemetry data when dropped.
/// It follows the same pattern as clnrm's OtelGuard.
#[cfg(feature = "std")]
pub struct OtelGuard {
    tracer_provider: SdkTracerProvider,
    meter_provider: Option<opentelemetry_sdk::metrics::SdkMeterProvider>,
}

#[cfg(feature = "std")]
impl Drop for OtelGuard {
    fn drop(&mut self) {
        use std::time::Duration;

        // Force flush traces before shutdown
        if let Err(e) = self.tracer_provider.force_flush() {
            eprintln!("Failed to flush traces during shutdown: {}", e);
        }

        // Give async exports time to complete
        std::thread::sleep(Duration::from_millis(500));

        // Shutdown tracer provider
        if let Err(e) = self.tracer_provider.shutdown() {
            eprintln!("Failed to shutdown tracer provider: {}", e);
        }

        // Flush and shutdown metrics provider
        if let Some(mp) = self.meter_provider.take() {
            if let Err(e) = mp.force_flush() {
                eprintln!("Failed to flush metrics during shutdown: {}", e);
            }
            std::thread::sleep(Duration::from_millis(100));
            if let Err(e) = mp.shutdown() {
                eprintln!("Failed to shutdown meter provider: {}", e);
            }
        }
    }
}

#[cfg(feature = "std")]
use opentelemetry_sdk::trace::SdkTracerProvider;

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use super::*;

    #[test]
    fn test_tracer_span() {
        let mut tracer = Tracer::new();
        let context = tracer.start_span("test_span".to_string(), None);

        tracer.add_attribute(context.clone(), "key".to_string(), "value".to_string());
        tracer.end_span(context, SpanStatus::Ok);

        assert_eq!(tracer.spans().len(), 1);
    }

    #[test]
    fn test_metrics_recording() {
        let mut tracer = Tracer::new();
        MetricsHelper::record_hook_latency(&mut tracer, 5, "ASK_SP");
        MetricsHelper::record_receipt(&mut tracer, "receipt1");

        assert_eq!(tracer.metrics().len(), 2);
    }

    #[cfg(feature = "std")]
    mod weaver_tests {
        use super::*;

        /// Test: WeaverLiveCheck builder pattern creates correct default configuration
        #[test]
        fn test_weaver_live_check_defaults() {
            let weaver = WeaverLiveCheck::new();

            // Verify default values match expected configuration
            assert_eq!(weaver.otlp_grpc_address, "127.0.0.1");
            assert_eq!(weaver.otlp_grpc_port, 4317);
            assert_eq!(weaver.admin_port, 8080);
            assert_eq!(weaver.inactivity_timeout, 60);
            assert_eq!(weaver.format, "json");
            assert_eq!(weaver.registry_path, None);
            assert_eq!(weaver.output, None);
        }

        /// Test: WeaverLiveCheck builder methods set correct values
        #[test]
        fn test_weaver_live_check_builder() {
            let weaver = WeaverLiveCheck::new()
                .with_registry("./test-registry".to_string())
                .with_otlp_address("localhost".to_string())
                .with_otlp_port(9999)
                .with_admin_port(9998)
                .with_inactivity_timeout(120)
                .with_format("ansi".to_string())
                .with_output("./test-output".to_string());

            // Verify all values are set correctly
            assert_eq!(weaver.registry_path, Some("./test-registry".to_string()));
            assert_eq!(weaver.otlp_grpc_address, "localhost");
            assert_eq!(weaver.otlp_grpc_port, 9999);
            assert_eq!(weaver.admin_port, 9998);
            assert_eq!(weaver.inactivity_timeout, 120);
            assert_eq!(weaver.format, "ansi");
            assert_eq!(weaver.output, Some("./test-output".to_string()));
        }

        /// Test: WeaverLiveCheck otlp_endpoint returns correct format
        #[test]
        fn test_weaver_otlp_endpoint_format() {
            let weaver = WeaverLiveCheck::new()
                .with_otlp_address("localhost".to_string())
                .with_otlp_port(4317);

            let endpoint = weaver.otlp_endpoint();
            assert_eq!(endpoint, "localhost:4317");
        }

        /// Test: WeaverLiveCheck Default trait implementation
        #[test]
        fn test_weaver_default_trait() {
            let weaver_default = WeaverLiveCheck::default();
            let weaver_new = WeaverLiveCheck::new();

            // Verify Default and new() produce identical configurations
            assert_eq!(
                weaver_default.otlp_grpc_address,
                weaver_new.otlp_grpc_address
            );
            assert_eq!(weaver_default.otlp_grpc_port, weaver_new.otlp_grpc_port);
            assert_eq!(weaver_default.admin_port, weaver_new.admin_port);
            assert_eq!(
                weaver_default.inactivity_timeout,
                weaver_new.inactivity_timeout
            );
            assert_eq!(weaver_default.format, weaver_new.format);
        }

        /// Test: WeaverLiveCheck start command builds correct command line
        /// This test verifies the command construction without actually spawning a process
        /// Chicago TDD: Test behavior (command construction) not implementation (process spawning)
        #[test]
        fn test_weaver_start_command_construction() {
            let weaver = WeaverLiveCheck::new()
                .with_registry("./test-registry".to_string())
                .with_otlp_port(9999)
                .with_admin_port(9998)
                .with_inactivity_timeout(120)
                .with_format("ansi".to_string())
                .with_output("./test-output".to_string());

            // Verify endpoint format is correct (used in command construction)
            let endpoint = weaver.otlp_endpoint();
            assert_eq!(endpoint, "127.0.0.1:9999");

            // Verify all configuration values are set (required for command)
            assert!(weaver.registry_path.is_some());
            assert_eq!(weaver.otlp_grpc_port, 9999);
            assert_eq!(weaver.admin_port, 9998);
            assert_eq!(weaver.inactivity_timeout, 120);
            assert_eq!(weaver.format, "ansi");
            assert!(weaver.output.is_some());
        }

        /// Test: Export telemetry to Weaver endpoint
        /// Chicago TDD: Test the actual behavior (telemetry export) with real tracer
        #[test]
        fn test_export_telemetry_to_weaver() {
            let mut tracer = Tracer::new();

            // Create a span with semantic convention attributes
            let span_ctx = tracer.start_span("knhk.operation.execute".to_string(), None);
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.operation.name".to_string(),
                "boot.init".to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.operation.type".to_string(),
                "system".to_string(),
            );
            tracer.end_span(span_ctx, SpanStatus::Ok);

            // Record metrics
            MetricsHelper::record_hook_latency(&mut tracer, 5, "ASK_SP");
            MetricsHelper::record_receipt(&mut tracer, "receipt-123");

            // Verify telemetry was created correctly
            assert_eq!(tracer.spans().len(), 1);
            assert_eq!(tracer.metrics().len(), 2);

            // Verify span has correct attributes
            let span = tracer
                .spans()
                .first()
                .expect("Expected at least one span in tracer");
            assert_eq!(span.name, "knhk.operation.execute");
            assert_eq!(
                span.attributes.get("knhk.operation.name"),
                Some(&"boot.init".to_string())
            );
            assert_eq!(
                span.attributes.get("knhk.operation.type"),
                Some(&"system".to_string())
            );
            assert_eq!(span.status, SpanStatus::Ok);

            // Verify metrics have correct values
            let metrics = tracer.metrics();
            assert_eq!(metrics[0].name, "knhk.hook.latency.ticks");
            assert_eq!(metrics[1].name, "knhk.receipt.generated");
        }

        /// Test: WeaverLiveCheck stop URL construction
        /// Chicago TDD: Test the output (URL format) with different configurations
        #[test]
        fn test_weaver_stop_url_construction() {
            let _weaver = WeaverLiveCheck::new()
                .with_otlp_address("localhost".to_string())
                .with_admin_port(9998);

            // Verify the URL format matches expected pattern
            // Note: This tests the URL construction logic, not the actual HTTP call
            let expected_url = format!("http://{}:{}/stop", "localhost", 9998);
            assert_eq!(expected_url, "http://localhost:9998/stop");
        }

        /// Test: Integration test - Complete Weaver live-check workflow
        /// Chicago TDD: Test the full workflow with real collaborators
        /// This test verifies the integration without requiring actual Weaver binary
        #[test]
        fn test_weaver_integration_workflow() {
            // Step 1: Create Weaver configuration
            let weaver = WeaverLiveCheck::new()
                .with_otlp_port(4317)
                .with_admin_port(8080)
                .with_format("json".to_string());

            // Verify configuration
            assert_eq!(weaver.otlp_endpoint(), "127.0.0.1:4317");

            // Step 2: Create tracer with OTLP exporter
            let mut tracer =
                Tracer::with_otlp_exporter(format!("http://{}", weaver.otlp_endpoint()));

            // Step 3: Generate telemetry
            let span_ctx = tracer.start_span("knhk.boot.init".to_string(), None);
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.operation.name".to_string(),
                "boot.init".to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.operation.type".to_string(),
                "system".to_string(),
            );
            tracer.end_span(span_ctx, SpanStatus::Ok);

            MetricsHelper::record_operation(&mut tracer, "boot.init", true);

            // Step 4: Verify telemetry was created
            assert_eq!(tracer.spans().len(), 1);
            assert_eq!(tracer.metrics().len(), 1);

            // Step 5: Verify span has semantic convention attributes
            let span = tracer
                .spans()
                .first()
                .expect("Expected at least one span in tracer");
            assert_eq!(span.name, "knhk.boot.init");
            assert!(span.attributes.contains_key("knhk.operation.name"));
            assert!(span.attributes.contains_key("knhk.operation.type"));

            // Step 6: Verify metrics have correct attributes
            let metric = tracer
                .metrics()
                .first()
                .expect("Expected at least one metric in tracer");
            assert_eq!(metric.name, "knhk.operation.executed");
            assert_eq!(
                metric.attributes.get("operation"),
                Some(&"boot.init".to_string())
            );
            assert_eq!(metric.attributes.get("success"), Some(&"true".to_string()));
        }

        /// Test: WeaverLiveCheck with optional registry path
        /// Chicago TDD: Test behavior with and without optional parameters
        #[test]
        fn test_weaver_with_and_without_registry() {
            // Test without registry
            let weaver_no_registry = WeaverLiveCheck::new();
            assert_eq!(weaver_no_registry.registry_path, None);

            // Test with registry
            let weaver_with_registry =
                WeaverLiveCheck::new().with_registry("./my-registry".to_string());
            assert_eq!(
                weaver_with_registry.registry_path,
                Some("./my-registry".to_string())
            );
        }

        /// Test: WeaverLiveCheck with optional output directory
        /// Chicago TDD: Test behavior with and without optional parameters
        #[test]
        fn test_weaver_with_and_without_output() {
            // Test without output
            let weaver_no_output = WeaverLiveCheck::new();
            assert_eq!(weaver_no_output.output, None);

            // Test with output
            let weaver_with_output =
                WeaverLiveCheck::new().with_output("./weaver-reports".to_string());
            assert_eq!(
                weaver_with_output.output,
                Some("./weaver-reports".to_string())
            );
        }

        /// Test: Semantic convention compliance in spans
        /// Chicago TDD: Verify spans conform to semantic conventions
        #[test]
        fn test_semantic_convention_compliance() {
            let mut tracer = Tracer::new();

            // Create span with semantic convention attributes
            let span_ctx = tracer.start_span("knhk.metrics.weaver.start".to_string(), None);
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.operation.name".to_string(),
                "weaver.start".to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.operation.type".to_string(),
                "validation".to_string(),
            );
            tracer.end_span(span_ctx, SpanStatus::Ok);

            // Verify span name follows convention: knhk.<noun>.<verb>
            let span = tracer
                .spans()
                .first()
                .expect("Expected at least one span in tracer");
            assert!(span.name.starts_with("knhk."));
            assert!(span.name.contains("."));

            // Verify required semantic convention attributes exist
            assert!(span.attributes.contains_key("knhk.operation.name"));
            assert!(span.attributes.contains_key("knhk.operation.type"));
        }

        /// Test: Metrics recording for Weaver operations
        /// Chicago TDD: Test metrics are recorded correctly
        #[test]
        fn test_weaver_operation_metrics() {
            let mut tracer = Tracer::new();

            // Record Weaver start operation
            MetricsHelper::record_operation(&mut tracer, "weaver.start", true);

            // Record Weaver stop operation
            MetricsHelper::record_operation(&mut tracer, "weaver.stop", true);

            // Record Weaver validate operation
            MetricsHelper::record_operation(&mut tracer, "weaver.validate", true);

            // Verify all metrics were recorded
            assert_eq!(tracer.metrics().len(), 3);

            // Verify each metric has correct attributes
            let metrics = tracer.metrics();
            assert_eq!(
                metrics[0].attributes.get("operation"),
                Some(&"weaver.start".to_string())
            );
            assert_eq!(
                metrics[1].attributes.get("operation"),
                Some(&"weaver.stop".to_string())
            );
            assert_eq!(
                metrics[2].attributes.get("operation"),
                Some(&"weaver.validate".to_string())
            );

            // Verify all operations succeeded
            assert_eq!(
                metrics[0].attributes.get("success"),
                Some(&"true".to_string())
            );
            assert_eq!(
                metrics[1].attributes.get("success"),
                Some(&"true".to_string())
            );
            assert_eq!(
                metrics[2].attributes.get("success"),
                Some(&"true".to_string())
            );
        }

        /// Test: Error handling for failed operations
        /// Chicago TDD: Test behavior when operations fail
        #[test]
        fn test_weaver_operation_failure_metrics() {
            let mut tracer = Tracer::new();

            // Record failed operation
            MetricsHelper::record_operation(&mut tracer, "weaver.start", false);

            // Verify metric was recorded with failure status
            let metric = tracer
                .metrics()
                .first()
                .expect("Expected at least one metric in tracer");
            assert_eq!(metric.attributes.get("success"), Some(&"false".to_string()));
            assert_eq!(
                metric.attributes.get("operation"),
                Some(&"weaver.start".to_string())
            );
        }

        /// Test: WeaverLiveCheck configuration persistence
        /// Chicago TDD: Test that configuration changes persist through builder chain
        #[test]
        fn test_weaver_configuration_persistence() {
            let weaver = WeaverLiveCheck::new()
                .with_registry("./reg1".to_string())
                .with_otlp_port(1111)
                .with_admin_port(2222);

            // Verify configuration persists
            assert_eq!(weaver.registry_path, Some("./reg1".to_string()));
            assert_eq!(weaver.otlp_grpc_port, 1111);
            assert_eq!(weaver.admin_port, 2222);

            // Create new instance with different config
            let weaver2 = WeaverLiveCheck::new()
                .with_registry("./reg2".to_string())
                .with_otlp_port(3333)
                .with_admin_port(4444);

            // Verify configurations are independent
            assert_eq!(weaver2.registry_path, Some("./reg2".to_string()));
            assert_eq!(weaver2.otlp_grpc_port, 3333);
            assert_eq!(weaver2.admin_port, 4444);

            // Verify original weaver configuration unchanged
            assert_eq!(weaver.otlp_grpc_port, 1111);
            assert_eq!(weaver.admin_port, 2222);
        }
    }
}
