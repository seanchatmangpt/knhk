# knhk-otel

OpenTelemetry integration for observability and telemetry validation.

## Overview

`knhk-otel` provides OpenTelemetry (OTEL) integration for KNHK operations, including span generation, metrics recording, and Weaver live-check integration for telemetry validation. All telemetry follows semantic conventions for consistency.

## Quick Start

```rust
use knhk_otel::{Tracer, SpanStatus, MetricsHelper, WeaverLiveCheck};

// Create tracer
let mut tracer = Tracer::new();

// Start span with semantic convention attributes
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

// Record metrics
MetricsHelper::record_hook_latency(&mut tracer, 5, "ASK_SP");
MetricsHelper::record_receipt(&mut tracer, "receipt-123");

// End span
tracer.end_span(span_ctx, SpanStatus::Ok);

// Export to OTLP endpoint
#[cfg(feature = "std")]
{
    let exporter = Tracer::with_otlp_exporter("http://localhost:4317".to_string());
    exporter.export()?;
}
```

## Weaver Live-Check Integration

```rust
use knhk_otel::WeaverLiveCheck;

// Create Weaver live-check instance
let weaver = WeaverLiveCheck::new()
    .with_registry("./schemas/reflex-enterprise-registry".to_string())
    .with_otlp_port(4317)
    .with_admin_port(8080)
    .with_format("json".to_string());

// Start Weaver (spawns process)
let mut weaver_process = weaver.start()?;

// Send telemetry to Weaver's OTLP endpoint
let mut tracer = Tracer::with_otlp_exporter(weaver.otlp_endpoint());
// ... generate telemetry ...

// Stop Weaver via HTTP admin endpoint
weaver.stop()?;
```

## Key Features

- **Span Generation**: OTEL-compatible span IDs (128-bit trace, 64-bit span)
- **Metrics Recording**: Counters, gauges, histograms
- **Weaver Integration**: Live-check validation for semantic conventions
- **Semantic Conventions**: Follows `knhk.*` naming conventions
- **OTLP Export**: HTTP/gRPC export to collectors
- **No-Std Support**: Works in no_std environments (limited features)

## Semantic Conventions

All telemetry follows these conventions:

- **Span Names**: `knhk.<noun>.<verb>` (e.g., `knhk.boot.init`)
- **Attributes**:
  - `knhk.operation.name` - Operation identifier
  - `knhk.operation.type` - Path type (hot/warm/cold)
  - `knhk.hot.latency.ticks` - Hot path latency (≤8)
  - `knhk.reflex.map.applied` - Reflex map application
  - `knhk.receipt.hash` - Receipt hash

## Metrics

- `knhk.hook.latency.ticks` - Hook execution latency
- `knhk.receipt.generated` - Receipt generation count
- `knhk.guard.violation` - Guard violation count
- `knhk.warm_path.operations.latency` - Warm path latency
- `knhk.connector.throughput` - Connector throughput
- `knhk.operation.executed` - Operation execution count

## Dependencies

- `reqwest` (optional, std feature) - HTTP client for OTLP export
- `serde_json` (optional, std feature) - JSON serialization
- `rand` (optional, std feature) - Random ID generation

## Performance

- **Span Creation**: ~100ns overhead
- **Metrics Recording**: ~50ns overhead
- **OTLP Export**: Network-bound (typically <10ms)

## Related Documentation

- [Technical Documentation](docs/README.md) - Detailed API reference
- [Architecture](../../docs/architecture.md) - System architecture
- [Weaver Integration](../../docs/weaver-integration.md) - Weaver live-check guide
- [OTEL/Weaver Summary](../../docs/otel-weaver-integration-summary.md) - Integration details

