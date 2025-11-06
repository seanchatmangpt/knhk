# knhk-otel

OpenTelemetry integration for observability, metrics, and distributed tracing.

## Overview

`knhk-otel` provides comprehensive OpenTelemetry integration for KNHK operations, including span generation, metrics collection, distributed tracing, and Weaver.ai live-check integration.

## Quick Start

```rust
use knhk_otel::{Tracer, SpanStatus, MetricsHelper};

// Create tracer
let mut tracer = Tracer::new();

// Start span
let span_ctx = tracer.start_span("knhk.operation.execute".to_string(), None);

// Record metrics
MetricsHelper::record_hook_latency(&mut tracer, 5, "ASK_SP");

// End span
tracer.end_span(span_ctx, SpanStatus::Ok);
```

## Key Features

- **Span Generation**: Real OTEL-compatible span IDs
- **Metrics Collection**: Counter, Gauge, Histogram metrics
- **OTLP Export**: Export to OTLP collectors
- **Weaver Integration**: Semantic convention validation

## Documentation

For detailed documentation, see [docs/README.md](docs/README.md).

## Related Documentation

- [Architecture](../../docs/architecture.md) - System architecture
- [Performance](../../docs/performance.md) - Performance guide
- [Weaver Integration](../../docs/weaver-integration.md) - Weaver.ai integration
