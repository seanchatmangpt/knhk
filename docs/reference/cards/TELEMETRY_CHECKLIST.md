# Telemetry Implementation Checklist

**Purpose**: One-page verification for proper OpenTelemetry instrumentation
**Target**: Schema-validated, production-ready telemetry
**Validation**: Weaver live-check is the source of truth

---

## 1. Schema Definition (MANDATORY)

**All telemetry must be declared in OTEL schema first:**

- [ ] **Schema file created in `/home/user/knhk/registry/`**
- [ ] **All spans defined with attributes**
- [ ] **All metrics defined with units and types**
- [ ] **All logs defined with severity levels**
- [ ] **`weaver registry check -r /home/user/knhk/registry/` passes**

**Pass Criteria**: All 5 items checked ✅ - No code before schema!

---

## 2. Span Instrumentation

**Strategic span placement for distributed tracing:**

- [ ] Spans created at service boundaries (HTTP requests, gRPC calls)
- [ ] Spans created for significant operations (query execution, workflow steps)
- [ ] Span names follow convention: `knhk.operation.action` (e.g., `knhk.query.execute`)
- [ ] Span attributes include operation-specific context (query type, workflow ID)
- [ ] Span status set correctly (Ok, Error) with error messages
- [ ] Span IDs correlate with receipts for provenance
- [ ] Parent-child relationships maintained for nested operations
- [ ] No spans in hot path critical sections (≤8 ticks)

**Pass Criteria**: All 8 items checked ✅

**Example**:
```rust
use knhk_otel::{Tracer, SpanStatus};

let mut tracer = Tracer::new();
let span_ctx = tracer.start_span("knhk.query.execute".to_string(), None);
tracer.add_attribute(span_ctx.clone(), "query.type".to_string(), "ASK".to_string());
// ... execute operation ...
tracer.end_span(span_ctx, SpanStatus::Ok);
```

---

## 3. Metrics Collection

**Quantitative measurements for monitoring:**

- [ ] Counter metrics for events (requests processed, errors encountered)
- [ ] Histogram metrics for distributions (query latency, workflow duration)
- [ ] Gauge metrics for current state (active connections, queue depth)
- [ ] Metric names follow convention: `knhk.operation.metric_name` (e.g., `knhk.query.latency`)
- [ ] Metric units specified (seconds, bytes, count)
- [ ] Metrics aggregated appropriately (sum, average, p99)
- [ ] Metrics include labels for dimensions (operation type, status)
- [ ] Metrics overhead measured (≤5% CPU)

**Pass Criteria**: All 8 items checked ✅

**Example**:
```rust
use knhk_otel::{MetricsHelper, Tracer};

let mut tracer = Tracer::new();
MetricsHelper::record_latency(&mut tracer, "query.execute", 1.2); // 1.2 ns
MetricsHelper::record_operation(&mut tracer, "query.execute", true); // success
```

---

## 4. Logging Strategy

**Structured logging for debugging and auditing:**

- [ ] Logs use `tracing::info!`, `tracing::warn!`, `tracing::error!` (not `println!`)
- [ ] Log levels set appropriately (DEBUG, INFO, WARN, ERROR)
- [ ] Structured fields included (operation, duration, status)
- [ ] Errors logged with stack traces and context
- [ ] Sensitive data never logged (credentials, PII)
- [ ] Log volume tested (not overwhelming in production)
- [ ] Logs correlated with spans (trace ID, span ID)
- [ ] Log sampling configured for high-volume paths

**Pass Criteria**: All 8 items checked ✅

**Example**:
```rust
use tracing::{info, error};

info!(query_type = "ASK", duration_ns = 1.2, "Query executed successfully");
error!(query_type = "ASK", error = %err, "Query execution failed");
```

---

## 5. Context Propagation

**Distributed tracing across service boundaries:**

- [ ] W3C Trace Context headers propagated (traceparent, tracestate)
- [ ] Span context extracted from incoming requests
- [ ] Span context injected into outgoing requests
- [ ] Baggage propagated for cross-cutting concerns
- [ ] Context stored in thread-local storage where needed
- [ ] Context passed explicitly in async functions
- [ ] Context maintained across workflow steps
- [ ] Context cleared appropriately to prevent leaks

**Pass Criteria**: All 8 items checked ✅

---

## 6. Sampling Configuration

**Control telemetry volume for production:**

- [ ] Trace sampling configured (e.g., 10% of requests)
- [ ] Error traces always sampled (100% of errors)
- [ ] High-value operations always sampled (critical paths)
- [ ] Sampling decision propagated to child spans
- [ ] Sampling rate adjustable via configuration
- [ ] Sampling overhead measured (≤1% CPU)
- [ ] Sampling decisions logged for debugging
- [ ] Head-based sampling for simplicity (tail-based if needed)

**Pass Criteria**: All 8 items checked ✅

---

## 7. OTLP Exporter Setup

**Production-ready telemetry export:**

- [ ] OTLP exporter configured (HTTP or gRPC)
- [ ] Collector endpoint specified (e.g., `http://localhost:4318`)
- [ ] Batching configured (batch size, timeout)
- [ ] Retry logic implemented (exponential backoff)
- [ ] TLS configured for production
- [ ] Resource attributes set (service name, version, environment)
- [ ] Exporter health monitored (connection status, export errors)
- [ ] Fallback to stdout exporter if collector unavailable

**Pass Criteria**: All 8 items checked ✅

**Example**:
```rust
use knhk_otel::init_tracer;

let guard = init_tracer("knhk-service", "1.0.0", Some("http://localhost:4318"))
    .expect("Failed to initialize tracer");
```

---

## 8. Performance Budget

**Telemetry must not degrade performance:**

- [ ] Telemetry overhead ≤5% CPU measured
- [ ] Telemetry overhead ≤10MB memory measured
- [ ] No telemetry in hot path critical sections (≤8 ticks)
- [ ] Telemetry operations asynchronous where possible
- [ ] Batch exports to reduce network overhead
- [ ] No blocking I/O on critical path
- [ ] Telemetry disabled in development if needed (via feature flags)
- [ ] Performance regression tests include telemetry overhead

**Pass Criteria**: All 8 items checked ✅

---

## 9. Weaver Validation (MANDATORY)

**Schema compliance is the source of truth:**

- [ ] **`weaver registry check -r /home/user/knhk/registry/` passes** (schema valid)
- [ ] **`weaver registry live-check --registry /home/user/knhk/registry/` passes** (runtime matches schema)
- [ ] All spans emitted match schema definitions
- [ ] All metrics emitted match schema definitions
- [ ] All logs emitted match schema definitions
- [ ] Schema version controlled with code
- [ ] Schema changes reviewed in PRs
- [ ] Schema documentation generated

**Pass Criteria**: All 8 items checked ✅ - First 2 are MANDATORY

---

## Quick Validation Commands

```bash
# 1. Validate schema definition
weaver registry check -r /home/user/knhk/registry/

# 2. Run application with telemetry
cargo run --bin knhk-cli -- query ask "ASK { ?s ?p ?o }"

# 3. Validate runtime telemetry matches schema
weaver registry live-check --registry /home/user/knhk/registry/

# 4. Check telemetry in collector logs
docker logs otlp-collector | grep knhk

# 5. Measure telemetry overhead
cargo bench --bench telemetry_overhead
```

---

## Telemetry Instrumentation Pyramid

**Strategic instrumentation (few → many):**

```
         Spans (few)
        /           \
       /    Metrics   \
      /    (moderate)  \
     /                  \
    /     Logs (many)    \
   /_______________________\
```

- **Spans**: Service boundaries, significant operations (10-20 per request)
- **Metrics**: Quantitative measurements (50-100 metrics per service)
- **Logs**: Detailed debugging info (100-1000 logs per request)

---

## Final Sign-Off

- [ ] **All 9 sections completed** (72 total checks)
- [ ] **Weaver validation passes** (MANDATORY)
- [ ] **Schema defined before code** (schema-first approach)
- [ ] **Performance budget met** (≤5% overhead)
- [ ] **Production collector configured**
- [ ] **Sampling configured**

**Telemetry Approved By**: ________________
**Date**: ________________

---

**See Also**:
- [Production Readiness Checklist](/home/user/knhk/docs/reference/cards/PRODUCTION_READINESS_CHECKLIST.md)
- [OTEL Integration Guide](/home/user/knhk/docs/INTEGRATION.md)
- [Weaver Documentation](https://github.com/open-telemetry/weaver)
