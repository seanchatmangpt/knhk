# Weaver Live-Check Validation Report

**Agent**: Production Validator (Critical Path Agent #3)
**Swarm ID**: swarm-1762466485307-u67jafg4t
**Date**: 2025-11-06
**Status**: 🔴 **BLOCKED - Missing Semantic Convention Schemas**

## Executive Summary

Weaver validation is **BLOCKED** because the `registry/` directory contains no semantic convention schema files. While the sidecar has full OTEL instrumentation capability via `knhk-otel`, we cannot validate telemetry conformance without schema definitions.

**CRITICAL**: This is KNHK's source of truth for validation. Tests can pass with false positives, but Weaver validation proves actual runtime behavior matches declared schemas.

## Current State Analysis

### ✅ What Exists

1. **Weaver Binary Installed**
   ```bash
   $ which weaver
   /Users/sac/.cargo/bin/weaver
   ```

2. **OTEL Infrastructure Complete**
   - `knhk-otel` crate with full OTLP export capability
   - `OtlpExporter` with span/metric export to OTLP endpoints
   - `WeaverLiveCheck` wrapper for starting validation
   - `Tracer` with span lifecycle management

3. **Sidecar OTEL Integration**
   - Service exports telemetry via `export_telemetry()` method
   - Spans emitted for all operations: `apply_transaction`, `query`, `validate_graph`, `evaluate_hook`
   - Metrics recorded via `MetricsHelper`
   - Feature flag `otel` enabled by default

4. **Registry Directory Structure**
   ```
   registry/
   └── README.md  (documentation only, no schemas)
   ```

### ❌ What's Missing (BLOCKERS)

1. **No Semantic Convention Schemas**
   - No `.yaml` files defining telemetry conventions
   - Cannot run `weaver registry check`
   - Cannot validate schema definitions

2. **No Live Telemetry Validation**
   - Cannot run `weaver live-check` without schemas
   - No proof that runtime telemetry matches declarations
   - No validation of span attributes, metrics, or naming

## OTEL Telemetry Architecture

### Sidecar Telemetry Emission (from service.rs)

```rust
// Lines 115-141: export_telemetry method
async fn export_telemetry(
    &self,
    span_name: &str,
    operation_name: &str,
    success: bool,
    latency_ms: u64,
    attributes: Vec<(&str, String)>
) {
    if let Some(ref endpoint) = self.weaver_endpoint {
        use knhk_otel::{Tracer, SpanStatus};
        let mut tracer = knhk_otel::Tracer::with_otlp_exporter(endpoint.clone());
        let span_ctx = tracer.start_span(span_name.to_string(), None);

        tracer.add_attribute(span_ctx.clone(), "knhk.operation.name", operation_name);
        tracer.add_attribute(span_ctx.clone(), "knhk.operation.type", "sidecar");
        tracer.add_attribute(span_ctx.clone(), "knhk.sidecar.success", success);
        tracer.add_attribute(span_ctx.clone(), "knhk.sidecar.latency_ms", latency_ms);

        for (key, value) in attributes {
            tracer.add_attribute(span_ctx.clone(), format!("knhk.sidecar.{}", key), value);
        }

        tracer.end_span(span_ctx, if success { SpanStatus::Ok } else { SpanStatus::Error });

        // Record metrics
        use knhk_otel::MetricsHelper;
        MetricsHelper::record_operation(&mut tracer, operation_name, success);
        MetricsHelper::record_warm_path_latency(&mut tracer, latency_ms * 1000, operation_name);

        if let Err(e) = tracer.export() {
            warn!(error = %e, "Failed to export telemetry to Weaver");
        }
    }
}
```

### Declared Telemetry Conventions (from registry/README.md)

**Spans:**
- `knhk.sidecar.start` - Sidecar server startup
- `knhk.sidecar.request` - gRPC request handling
- `knhk.sidecar.batch` - Request batching operations
- `knhk.sidecar.retry` - Retry operations
- `knhk.sidecar.circuit_breaker` - Circuit breaker state changes

**Metrics:**
- `knhk.sidecar.requests.total` - Total requests
- `knhk.sidecar.requests.success` - Successful requests
- `knhk.sidecar.requests.failure` - Failed requests
- `knhk.sidecar.latency.p50_ms` - P50 latency
- `knhk.sidecar.latency.p95_ms` - P95 latency
- `knhk.sidecar.latency.p99_ms` - P99 latency
- `knhk.sidecar.batch.size` - Batch sizes
- `knhk.sidecar.retry.count` - Retry counts

**Attributes:**
- `knhk.operation.name` - Operation name
- `knhk.operation.type` - Operation type
- `knhk.sidecar.address` - Sidecar bind address
- `knhk.sidecar.method` - gRPC method name
- `knhk.sidecar.batch.size` - Batch size
- `knhk.sidecar.retry.attempt` - Retry attempt number

## Required Schema Files

To enable Weaver validation, we need to create semantic convention schema files in `registry/`:

### 1. `registry/knhk-sidecar-spans.yaml`

```yaml
groups:
  - id: knhk.sidecar
    type: span
    brief: "KNHK Sidecar telemetry spans"
    attributes:
      - id: knhk.operation.name
        type: string
        brief: "Name of the operation"
        examples: ["apply_transaction", "query", "validate_graph"]
      - id: knhk.operation.type
        type: string
        brief: "Type of operation"
        examples: ["sidecar", "system"]
      - id: knhk.sidecar.success
        type: boolean
        brief: "Whether operation succeeded"
      - id: knhk.sidecar.latency_ms
        type: int
        brief: "Operation latency in milliseconds"
    spans:
      - id: knhk.sidecar.start
        brief: "Sidecar server startup"
        attributes:
          - ref: knhk.sidecar.address
      - id: knhk.sidecar.request
        brief: "gRPC request handling"
        attributes:
          - ref: knhk.sidecar.method
      - id: knhk.sidecar.batch
        brief: "Batch operations"
        attributes:
          - ref: knhk.sidecar.batch.size
      - id: knhk.sidecar.retry
        brief: "Retry operations"
        attributes:
          - ref: knhk.sidecar.retry.attempt
      - id: knhk.sidecar.circuit_breaker
        brief: "Circuit breaker state changes"
```

### 2. `registry/knhk-sidecar-metrics.yaml`

```yaml
groups:
  - id: knhk.sidecar.metrics
    type: metric
    brief: "KNHK Sidecar telemetry metrics"
    metrics:
      - id: knhk.sidecar.requests.total
        type: counter
        brief: "Total number of requests"
        instrument: counter
        unit: "{request}"
      - id: knhk.sidecar.requests.success
        type: counter
        brief: "Number of successful requests"
        instrument: counter
        unit: "{request}"
      - id: knhk.sidecar.requests.failure
        type: counter
        brief: "Number of failed requests"
        instrument: counter
        unit: "{request}"
      - id: knhk.sidecar.latency.p50_ms
        type: gauge
        brief: "P50 latency"
        instrument: gauge
        unit: "ms"
      - id: knhk.sidecar.latency.p95_ms
        type: gauge
        brief: "P95 latency"
        instrument: gauge
        unit: "ms"
      - id: knhk.sidecar.latency.p99_ms
        type: gauge
        brief: "P99 latency"
        instrument: gauge
        unit: "ms"
```

## Validation Workflow (Once Unblocked)

### Phase 1: Schema Validation

```bash
# Validate schema definitions
weaver registry check -r registry/

# Expected output:
# ✅ All schemas valid
# ✅ No naming conflicts
# ✅ All references resolved
```

### Phase 2: Sidecar Startup with OTEL

```bash
cd rust/knhk-sidecar

# Configure OTEL export
export KGC_SIDECAR_WEAVER_ENABLED=true
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export RUST_LOG=info

# Start sidecar with OTEL feature
cargo run --features otel --bin knhk-sidecar
```

### Phase 3: Weaver Live-Check Validation

```bash
# In separate terminal, start Weaver live-check
weaver registry live-check \
  --registry registry/ \
  --otlp-grpc-address 127.0.0.1 \
  --otlp-grpc-port 4317 \
  --admin-port 8080 \
  --format json \
  --output ./weaver-reports

# Expected output:
# 🔄 Listening for OTLP telemetry on 127.0.0.1:4317
# 📊 Admin endpoint: http://127.0.0.1:8080
# ✅ Validating against registry/
```

### Phase 4: Test Scenarios

```bash
# Call gRPC methods to generate telemetry
grpcurl -plaintext -d '{"rdf_data": "..."}' localhost:50051 \
  kgc.sidecar.v1.KgcSidecar/ApplyTransaction

grpcurl -plaintext -d '{"query_type": "..."}' localhost:50051 \
  kgc.sidecar.v1.KgcSidecar/Query

# Weaver should capture and validate telemetry in real-time
```

### Phase 5: Validation Results

Expected Weaver output:
```json
{
  "validation_results": {
    "total_spans": 10,
    "valid_spans": 10,
    "invalid_spans": 0,
    "violations": [],
    "summary": {
      "knhk.sidecar.request": {
        "count": 5,
        "conformance": 100,
        "attributes_validated": ["knhk.operation.name", "knhk.sidecar.method"]
      },
      "knhk.sidecar.batch": {
        "count": 3,
        "conformance": 100,
        "attributes_validated": ["knhk.sidecar.batch.size"]
      }
    }
  }
}
```

## Integration Points

### knhk-otel OTLP Export (lib.rs:301-338)

```rust
pub fn export_spans(&self, spans: &[Span]) -> Result<(), String> {
    use reqwest::blocking::Client;
    use std::time::Duration;

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let url = format!("{}/v1/traces", self.endpoint);

    match &spans[..] {
        [] => Ok(()),
        spans => {
            let payload = self.build_otlp_spans_payload(spans);

            match client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&payload)
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
    }
}
```

### WeaverLiveCheck Start (lib.rs:229-260)

```rust
pub fn start(&self) -> Result<std::process::Child, String> {
    Self::check_weaver_available()?;
    use std::process::Command;

    let mut cmd = Command::new("weaver");
    cmd.args(&["registry", "live-check"]);

    if let Some(ref registry) = self.registry_path {
        cmd.args(&["--registry", registry]);
    }

    cmd.args(&["--otlp-grpc-address", &self.otlp_grpc_address]);
    cmd.args(&["--otlp-grpc-port", &self.otlp_grpc_port.to_string()]);
    cmd.args(&["--admin-port", &self.admin_port.to_string()]);
    cmd.args(&["--inactivity-timeout", &self.inactivity_timeout.to_string()]);
    cmd.args(&["--format", &self.format]);

    if let Some(ref output) = self.output {
        cmd.args(&["--output", output]);
    }

    cmd.spawn()
}
```

## Dependencies Analysis

### OTEL Dependencies (knhk-sidecar/Cargo.toml)

```toml
[dependencies]
knhk-otel = { path = "../knhk-otel", features = ["std"] }

[features]
default = ["otel"]
otel = []
```

### Transitive OTEL Dependencies

```
knhk-sidecar v0.5.0
├── opentelemetry v0.31.0
├── opentelemetry-http v0.31.0
├── opentelemetry-otlp v0.31.0
│   ├── opentelemetry v0.31.0
│   ├── opentelemetry-proto v0.31.0
│   └── opentelemetry_sdk v0.31.0
└── opentelemetry_sdk v0.31.0
```

**Note**: These dependencies are present in the tree but not directly used. The sidecar uses `knhk-otel` which provides a custom OTLP exporter implementation.

## Critical Path Dependencies

### Blocker Chain

1. ❌ **Schema files** (BLOCKER)
   - Must create semantic convention YAML files
   - Defines expected telemetry structure

2. ⏸️ **Schema validation** (Waiting for #1)
   - `weaver registry check`
   - Validates schema structure

3. ⏸️ **Sidecar OTEL startup** (Waiting for #1)
   - Configure OTLP endpoint
   - Enable telemetry export

4. ⏸️ **Live-check validation** (Waiting for #1, #2, #3)
   - `weaver registry live-check`
   - Validates runtime telemetry

5. ⏸️ **Test execution** (Waiting for #1-4)
   - Call gRPC methods
   - Generate telemetry
   - Verify conformance

## Recommendations

### Immediate Actions Required

1. **Create Semantic Convention Schemas**
   - Priority: P0 (CRITICAL)
   - Files: `registry/knhk-sidecar-spans.yaml`, `registry/knhk-sidecar-metrics.yaml`
   - Reference: OTel semantic conventions format

2. **Run Schema Validation**
   - Command: `weaver registry check -r registry/`
   - Fix any schema definition errors

3. **Test OTLP Export**
   - Start sidecar with OTEL enabled
   - Verify telemetry reaches endpoint
   - Check span/metric structure

4. **Execute Live-Check**
   - Start Weaver validation
   - Generate test telemetry
   - Verify 0 violations

### Long-Term Improvements

1. **Automate Validation**
   - Add Weaver check to CI/CD
   - Block PRs with telemetry violations
   - Track conformance metrics

2. **Expand Coverage**
   - Add schemas for all subsystems (etl, connectors)
   - Validate transaction telemetry
   - Cover failure scenarios

3. **Integration Testing**
   - Automated Weaver validation tests
   - Performance impact measurement
   - Schema evolution testing

## Why Weaver Validation Matters (KNHK Core Principle)

### The False Positive Problem

```
Traditional Testing (What KNHK Eliminates):
  cargo test --workspace ✅  ← Can pass even when features don't work
  └─ Tests validate test logic, not production behavior

KNHK Solution:
  weaver registry live-check ✅  ← Only passes if runtime telemetry conforms
  └─ Schema validation proves actual runtime behavior
```

### Examples of False Positives Tests Can't Catch

1. **Function exists but calls unimplemented!()**
   - Test: ✅ Compiles and passes
   - Reality: ❌ Panics in production
   - Weaver: ❌ No telemetry emitted → FAIL

2. **Command has --help but does nothing**
   - Test: ✅ `--help` returns text
   - Reality: ❌ Command doesn't work
   - Weaver: ❌ No operation telemetry → FAIL

3. **Fake-green tests**
   - Test: ✅ Asserts pass
   - Reality: ❌ Tests test the wrong thing
   - Weaver: ❌ Expected telemetry missing → FAIL

### Why Weaver is Different

- **Schema-first**: Code must conform to declared telemetry
- **Live validation**: Verifies actual runtime behavior
- **No circular dependency**: External tool validates our framework
- **Industry standard**: OTel's official validation approach
- **Detects fake-green**: Catches tests that pass but don't validate behavior

## Conclusion

**Status**: 🔴 **VALIDATION BLOCKED**

The sidecar has complete OTEL infrastructure and telemetry emission capability. However, we cannot execute Weaver validation (KNHK's source of truth) because the required semantic convention schema files do not exist in `registry/`.

**Next Steps**:
1. Create `registry/knhk-sidecar-spans.yaml`
2. Create `registry/knhk-sidecar-metrics.yaml`
3. Run `weaver registry check`
4. Start sidecar with OTEL export
5. Run `weaver live-check` validation
6. Verify 0 violations

**Critical Reminder**: Until Weaver validation passes, the sidecar's OTEL instrumentation is unverified. Tests may pass with false positives. Only Weaver validation proves the telemetry works correctly.

---

**Agent**: production-validator
**Coordination**: npx claude-flow@alpha hooks post-task --task-id "weaver-validation"
