# Weaver Integration - Consolidated Guide

**Last Updated:** 2025-11-07  
**Status:** ✅ Integrated

---

## Overview

Weaver is OpenTelemetry's "Observability by Design" platform that treats telemetry as a first-class public API. KNHK integrates Weaver for live-check validation of telemetry against semantic conventions.

---

## Integration Status

### ✅ Completed
- Weaver live-check integrated into sidecar
- Real-time telemetry validation
- Schema-first validation approach
- OTEL span integration for errors
- Structured error diagnostics with OTEL correlation

### Architecture
- **Live-Check Process**: Validates telemetry against semantic conventions
- **Health Checks**: Monitors Weaver process health
- **Automatic Restarts**: Restarts Weaver if it crashes
- **Graceful Shutdown**: Proper cleanup on sidecar shutdown

---

## Key Learnings from Weaver

### 1. Architecture Patterns
- **Modular Crate Design**: Workspace-based organization (already following)
- **Ingester Pattern**: Multiple input sources via trait-based ingesters
- **Advisor Pattern**: Pluggable advisors for validation (can apply to schema validation)

### 2. Error Handling
- **Diagnostic System**: Structured diagnostics with rich context
- **Error Types**: `thiserror` with `#[non_exhaustive]` enum variants (already using)

### 3. Schema Management
- **Resolved Schema Pattern**: Self-contained schemas without external references
- **Registry Pattern**: Centralized registry with versioning

### 4. Validation and Policy
- **Policy Engine**: Rego-based policy engine for custom validation rules
- **Live-Check Architecture**: Streaming validation with multiple advisors

---

## Implementation Details

### Sidecar Integration

**Configuration:**
```rust
pub struct SidecarConfig {
    pub weaver_endpoint: Option<String>,
    // ... other config
}
```

**Usage:**
```rust
let server = SidecarServer::new_with_weaver(
    server_config,
    client,
    metrics,
    health,
    Some(weaver_endpoint),
    Some(beat_admission),
).await?;
```

### Error Diagnostics

Structured error diagnostics with OTEL span integration:
```rust
pub struct ErrorContext {
    pub code: String,
    pub message: String,
    pub attributes: BTreeMap<String, String>,
    pub source_location: Option<String>,
    pub otel_span_id: Option<u64>,
}
```

### Telemetry Export

Continuous telemetry export to Weaver:
- Spans for all pipeline stages
- Metrics for performance tracking
- Errors linked to spans via span_id

---

## Future Improvements (v1.1)

### High Priority
1. **Policy Engine Integration** - Rego-based policies for guard constraints
2. **Improved Error Diagnostics** - miette-style diagnostics with JSON output
3. **Schema Resolution** - Resolved schema pattern for RDF schemas

### Medium Priority
4. **Streaming Processing** - Streaming ingesters for RDF parsing
5. **Template Engine** - Enhanced AOT template engine with Jinja2-like features

---

## Files

- `rust/knhk-sidecar/src/lib.rs` - Weaver integration
- `rust/knhk-sidecar/src/service.rs` - Telemetry export
- `rust/knhk-sidecar/src/error.rs` - Structured error diagnostics
- `rust/knhk-otel/src/lib.rs` - Weaver live-check client
- `scripts/verify-weaver.sh` - Verification script

---

## References

- [Weaver Documentation](https://github.com/open-telemetry/opentelemetry-weaver)
- [OTEL Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- `docs/WEAVER_PATTERN_ALIGNMENT.md` - Pattern alignment analysis
- `docs/WEAVER_INSIGHTS_IMPLEMENTATION.md` - Implementation details

