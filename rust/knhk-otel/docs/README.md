# knhk-otel Documentation

OpenTelemetry integration for observability, metrics, and distributed tracing.

## Overview

The `knhk-otel` crate provides comprehensive OpenTelemetry integration for KNHK operations, including span generation, metrics collection, distributed tracing, and Weaver.ai live-check integration for semantic convention validation.

## Quick Start

```rust
use knhk_otel::{Tracer, SpanStatus, MetricsHelper};

// Create tracer
let mut tracer = Tracer::new();

// Start span
let span_ctx = tracer.start_span("knhk.operation.execute".to_string(), None);

// Add attributes
tracer.add_attribute(span_ctx.clone(), "knhk.operation.name".to_string(), "boot.init".to_string());

// Record metrics
MetricsHelper::record_hook_latency(&mut tracer, 5, "ASK_SP");

// End span
tracer.end_span(span_ctx, SpanStatus::Ok);
```

## Core Components

### Tracer

Main tracer for span and metric management:

```rust
pub struct Tracer {
    spans: Vec<Span>,
    metrics: Vec<Metric>,
    #[cfg(feature = "std")]
    exporter: Option<OtlpExporter>,
}
```

**Methods:**
- `new()` - Create tracer without exporter
- `with_otlp_exporter(endpoint)` - Create tracer with OTLP exporter
- `start_span(name, parent)` - Start new span
- `end_span(context, status)` - End span with status
- `add_attribute(context, key, value)` - Add attribute to span
- `add_event(context, event)` - Add event to span
- `record_metric(metric)` - Record metric
- `export()` - Export spans and metrics to OTLP endpoint
- `export_to_weaver(endpoint)` - Export to Weaver live-check endpoint

### Span Types

**SpanContext:**
```rust
pub struct SpanContext {
    pub trace_id: TraceId,        // 128-bit trace ID
    pub span_id: SpanId,           // 64-bit span ID
    pub parent_span_id: Option<SpanId>,
    pub flags: u8,
}
```

**Span:**
```rust
pub struct Span {
    pub context: SpanContext,
    pub name: String,
    pub start_time_ms: u64,
    pub end_time_ms: Option<u64>,
    pub attributes: Attributes,
    pub events: Vec<SpanEvent>,
    pub status: SpanStatus,
}
```

**SpanStatus:**
```rust
pub enum SpanStatus {
    Ok,
    Error,
    Unset,
}
```

### Metrics

**Metric:**
```rust
pub struct Metric {
    pub name: String,
    pub value: MetricValue,
    pub timestamp_ms: u64,
    pub attributes: Attributes,
}
```

**MetricValue:**
```rust
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<u64>),
}
```

### OtlpExporter

OTLP exporter for sending spans/metrics to collectors:

```rust
#[cfg(feature = "std")]
pub struct OtlpExporter {
    endpoint: String,
}

impl OtlpExporter {
    pub fn new(endpoint: String) -> Self;
    pub fn export_spans(&self, spans: &[Span]) -> Result<(), String>;
    pub fn export_metrics(&self, metrics: &[Metric]) -> Result<(), String>;
}
```

### WeaverLiveCheck

Weaver.ai live-check integration for semantic convention validation:

```rust
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
```

**Builder Methods:**
- `with_registry(path)` - Set semantic convention registry path
- `with_otlp_address(address)` - Set OTLP gRPC address
- `with_otlp_port(port)` - Set OTLP gRPC port
- `with_admin_port(port)` - Set admin HTTP port
- `with_inactivity_timeout(seconds)` - Set inactivity timeout
- `with_format(format)` - Set output format (json, ansi)
- `with_output(path)` - Set output directory

**Methods:**
- `start()` - Start Weaver live-check process
- `stop()` - Stop Weaver live-check via HTTP admin endpoint
- `otlp_endpoint()` - Get OTLP endpoint address:port

## Usage Examples

### Basic Tracing

```rust
use knhk_otel::{Tracer, SpanStatus};

let mut tracer = Tracer::new();

// Start root span
let root_ctx = tracer.start_span("knhk.boot.init".to_string(), None);
tracer.add_attribute(root_ctx.clone(), "knhk.operation.name".to_string(), "boot.init".to_string());

// Start child span
let child_ctx = tracer.start_span("knhk.hook.execute".to_string(), Some(root_ctx.clone()));
tracer.add_attribute(child_ctx.clone(), "knhk.hook.id".to_string(), "hook-123".to_string());

// End child span
tracer.end_span(child_ctx, SpanStatus::Ok);

// End root span
tracer.end_span(root_ctx, SpanStatus::Ok);
```

### Metrics Recording

```rust
use knhk_otel::{Tracer, MetricsHelper};

let mut tracer = Tracer::new();

// Record hook execution latency
MetricsHelper::record_hook_latency(&mut tracer, 5, "ASK_SP");

// Record receipt generation
MetricsHelper::record_receipt(&mut tracer, "receipt-123");

// Record guard violation
MetricsHelper::record_guard_violation(&mut tracer, "max_run_len");

// Record warm path operation latency
MetricsHelper::record_warm_path_latency(&mut tracer, 100, "query.execute");

// Record connector throughput
MetricsHelper::record_connector_throughput(&mut tracer, "kafka-connector", 1000);

// Record generic operation
MetricsHelper::record_operation(&mut tracer, "boot.init", true);
```

### OTLP Export

```rust
use knhk_otel::{Tracer, SpanStatus};

// Create tracer with OTLP exporter
let mut tracer = Tracer::with_otlp_exporter("http://localhost:4317".to_string());

// Generate spans and metrics
let span_ctx = tracer.start_span("knhk.operation.execute".to_string(), None);
tracer.end_span(span_ctx, SpanStatus::Ok);

MetricsHelper::record_operation(&mut tracer, "operation", true);

// Export to OTLP endpoint
tracer.export()?;
```

### Weaver Live-Check Integration

```rust
use knhk_otel::{WeaverLiveCheck, Tracer, SpanStatus, MetricsHelper};

// Create Weaver live-check configuration
let weaver = WeaverLiveCheck::new()
    .with_registry("./semantic-conventions".to_string())
    .with_otlp_port(4317)
    .with_admin_port(8080)
    .with_format("json".to_string())
    .with_output("./weaver-reports".to_string());

// Start Weaver live-check
let mut weaver_process = weaver.start()?;

// Create tracer pointing to Weaver endpoint
let mut tracer = Tracer::with_otlp_exporter(format!("http://{}", weaver.otlp_endpoint()));

// Generate telemetry with semantic conventions
let span_ctx = tracer.start_span("knhk.operation.execute".to_string(), None);
tracer.add_attribute(span_ctx.clone(), "knhk.operation.name".to_string(), "boot.init".to_string());
tracer.add_attribute(span_ctx.clone(), "knhk.operation.type".to_string(), "system".to_string());
tracer.end_span(span_ctx, SpanStatus::Ok);

MetricsHelper::record_operation(&mut tracer, "boot.init", true);

// Export to Weaver for validation
tracer.export_to_weaver(&weaver.otlp_endpoint())?;

// Stop Weaver live-check
weaver.stop()?;
```

### Span Events

```rust
use knhk_otel::{Tracer, SpanEvent, SpanStatus};
use alloc::collections::BTreeMap;

let mut tracer = Tracer::new();
let span_ctx = tracer.start_span("knhk.hook.execute".to_string(), None);

// Add event to span
let event = SpanEvent {
    name: "hook.started".to_string(),
    timestamp_ms: current_timestamp(),
    attributes: {
        let mut attrs = BTreeMap::new();
        attrs.insert("hook_id".to_string(), "hook-123".to_string());
        attrs
    },
};
tracer.add_event(span_ctx.clone(), event);

tracer.end_span(span_ctx, SpanStatus::Ok);
```

### Custom Metrics

```rust
use knhk_otel::{Tracer, Metric, MetricValue};
use alloc::collections::BTreeMap;

let mut tracer = Tracer::new();

// Create custom counter metric
let metric = Metric {
    name: "knhk.custom.counter".to_string(),
    value: MetricValue::Counter(42),
    timestamp_ms: current_timestamp(),
    attributes: {
        let mut attrs = BTreeMap::new();
        attrs.insert("source".to_string(), "custom".to_string());
        attrs
    },
};
tracer.record_metric(metric);

// Create custom gauge metric
let gauge = Metric {
    name: "knhk.custom.gauge".to_string(),
    value: MetricValue::Gauge(3.14),
    timestamp_ms: current_timestamp(),
    attributes: BTreeMap::new(),
};
tracer.record_metric(gauge);
```

## Key Features

- **Span Generation**: Real OTEL-compatible span IDs (not placeholders)
- **Distributed Tracing**: Trace ID propagation across services
- **Metrics Collection**: Counter, Gauge, Histogram metrics
- **OTLP Export**: Export spans and metrics to OTLP collectors
- **Weaver Integration**: Semantic convention validation via Weaver.ai
- **Semantic Conventions**: Follows KNHK semantic conventions (knhk.*)
- **no_std Support**: Works in no_std environments (limited features)

## Semantic Conventions

KNHK follows semantic conventions for span names and attributes:

**Span Names:**
- `knhk.operation.execute` - Operation execution
- `knhk.hook.execute` - Hook execution
- `knhk.boot.init` - Boot initialization
- `knhk.metrics.weaver.start` - Weaver start

**Attributes:**
- `knhk.operation.name` - Operation name
- `knhk.operation.type` - Operation type (system, validation, etc.)
- `knhk.hook.id` - Hook identifier
- `knhk.receipt.id` - Receipt identifier

**Metrics:**
- `knhk.hook.latency.ticks` - Hook execution latency in ticks
- `knhk.receipt.generated` - Receipt generation counter
- `knhk.guard.violation` - Guard violation counter
- `knhk.warm_path.operations.latency` - Warm path operation latency
- `knhk.connector.throughput` - Connector throughput

## Dependencies

- `reqwest` (optional, std feature) - HTTP client for OTLP export
- `serde_json` (optional, std feature) - JSON serialization
- `rand` (optional, std feature) - Random ID generation

## Feature Flags

- `std` - Enables std library features (OTLP export, Weaver integration, random ID generation)
- `reqwest` - Enables HTTP client for OTLP export
- `serde_json` - Enables JSON serialization

## Performance

- **Span Creation**: O(1) constant-time
- **Metric Recording**: O(1) constant-time
- **OTLP Export**: O(n) where n = span/metric count (network I/O)
- **ID Generation**: O(1) constant-time (std) or hash-based (no_std)

## Related Documentation

- [Architecture](../../../docs/architecture.md) - System architecture
- [Performance](../../../docs/performance.md) - Performance guide
- [Weaver Integration](../../../docs/weaver-integration.md) - Weaver.ai integration guide
- [Examples](../../../rust/knhk-otel/examples/weaver_live_check.rs) - Weaver live-check example
