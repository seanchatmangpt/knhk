# Weaver Validation Troubleshooting Guide

**Purpose**: Diagnose and fix common Weaver schema validation errors
**Context**: Weaver validation is the source of truth - if it fails, telemetry is broken
**Commands**: `weaver registry check -r registry/` and `weaver registry live-check`

---

## Understanding Weaver Validation

**Two validation levels:**

1. **Schema Check** (`weaver registry check`): Validates schema definition syntax and structure
2. **Live Check** (`weaver registry live-check`): Validates runtime telemetry against schema

**Both must pass for production readiness.**

---

## Error 1: Schema File Not Found

### Symptom
```
Error: Registry not found at path: registry/
```

### Cause
- Schema directory doesn't exist
- Wrong path specified
- Schema files missing

### Solution

```bash
# Check if registry directory exists
ls -la /home/user/knhk/registry/

# If missing, create it
mkdir -p /home/user/knhk/registry/

# Add schema files (YAML format)
cat > /home/user/knhk/registry/knhk-schema.yaml <<EOF
groups:
  - id: knhk.query
    type: span
    brief: "KNHK query operations"
    attributes:
      - id: query.type
        type: string
        brief: "Type of query (ASK, SELECT, CONSTRUCT)"
        examples: ["ASK", "SELECT"]
      - id: query.latency_ns
        type: int
        brief: "Query latency in nanoseconds"
EOF

# Validate
weaver registry check -r /home/user/knhk/registry/
```

### Prevention
- Keep schema files in version control (Git)
- Use absolute paths when running Weaver
- Document schema location in README

---

## Error 2: Invalid Schema Syntax

### Symptom
```
Error: Failed to parse schema file: knhk-schema.yaml
Expected mapping, found sequence
```

### Cause
- YAML syntax error (indentation, structure)
- Invalid schema fields
- Missing required fields

### Solution

```bash
# Validate YAML syntax
yamllint /home/user/knhk/registry/knhk-schema.yaml

# Check schema structure (must have these fields)
cat /home/user/knhk/registry/knhk-schema.yaml
```

**Correct Schema Structure:**
```yaml
groups:
  - id: knhk.query              # Required: unique identifier
    type: span                   # Required: span | metric | log
    brief: "Query operations"    # Required: short description
    attributes:                  # Optional: span attributes
      - id: query.type
        type: string             # Required: string | int | boolean | double
        brief: "Query type"
        examples: ["ASK"]        # Optional but recommended
```

**Common YAML Mistakes:**
```yaml
# ❌ Wrong: Mixed indentation (tabs + spaces)
groups:
	- id: knhk.query
  type: span

# ✅ Correct: Consistent 2-space indentation
groups:
  - id: knhk.query
    type: span

# ❌ Wrong: Missing colon
groups
  - id: knhk.query

# ✅ Correct: Colon after groups
groups:
  - id: knhk.query
```

### Prevention
- Use a YAML validator (`yamllint`)
- Use consistent indentation (2 spaces)
- Follow OpenTelemetry semantic conventions

---

## Error 3: Duplicate Attribute Definitions

### Symptom
```
Error: Duplicate attribute definition: query.type
Found in groups: knhk.query, knhk.query.hot
```

### Cause
- Same attribute defined in multiple schema groups
- Copy-paste error
- Inconsistent schema organization

### Solution

```yaml
# ❌ Wrong: Duplicate attributes
groups:
  - id: knhk.query
    type: span
    attributes:
      - id: query.type
        type: string

  - id: knhk.query.hot
    type: span
    attributes:
      - id: query.type      # Duplicate!
        type: string

# ✅ Correct: Define once, reference in multiple groups
groups:
  - id: knhk.query
    type: span
    attributes:
      - id: query.type
        type: string
        brief: "Query type"

  - id: knhk.query.hot
    type: span
    extends: knhk.query     # Inherit attributes
    attributes:
      - id: query.hot_path
        type: boolean
        brief: "Is hot path query"
```

### Prevention
- Use schema inheritance (`extends`)
- Define common attributes once
- Use namespacing (e.g., `knhk.query.*` vs `knhk.workflow.*`)

---

## Error 4: Attribute Type Mismatch

### Symptom
```
Error: Attribute 'query.latency_ns' has type mismatch
Schema defines: int
Runtime emits: string
```

### Cause
- Code emits wrong type (e.g., string instead of int)
- Schema defines wrong type
- Implicit type conversion issue

### Solution

**Check schema definition:**
```yaml
attributes:
  - id: query.latency_ns
    type: int              # Schema expects int
    brief: "Query latency in nanoseconds"
```

**Check code emission:**
```rust
// ❌ Wrong: Emitting string
tracer.add_attribute(
    span_ctx.clone(),
    "query.latency_ns".to_string(),
    "123".to_string(),  // String, not int!
);

// ✅ Correct: Emit int
tracer.add_attribute(
    span_ctx.clone(),
    "query.latency_ns".to_string(),
    123.to_string(),  // Or use proper int serialization
);

// Better: Use typed helper
tracer.add_attribute_int(
    span_ctx.clone(),
    "query.latency_ns".to_string(),
    123,
);
```

### Prevention
- Use typed telemetry helpers (not raw strings)
- Validate types in tests
- Use Weaver live-check in CI/CD

---

## Error 5: Missing Required Attributes

### Symptom
```
Error: Span 'knhk.query.execute' missing required attribute 'query.type'
```

### Cause
- Code doesn't emit required attribute
- Attribute name typo
- Conditional attribute emission (not always emitted)

### Solution

**Schema marks attribute as required:**
```yaml
attributes:
  - id: query.type
    type: string
    requirement_level: required  # This attribute is required
    brief: "Query type"
```

**Code must always emit it:**
```rust
// ❌ Wrong: Conditional emission
if query_type == "ASK" {
    tracer.add_attribute(span_ctx.clone(), "query.type".to_string(), "ASK".to_string());
}

// ✅ Correct: Always emit
let query_type = match query {
    Query::Ask(_) => "ASK",
    Query::Select(_) => "SELECT",
    Query::Construct(_) => "CONSTRUCT",
};
tracer.add_attribute(span_ctx.clone(), "query.type".to_string(), query_type.to_string());
```

### Prevention
- Mark only truly required attributes as `requirement_level: required`
- Use `requirement_level: recommended` for optional attributes
- Test all code paths emit required attributes

---

## Error 6: Span Name Not Defined in Schema

### Symptom
```
Error: Span name 'knhk.query.hot.execute' not found in schema
```

### Cause
- Code uses span name not defined in schema
- Schema not updated after code changes
- Typo in span name

### Solution

**Add span to schema:**
```yaml
groups:
  - id: knhk.query.hot.execute
    type: span
    span_kind: internal        # internal | server | client | producer | consumer
    brief: "Execute hot path query"
    attributes:
      - id: query.type
        type: string
        requirement_level: required
```

**Or fix code to match schema:**
```rust
// ❌ Wrong: Span name not in schema
let span_ctx = tracer.start_span("knhk.query.hot.execute".to_string(), None);

// ✅ Correct: Use schema-defined name
let span_ctx = tracer.start_span("knhk.query.execute".to_string(), None);
```

### Prevention
- Schema-first development (define schema before coding)
- CI/CD runs Weaver live-check
- Code generation from schema (if possible)

---

## Error 7: Metric Unit Mismatch

### Symptom
```
Error: Metric 'knhk.query.latency' has unit mismatch
Schema defines: ns (nanoseconds)
Runtime emits: ms (milliseconds)
```

### Cause
- Code emits wrong unit
- Schema defines wrong unit
- Unit conversion not applied

### Solution

**Schema defines unit:**
```yaml
groups:
  - id: knhk.query.latency
    type: metric
    metric_name: knhk.query.latency
    brief: "Query execution latency"
    instrument: histogram
    unit: ns                 # Nanoseconds
```

**Code must emit matching unit:**
```rust
// ❌ Wrong: Emitting milliseconds
let latency_ms = duration.as_millis();
MetricsHelper::record_latency(&mut tracer, "query.execute", latency_ms);

// ✅ Correct: Emit nanoseconds
let latency_ns = duration.as_nanos();
MetricsHelper::record_latency(&mut tracer, "query.execute", latency_ns);
```

### Prevention
- Use standard units (ns, ms, s for time; B, KB, MB for size)
- Document units in schema and code comments
- Use helper functions that enforce units

---

## Error 8: Live Check Fails But Schema Check Passes

### Symptom
```
$ weaver registry check -r registry/
✓ Schema valid

$ weaver registry live-check --registry registry/
✗ No telemetry data received
```

### Cause
- Application not running
- OTLP exporter not configured
- Telemetry not reaching Weaver
- No operations executed (no telemetry emitted)

### Solution

**Step 1: Verify application running and emitting telemetry:**
```rust
use knhk_otel::init_tracer;

// Initialize with OTLP endpoint
let _guard = init_tracer("knhk-test", "1.0.0", Some("http://localhost:4318"))
    .expect("Failed to initialize tracer");

// Execute operation that emits telemetry
let mut tracer = Tracer::new();
let span_ctx = tracer.start_span("knhk.query.execute".to_string(), None);
tracer.end_span(span_ctx, SpanStatus::Ok);

// Flush telemetry
drop(_guard);  // OtelGuard flushes on drop
```

**Step 2: Verify OTLP collector running:**
```bash
# Check if collector is running
curl http://localhost:4318/v1/traces

# Check collector logs
docker logs otlp-collector

# Start collector if not running
docker run -p 4318:4318 otel/opentelemetry-collector:latest
```

**Step 3: Run live-check with collector:**
```bash
# Start collector
docker run -d --name weaver-collector -p 4318:4318 otel/opentelemetry-collector:latest

# Run application
cargo run --bin knhk-cli -- query ask "ASK { ?s ?p ?o }"

# Run live-check
weaver registry live-check --registry /home/user/knhk/registry/ \
    --otlp-endpoint http://localhost:4318
```

### Prevention
- Always test with OTLP collector running
- Verify telemetry in collector logs before Weaver
- Run live-check in CI/CD with real collector

---

## Error 9: Context Propagation Failure

### Symptom
```
Error: Span 'knhk.workflow.step' has no parent
Expected parent span 'knhk.workflow.execute'
```

### Cause
- Parent span not propagated to child operation
- Async context lost
- Thread-local storage cleared

### Solution

**Propagate context explicitly:**
```rust
// ❌ Wrong: No parent context
async fn execute_workflow_step() {
    let span_ctx = tracer.start_span("knhk.workflow.step".to_string(), None);
    // ...
}

// ✅ Correct: Pass parent context
async fn execute_workflow_step(parent_span: &SpanContext) {
    let span_ctx = tracer.start_span(
        "knhk.workflow.step".to_string(),
        Some(parent_span.clone()),  // Link to parent
    );
    // ...
}
```

**Use tracing instrumentation:**
```rust
use tracing::instrument;

#[instrument(name = "knhk.workflow.execute")]
async fn execute_workflow() {
    // Child spans automatically get parent context
    execute_workflow_step().await;
}

#[instrument(name = "knhk.workflow.step")]
async fn execute_workflow_step() {
    // Automatically linked to parent
}
```

### Prevention
- Use `tracing::instrument` for automatic context propagation
- Pass `SpanContext` explicitly in function signatures
- Test distributed tracing in integration tests

---

## Error 10: Schema Version Mismatch

### Symptom
```
Error: Schema version mismatch
Runtime uses schema version: 1.0.0
Registry has schema version: 2.0.0
```

### Cause
- Schema updated but code not rebuilt
- Multiple schema versions deployed
- Version not bumped after schema change

### Solution

**Bump schema version:**
```yaml
# registry/knhk-schema.yaml
schema_url: https://example.com/schemas/knhk/1.1.0  # Bump version
groups:
  # ... schema definition
```

**Rebuild code with new schema:**
```bash
# Clean build
cargo clean
cargo build --release

# Verify version
cargo run --bin knhk-cli -- version
# Should show: knhk v1.1.0 (schema v1.1.0)
```

### Prevention
- Version control schema files
- CI/CD verifies schema version matches binary
- Semantic versioning for schema changes

---

## Quick Diagnostic Commands

```bash
# 1. Validate schema syntax
weaver registry check -r /home/user/knhk/registry/

# 2. Run application with telemetry
RUST_LOG=debug cargo run --bin knhk-cli -- query ask "ASK { ?s ?p ?o }"

# 3. Check OTLP collector
curl http://localhost:4318/v1/traces
docker logs otlp-collector | tail -50

# 4. Run live-check
weaver registry live-check --registry /home/user/knhk/registry/

# 5. Inspect telemetry data
# (use Jaeger UI or collector debug endpoint)
curl http://localhost:16686/api/traces?service=knhk-test
```

---

## Common Schema Patterns

### Pattern 1: Span with Attributes
```yaml
groups:
  - id: knhk.query.execute
    type: span
    span_kind: internal
    brief: "Execute SPARQL query"
    attributes:
      - id: query.type
        type: string
        requirement_level: required
        brief: "Query type (ASK, SELECT, CONSTRUCT)"
        examples: ["ASK", "SELECT"]
      - id: query.latency_ns
        type: int
        requirement_level: recommended
        brief: "Query latency in nanoseconds"
```

### Pattern 2: Metric with Unit
```yaml
groups:
  - id: knhk.query.latency
    type: metric
    metric_name: knhk.query.latency
    brief: "Query execution latency distribution"
    instrument: histogram
    unit: ns
    attributes:
      - id: query.type
        type: string
```

### Pattern 3: Log with Severity
```yaml
groups:
  - id: knhk.query.error
    type: log_record
    brief: "Query execution error"
    attributes:
      - id: error.type
        type: string
        requirement_level: required
      - id: error.message
        type: string
        requirement_level: required
```

---

## Weaver Validation Best Practices

1. **Schema-First Development**: Define schema before writing code
2. **Version Control**: Keep schema in Git with code
3. **CI/CD Integration**: Run Weaver checks in CI
4. **Live Testing**: Test with real OTLP collector
5. **Documentation**: Document schema in README
6. **Semantic Versioning**: Bump schema version on changes
7. **Backward Compatibility**: Don't remove attributes (deprecate instead)
8. **Code Generation**: Generate code from schema (if possible)

---

## See Also

- [Telemetry Checklist](/home/user/knhk/docs/reference/cards/TELEMETRY_CHECKLIST.md)
- [Production Readiness Checklist](/home/user/knhk/docs/reference/cards/PRODUCTION_READINESS_CHECKLIST.md)
- [Weaver Documentation](https://github.com/open-telemetry/weaver)
- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
