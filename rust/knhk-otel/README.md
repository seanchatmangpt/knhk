# knhk-otel

OpenTelemetry integration for the KNHK RDF processing framework.

## Features

- Automatic span generation for hot path operations
- Schema-first validation with Weaver
- Sub-8-tick instrumentation overhead
- Production-ready observability
- OTLP export support
- Zero-allocation telemetry in hot path
- OpenTelemetry SDK integration with tracing-subscriber

## Quick Start

### Basic Initialization

```rust
use knhk_otel::init_tracer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with stdout exporter (for development)
    let _guard = init_tracer("knhk-app", "1.0.0", None)?;
    
    // Use tracing macros for automatic instrumentation
    tracing::info!("Application started");
    
    // Your code here
    
    Ok(())
}
```

### OTLP Export

```rust
use knhk_otel::init_tracer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with OTLP HTTP exporter
    let _guard = init_tracer(
        "knhk-app",
        "1.0.0",
        Some("http://localhost:4318"), // OTLP HTTP endpoint
    )?;
    
    // Use tracing macros for automatic instrumentation
    tracing::info!("Application started");
    
    // Your code here
    
    Ok(())
}
```

### Manual Tracer (Legacy API)

```rust
use knhk_otel::{Tracer, SpanStatus, MetricsHelper};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tracer = Tracer::new();
    
    // Start span
    let span_ctx = tracer.start_span("knhk.operation.execute".to_string(), None);
    tracer.add_attribute(span_ctx.clone(), "knhk.operation.name".to_string(), "boot.init".to_string());
    
    // Record metrics
    MetricsHelper::record_hook_latency(&mut tracer, 5, "ASK_SP");
    
    // End span
    tracer.end_span(span_ctx, SpanStatus::Ok);
    
    Ok(())
}
```

## Weaver Schema Validation

This crate follows schema-first telemetry with OpenTelemetry Weaver validation:

```bash
weaver registry check -r registry/
weaver registry live-check --registry registry/
```

## Integration with tracing

The `init_tracer` function automatically integrates with `tracing-subscriber`, so you can use standard `tracing` macros:

```rust
use knhk_otel::init_tracer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = init_tracer("knhk-app", "1.0.0", None)?;
    
    // These automatically create OpenTelemetry spans
    tracing::info!("Info message");
    tracing::warn!("Warning message");
    tracing::error!("Error message");
    
    // Spans are automatically created for async operations
    tracing::instrument
    async fn my_operation() {
        tracing::info!("Inside operation");
    }
    
    Ok(())
}
```

## License

Licensed under MIT license.
