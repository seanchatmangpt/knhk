# Telemetry Troubleshooting Guide

**Purpose**: Diagnose and fix OpenTelemetry instrumentation issues
**Target**: Schema-validated telemetry with ≤5% overhead
**Validation**: Weaver live-check must pass

---

## Understanding Telemetry in KNHK

**Three telemetry signals:**
1. **Traces (Spans)**: Distributed request tracking
2. **Metrics**: Quantitative measurements
3. **Logs**: Structured debugging information

**Validation hierarchy:**
1. Weaver schema check (schema valid)
2. Weaver live-check (runtime matches schema)
3. OTLP collector receiving data
4. Telemetry visualized in backend (Jaeger, Prometheus, etc.)

---

## Issue 1: No Telemetry Data Received

### Symptom
```bash
$ weaver registry live-check --registry /home/user/knhk/registry/
Error: No telemetry data received from application
```

### Diagnosis

**Step 1: Verify application running:**
```bash
# Check if process running
ps aux | grep knhk

# If not, start it
cargo run --bin knhk-cli -- query ask "ASK { ?s ?p ?o }"
```

**Step 2: Verify OTLP exporter initialized:**
```rust
use knhk_otel::init_tracer;

// Check initialization
let guard = init_tracer("knhk-test", "1.0.0", Some("http://localhost:4318"))
    .expect("Failed to init tracer");

println!("Tracer initialized");
// Don't forget to keep guard alive until telemetry is flushed
```

**Step 3: Verify OTLP collector running:**
```bash
# Check if collector is listening
curl http://localhost:4318/v1/traces
# Should return 405 Method Not Allowed (GET not supported, but port is open)

# Check collector logs
docker logs otlp-collector | tail -50

# Start collector if not running
docker run -d --name otlp-collector \
    -p 4318:4318 -p 4317:4317 \
    otel/opentelemetry-collector:latest
```

### Common Causes & Solutions

**Cause 1: Tracer Not Initialized**
```rust
// ❌ Wrong: No tracer initialization
fn main() {
    execute_query("ASK { ?s ?p ?o }");  // No telemetry!
}

// ✅ Correct: Initialize tracer
use knhk_otel::init_tracer;

fn main() {
    let _guard = init_tracer("knhk-cli", "1.0.0", Some("http://localhost:4318"))
        .expect("Init tracer");

    execute_query("ASK { ?s ?p ?o }");

    // Guard flushes telemetry on drop
}
```

**Cause 2: Wrong OTLP Endpoint**
```rust
// ❌ Wrong: Incorrect endpoint
let _guard = init_tracer("knhk", "1.0.0", Some("http://localhost:9090"));  // Wrong port!

// ✅ Correct: Default OTLP HTTP endpoint
let _guard = init_tracer("knhk", "1.0.0", Some("http://localhost:4318"));

// ✅ Correct: Default OTLP gRPC endpoint
let _guard = init_tracer("knhk", "1.0.0", Some("http://localhost:4317"));
```

**Cause 3: Telemetry Not Flushed**
```rust
// ❌ Wrong: Guard dropped before flush
fn execute_operation() {
    let _guard = init_tracer("knhk", "1.0.0", Some("http://localhost:4318"))
        .expect("Init");

    execute_query("ASK { ?s ?p ?o }");
    // Guard dropped here, but async export may not complete!
}

// ✅ Correct: Wait for flush
fn execute_operation() {
    let guard = init_tracer("knhk", "1.0.0", Some("http://localhost:4318"))
        .expect("Init");

    execute_query("ASK { ?s ?p ?o }");

    drop(guard);  // Explicit drop
    std::thread::sleep(std::time::Duration::from_millis(100));  // Wait for export
}

// ✅ Better: Use shutdown signal
fn main() {
    let guard = init_tracer("knhk", "1.0.0", Some("http://localhost:4318"))
        .expect("Init");

    // Application logic
    run_application();

    // Proper shutdown
    drop(guard);
    std::thread::sleep(std::time::Duration::from_secs(1));
}
```

### Prevention
- Always initialize tracer in main()
- Keep guard alive until shutdown
- Wait for telemetry flush before exit
- Test with OTLP collector in dev

---

## Issue 2: Spans Missing Attributes

### Symptom
```
Error: Span 'knhk.query.execute' missing required attribute 'query.type'
```

### Diagnosis

**Step 1: Check schema definition:**
```yaml
# registry/knhk-schema.yaml
groups:
  - id: knhk.query.execute
    type: span
    attributes:
      - id: query.type
        type: string
        requirement_level: required  # This attribute is required!
```

**Step 2: Check code emission:**
```rust
// Is attribute added?
let span_ctx = tracer.start_span("knhk.query.execute".to_string(), None);
// Missing attribute here!
tracer.end_span(span_ctx, SpanStatus::Ok);
```

### Solution

**Always emit required attributes:**
```rust
// ❌ Wrong: Missing required attribute
let span_ctx = tracer.start_span("knhk.query.execute".to_string(), None);
tracer.end_span(span_ctx, SpanStatus::Ok);  // Missing query.type!

// ✅ Correct: Emit all required attributes
let span_ctx = tracer.start_span("knhk.query.execute".to_string(), None);
tracer.add_attribute(
    span_ctx.clone(),
    "query.type".to_string(),
    "ASK".to_string(),  // Required attribute
);
tracer.end_span(span_ctx, SpanStatus::Ok);
```

**Use helper functions:**
```rust
// Helper to ensure required attributes
fn start_query_span(tracer: &mut Tracer, query_type: &str) -> SpanContext {
    let span_ctx = tracer.start_span("knhk.query.execute".to_string(), None);
    tracer.add_attribute(
        span_ctx.clone(),
        "query.type".to_string(),
        query_type.to_string(),
    );
    span_ctx
}

// Usage
let span_ctx = start_query_span(&mut tracer, "ASK");
// ... execute query ...
tracer.end_span(span_ctx, SpanStatus::Ok);
```

### Prevention
- Schema-first development (define attributes before code)
- Use helper functions for span creation
- Run Weaver live-check in tests
- Code generation from schema

---

## Issue 3: Attribute Type Mismatch

### Symptom
```
Error: Attribute 'query.latency_ns' has type mismatch
Schema defines: int
Runtime emits: string
```

### Diagnosis

**Check schema definition:**
```yaml
attributes:
  - id: query.latency_ns
    type: int  # Schema expects integer
```

**Check code emission:**
```rust
// What type is emitted?
tracer.add_attribute(
    span_ctx.clone(),
    "query.latency_ns".to_string(),
    latency.to_string(),  // String! Wrong type!
);
```

### Solution

**Emit correct type:**
```rust
// ❌ Wrong: String instead of int
let latency = 1234;
tracer.add_attribute(
    span_ctx.clone(),
    "query.latency_ns".to_string(),
    latency.to_string(),  // "1234" (string)
);

// ✅ Correct: Emit as int
use knhk_otel::AttributeValue;

let latency = 1234_i64;
tracer.add_attribute_int(
    span_ctx.clone(),
    "query.latency_ns".to_string(),
    latency,  // 1234 (int)
);
```

**Type-safe helpers:**
```rust
// Define typed helpers
impl Tracer {
    pub fn add_attribute_int(&mut self, span: SpanContext, key: String, value: i64) {
        // Serialize as int, not string
        self.add_attribute(span, key, value.to_string());  // FIXME: proper int serialization
    }

    pub fn add_attribute_bool(&mut self, span: SpanContext, key: String, value: bool) {
        self.add_attribute(span, key, value.to_string());
    }

    pub fn add_attribute_float(&mut self, span: SpanContext, key: String, value: f64) {
        self.add_attribute(span, key, value.to_string());
    }
}
```

### Prevention
- Use typed attribute helpers
- Don't call `.to_string()` on numeric types
- Validate types in tests
- Use code generation from schema

---

## Issue 4: Context Not Propagated (Broken Traces)

### Symptom
```
Warning: Span 'knhk.workflow.step' has no parent
Expected parent: 'knhk.workflow.execute'
```

### Diagnosis

**Step 1: Check if parent context passed:**
```rust
async fn execute_workflow() {
    let span_ctx = tracer.start_span("knhk.workflow.execute".to_string(), None);

    // Is context passed to child?
    execute_workflow_step().await;  // No context passed!

    tracer.end_span(span_ctx, SpanStatus::Ok);
}

async fn execute_workflow_step() {
    // No parent context available here!
    let span_ctx = tracer.start_span("knhk.workflow.step".to_string(), None);
    // ...
}
```

### Solution

**Pass context explicitly:**
```rust
// ❌ Wrong: No context propagation
async fn execute_workflow() {
    let span_ctx = tracer.start_span("knhk.workflow.execute".to_string(), None);
    execute_workflow_step().await;  // No context!
    tracer.end_span(span_ctx, SpanStatus::Ok);
}

async fn execute_workflow_step() {
    let span_ctx = tracer.start_span("knhk.workflow.step".to_string(), None);  // Orphan span!
    // ...
}

// ✅ Correct: Explicit context propagation
async fn execute_workflow(tracer: &mut Tracer) {
    let parent_ctx = tracer.start_span("knhk.workflow.execute".to_string(), None);
    execute_workflow_step(tracer, &parent_ctx).await;  // Pass context!
    tracer.end_span(parent_ctx, SpanStatus::Ok);
}

async fn execute_workflow_step(tracer: &mut Tracer, parent: &SpanContext) {
    let span_ctx = tracer.start_span(
        "knhk.workflow.step".to_string(),
        Some(parent.clone()),  // Link to parent!
    );
    // ...
    tracer.end_span(span_ctx, SpanStatus::Ok);
}
```

**Use `tracing` crate for automatic propagation:**
```rust
use tracing::{instrument, info};

#[instrument(name = "knhk.workflow.execute")]
async fn execute_workflow() {
    info!("Executing workflow");
    execute_workflow_step().await;  // Context auto-propagated!
}

#[instrument(name = "knhk.workflow.step")]
async fn execute_workflow_step() {
    info!("Executing step");
    // Automatically linked to parent span!
}
```

### Prevention
- Pass `SpanContext` explicitly in function signatures
- Use `tracing::instrument` for automatic propagation
- Test distributed tracing in integration tests
- Validate parent-child relationships with Weaver

---

## Issue 5: High Telemetry Overhead (>5% CPU)

### Symptom
```bash
# Without telemetry
$ perf stat cargo run
Performance counter stats for 'cargo run':
    2.5s  time elapsed
    2.4s  user
    0.1s  system

# With telemetry
$ perf stat cargo run
Performance counter stats for 'cargo run':
    3.8s  time elapsed  # 52% slower!
    3.6s  user
    0.2s  system
```

### Diagnosis

**Step 1: Measure telemetry overhead:**
```rust
// Benchmark without telemetry
fn bench_no_telemetry() {
    for _ in 0..10000 {
        execute_query("ASK { ?s ?p ?o }");
    }
}

// Benchmark with telemetry
fn bench_with_telemetry() {
    let _guard = init_tracer("bench", "1.0.0", Some("http://localhost:4318")).unwrap();

    for _ in 0..10000 {
        let mut tracer = Tracer::new();
        let span = tracer.start_span("query".to_string(), None);
        execute_query("ASK { ?s ?p ?o }");
        tracer.end_span(span, SpanStatus::Ok);
    }
}
```

**Step 2: Profile telemetry code:**
```bash
perf record -g cargo run
perf report --stdio | grep -E '(otel|tracing|span)'
```

### Common Causes & Solutions

**Cause 1: Telemetry in Hot Path**
```rust
// ❌ Wrong: Span in hot path (≤8 ticks)
fn execute_hot_path_ask() -> bool {
    let mut tracer = Tracer::new();  // ~100 ticks!
    let span = tracer.start_span("hot.ask".to_string(), None);  // ~500 ticks!

    // Hot path logic (8 ticks)
    let result = check_triple_exists();

    tracer.end_span(span, SpanStatus::Ok);  // ~500 ticks!
    result
}

// ✅ Correct: No telemetry in hot path
fn execute_hot_path_ask() -> bool {
    // Hot path logic only (8 ticks)
    check_triple_exists()
}

// Add telemetry outside hot path
fn execute_query_with_telemetry(query: &str) -> bool {
    let mut tracer = Tracer::new();
    let span = tracer.start_span("query.execute".to_string(), None);

    let result = execute_hot_path_ask();  // Hot path, no telemetry

    tracer.end_span(span, SpanStatus::Ok);
    result
}
```

**Cause 2: Too Many Spans**
```rust
// ❌ Wrong: Span for every small operation
fn process_items(items: &[Item]) {
    for item in items {
        let span = tracer.start_span("process.item".to_string(), None);  // 1000 spans!
        process_item(item);
        tracer.end_span(span, SpanStatus::Ok);
    }
}

// ✅ Correct: One span for batch
fn process_items(items: &[Item]) {
    let span = tracer.start_span("process.batch".to_string(), None);  // 1 span
    tracer.add_attribute(span.clone(), "batch.size".to_string(), items.len().to_string());

    for item in items {
        process_item(item);  // No telemetry
    }

    tracer.end_span(span, SpanStatus::Ok);
}
```

**Cause 3: Synchronous Export**
```rust
// ❌ Wrong: Blocking export
impl Tracer {
    pub fn end_span(&mut self, span: SpanContext, status: SpanStatus) {
        // ... build span ...

        // Synchronous HTTP POST (blocks ~10ms)
        reqwest::blocking::post("http://localhost:4318/v1/traces")
            .json(&spans)
            .send()
            .unwrap();
    }
}

// ✅ Correct: Async batch export
impl Tracer {
    pub fn end_span(&mut self, span: SpanContext, status: SpanStatus) {
        // ... build span ...

        // Add to batch (no blocking)
        self.span_batch.push(span);

        // Export asynchronously every 1000 spans or 10s
        if self.span_batch.len() >= 1000 {
            self.flush_async();
        }
    }
}
```

**Cause 4: Expensive Attribute Serialization**
```rust
// ❌ Wrong: Serialize large objects
let span = tracer.start_span("query".to_string(), None);
tracer.add_attribute(
    span.clone(),
    "query.full_result".to_string(),
    format!("{:?}", large_result),  // Serialize 10MB object!
);

// ✅ Correct: Only essential attributes
let span = tracer.start_span("query".to_string(), None);
tracer.add_attribute(
    span.clone(),
    "query.result_size".to_string(),
    large_result.len().to_string(),  // Just the size
);
```

### Prevention
- Never instrument hot path (≤8 ticks)
- Batch span exports (async, non-blocking)
- Limit span attributes (only essential data)
- Sample traces (e.g., 10% in production)
- Benchmark with/without telemetry

---

## Issue 6: Spans Not Visible in Jaeger/Zipkin

### Symptom
```bash
# Application running and emitting telemetry
$ cargo run --bin knhk-cli -- query ask "ASK { ?s ?p ?o }"

# But no traces in Jaeger UI
$ open http://localhost:16686
# No traces for "knhk-cli" service
```

### Diagnosis

**Step 1: Verify OTLP collector receiving data:**
```bash
# Check collector logs
docker logs otlp-collector | grep -i traces

# Should see:
# Trace received: service=knhk-cli, spans=3
```

**Step 2: Verify collector exporting to Jaeger:**
```yaml
# otel-collector-config.yaml
exporters:
  jaeger:
    endpoint: "jaeger:14250"  # Is this correct?
    tls:
      insecure: true

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [jaeger]  # Is Jaeger exporter configured?
```

**Step 3: Verify Jaeger receiving data:**
```bash
# Check Jaeger logs
docker logs jaeger | grep -i received
```

### Solution

**Configure OTLP collector to export to Jaeger:**
```yaml
# otel-collector-config.yaml
receivers:
  otlp:
    protocols:
      http:
        endpoint: "0.0.0.0:4318"
      grpc:
        endpoint: "0.0.0.0:4317"

exporters:
  jaeger:
    endpoint: "jaeger:14250"
    tls:
      insecure: true

  logging:
    loglevel: debug  # Debug export issues

processors:
  batch:
    timeout: 10s
    send_batch_size: 1024

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [jaeger, logging]  # Log and export to Jaeger
```

**Docker Compose setup:**
```yaml
version: '3'
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # Jaeger UI
      - "14250:14250"  # Jaeger gRPC

  otel-collector:
    image: otel/opentelemetry-collector:latest
    volumes:
      - ./otel-collector-config.yaml:/etc/otel/config.yaml
    command: ["--config=/etc/otel/config.yaml"]
    ports:
      - "4318:4318"  # OTLP HTTP
      - "4317:4317"  # OTLP gRPC
    depends_on:
      - jaeger
```

### Prevention
- Test telemetry pipeline end-to-end
- Use logging exporter for debugging
- Verify collector configuration
- Check Jaeger service name matches application

---

## Quick Telemetry Diagnostic Commands

```bash
# 1. Verify schema valid
weaver registry check -r /home/user/knhk/registry/

# 2. Start OTLP collector
docker run -d --name otlp-collector -p 4318:4318 otel/opentelemetry-collector:latest

# 3. Run application with telemetry
RUST_LOG=debug cargo run --bin knhk-cli -- query ask "ASK { ?s ?p ?o }"

# 4. Check collector received data
curl http://localhost:4318/v1/traces  # Should return 405 (port open)
docker logs otlp-collector | grep -i trace

# 5. Validate runtime telemetry
weaver registry live-check --registry /home/user/knhk/registry/

# 6. Measure telemetry overhead
cargo bench --bench telemetry_overhead

# 7. View traces in Jaeger
open http://localhost:16686
```

---

## See Also

- [Telemetry Checklist](/home/user/knhk/docs/reference/cards/TELEMETRY_CHECKLIST.md)
- [Weaver Validation Troubleshooting](/home/user/knhk/docs/troubleshooting/WEAVER_VALIDATION_TROUBLESHOOTING.md)
- [Production Readiness Checklist](/home/user/knhk/docs/reference/cards/PRODUCTION_READINESS_CHECKLIST.md)
