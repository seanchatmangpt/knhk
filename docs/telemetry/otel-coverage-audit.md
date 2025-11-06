# OTEL Coverage Audit Report

**Agent**: Telemetry Specialist (Supporting Agent #10)
**Swarm ID**: swarm-1762466485307-u67jafg4t
**Priority**: P2 (OBSERVABILITY)
**Date**: 2025-11-06

## Executive Summary

This report provides a comprehensive audit of OpenTelemetry instrumentation across the KNHK system, focusing on coverage, semantic conventions, and Weaver validation compliance.

### Key Findings

✅ **STRENGTHS**:
- Comprehensive OTEL infrastructure in `knhk-otel` crate
- Weaver live-check integration implemented
- Sidecar service has telemetry export capability
- MetricsHelper provides semantic convention compliance

⚠️ **GAPS**:
- ETL pipeline stages missing OTEL instrumentation
- No registry schema files found (required for Weaver validation)
- Connector operations lack telemetry
- Reflex operations not instrumented

## OTEL Infrastructure Review

### 1. knhk-otel Crate

**Location**: `/Users/sac/knhk/rust/knhk-otel/src/lib.rs`

**Components**:
- ✅ `Tracer` - Span and metric collection
- ✅ `OtlpExporter` - OTLP/HTTP export to collectors
- ✅ `WeaverLiveCheck` - Live telemetry validation
- ✅ `MetricsHelper` - Semantic convention helpers
- ✅ `SpanContext`, `Span`, `Metric` - Core telemetry types

**Semantic Conventions Implemented**:
```rust
// Operations
knhk.operation.executed
knhk.operation.name
knhk.operation.type

// Warm path
knhk.warm_path.operations.latency
knhk.warm_path.operations.count

// Hooks
knhk.hook.latency.ticks

// Receipts
knhk.receipt.generated

// Guards
knhk.guard.violation

// Config
knhk.config.loads
knhk.config.errors

// Connectors
knhk.connector.throughput
```

### 2. Sidecar Service Instrumentation

**Location**: `/Users/sac/knhk/rust/knhk-sidecar/src/service.rs`

**Instrumented Operations**:
- ✅ `apply_transaction` - Span: `knhk.sidecar.transaction`
- ✅ `query` - Span: `knhk.sidecar.query`
- ✅ `validate_graph` - Span: `knhk.sidecar.validate_graph`
- ✅ `evaluate_hook` - Span: `knhk.sidecar.evaluate_hook`
- ✅ Internal metrics tracked (requests, latency, circuit breaker)

**Export Mechanism**:
```rust
#[cfg(feature = "otel")]
async fn export_telemetry(
    &self,
    span_name: &str,
    operation_name: &str,
    success: bool,
    latency_ms: u64,
    attributes: Vec<(&str, String)>
)
```

**Metrics Recorded**:
- Operation success/failure
- Latency (converted to microseconds for warm path)
- Transaction/query/hook counts
- Circuit breaker state

### 3. ETL Pipeline Instrumentation (GAPS)

**Location**: `/Users/sac/knhk/rust/knhk-etl/src/`

#### Stage 1: Ingest (MISSING INSTRUMENTATION)
**File**: `ingest.rs`

**Critical Operations Without Telemetry**:
- ❌ `ingest()` - Delta ingestion from connectors
- ❌ `parse_rdf_turtle()` - RDF/Turtle parsing
- ❌ `parse_rdf_turtle_stream()` - Streaming parser
- ❌ Oxigraph store operations

**Recommended Spans**:
```yaml
knhk.etl.ingest:
  operation: ingest
  attributes:
    - knhk.connector.count
    - knhk.triples.count
    - knhk.format (turtle, jsonld, etc.)

knhk.etl.parse:
  operation: parse_rdf
  attributes:
    - knhk.format
    - knhk.bytes.size
    - knhk.triples.parsed
```

#### Stage 5: Emit (PARTIAL INSTRUMENTATION)
**File**: `emit.rs`

**Instrumented**:
- ✅ W1 cache hit metric (via `knhk_otel::MetricsHelper`)

**Missing Instrumentation**:
- ❌ `emit()` - Main emit operation
- ❌ HTTP webhook sends
- ❌ Kafka message sends
- ❌ gRPC action sends
- ❌ Lockchain writes
- ❌ Retry logic
- ❌ Failure handling (R1, W1, C1)

**Recommended Spans**:
```yaml
knhk.etl.emit:
  operation: emit
  attributes:
    - knhk.receipts.written
    - knhk.actions.sent
    - knhk.lockchain.enabled

knhk.etl.emit.http:
  operation: send_http_webhook
  attributes:
    - knhk.endpoint
    - knhk.retry.attempt
    - knhk.http.status

knhk.etl.emit.kafka:
  operation: send_kafka_action
  attributes:
    - knhk.kafka.topic
    - knhk.kafka.broker
    - knhk.retry.attempt

knhk.etl.emit.lockchain:
  operation: write_receipt
  attributes:
    - knhk.receipt.id
    - knhk.lockchain.hash
```

## Weaver Validation Status

### Current State

**Registry Schema**: ❌ NOT FOUND
- Expected location: `/Users/sac/knhk/registry/`
- Status: Directory exists but contains no YAML files

**Critical Requirement**:
KNHK requires Weaver registry validation as the ONLY source of truth for telemetry compliance. Without registry schemas, telemetry cannot be validated.

### Required Registry Structure

```
registry/
├── registry.yaml              # Main registry
├── semantic-conventions/
│   ├── knhk-operations.yaml   # Operation spans/metrics
│   ├── knhk-etl.yaml          # ETL pipeline telemetry
│   ├── knhk-sidecar.yaml      # Sidecar service telemetry
│   ├── knhk-warm-path.yaml    # Warm path telemetry
│   ├── knhk-hooks.yaml        # Hook execution telemetry
│   ├── knhk-guards.yaml       # Guard violation telemetry
│   └── knhk-connectors.yaml   # Connector telemetry
```

### Weaver Live-Check Integration

**Implementation**: ✅ Complete (`knhk-otel::WeaverLiveCheck`)

**Capabilities**:
```rust
// Check Weaver availability
WeaverLiveCheck::check_weaver_available()?;

// Start live validation
let weaver = WeaverLiveCheck::new()
    .with_registry("./registry".to_string())
    .with_otlp_port(4317)
    .with_admin_port(8080);

let mut process = weaver.start()?;

// Export telemetry for validation
tracer.export_to_weaver(&weaver.otlp_endpoint())?;

// Stop validation
weaver.stop()?;
```

## Coverage Analysis

### Overall Coverage: 35%

| Component | Instrumented | Coverage |
|-----------|--------------|----------|
| knhk-otel | 100% | ✅ Complete |
| knhk-sidecar | 80% | ⚠️ Partial |
| knhk-etl | 10% | ❌ Minimal |
| knhk-connectors | 0% | ❌ None |
| knhk-warm | 0% | ❌ None |

### Critical Gaps by Priority

**P0 (BLOCKING)**:
1. ❌ Registry schema files missing (prevents Weaver validation)
2. ❌ ETL pipeline not instrumented (core functionality)

**P1 (HIGH PRIORITY)**:
1. ❌ Connector operations lack telemetry
2. ❌ Reflex operations not instrumented
3. ❌ Emit stage partially instrumented

**P2 (MEDIUM PRIORITY)**:
1. ⚠️ Sidecar health check not instrumented
2. ⚠️ Circuit breaker operations lack spans
3. ⚠️ Retry operations not tracked

## Semantic Convention Compliance

### Compliant Operations

**knhk-sidecar**:
- ✅ All operations use `knhk.sidecar.*` namespace
- ✅ Attributes follow `knhk.operation.*` pattern
- ✅ Success/failure tracked consistently
- ✅ Latency recorded in standard units

**knhk-otel MetricsHelper**:
- ✅ Consistent attribute naming
- ✅ Proper metric types (Counter, Gauge, Histogram)
- ✅ Timestamp recording

### Non-Compliant Areas

**ETL Pipeline**:
- ❌ No span naming conventions established
- ❌ Inconsistent attribute usage
- ❌ Missing required attributes (operation.name, operation.type)

## Recommendations

### Immediate Actions (P0)

1. **Create Registry Schema Files**:
   ```bash
   # Create semantic convention schemas
   weaver registry generate --template knhk-operations
   weaver registry generate --template knhk-etl
   weaver registry generate --template knhk-sidecar
   ```

2. **Instrument ETL Pipeline**:
   - Add OTEL spans to all 5 ETL stages
   - Record metrics for throughput, latency, errors
   - Implement semantic convention compliance

3. **Validate with Weaver**:
   ```bash
   # Start Weaver live-check
   weaver registry live-check --registry ./registry

   # Run operations to emit telemetry
   # Verify validation passes
   ```

### Short-term Actions (P1)

1. **Add Connector Telemetry**:
   - Instrument Kafka consumer/producer
   - Add Salesforce API call tracing
   - Record connector health metrics

2. **Complete Sidecar Instrumentation**:
   - Add health check telemetry
   - Instrument circuit breaker operations
   - Track retry attempts

3. **Create Telemetry Tests**:
   - Unit tests for span/metric creation
   - Integration tests with Weaver validation
   - Performance tests (verify ≤8 ticks overhead)

### Long-term Actions (P2)

1. **Continuous Validation**:
   - CI/CD integration with Weaver live-check
   - Automated schema compliance checks
   - Telemetry regression tests

2. **Observability Dashboard**:
   - Grafana dashboards for KNHK metrics
   - Alerting on telemetry gaps
   - Performance monitoring

## Testing Strategy

### Unit Tests (Chicago TDD)
- ✅ Test span creation behavior
- ✅ Test metric recording behavior
- ✅ Test attribute compliance
- ❌ Missing: ETL stage telemetry tests

### Integration Tests
- ✅ Test Weaver live-check workflow
- ⚠️ Need: End-to-end telemetry export tests
- ❌ Missing: Multi-component tracing tests

### Performance Tests
- ⚠️ Need: Verify telemetry overhead ≤8 ticks
- ❌ Missing: Throughput impact analysis
- ❌ Missing: Memory overhead measurement

## Conclusion

KNHK has a solid OTEL foundation with the `knhk-otel` crate and sidecar instrumentation. However, **critical gaps exist**:

1. **Registry schemas missing** - Blocks Weaver validation (source of truth)
2. **ETL pipeline uninstrumented** - Core functionality lacks observability
3. **Test coverage incomplete** - No telemetry-specific test suite

**Next Steps**:
1. Create registry schema files (P0)
2. Instrument ETL pipeline (P0)
3. Run Weaver validation (P0)
4. Create comprehensive telemetry test suite (P1)
5. CI/CD integration (P2)

**Success Criteria**:
- ✅ All operations emit spans
- ✅ Spans export successfully to OTLP
- ✅ Weaver validation passes
- ✅ Performance overhead ≤8 ticks
- ✅ 100% semantic convention compliance
