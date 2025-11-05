# knhk-otel Documentation

OpenTelemetry integration for observability.

## File Structure

```
rust/knhk-otel/
├── src/
│   └── lib.rs              # OTEL integration
├── examples/
│   └── weaver_live_check.rs # Example usage
└── Cargo.toml
```

## Core Components

### Span Generation
- OTEL-compatible span ID generation
- Span ID utilities

### Metrics
- Query latency metrics
- Cache hit rate metrics
- Query count metrics

### Tracing
- Distributed tracing support
- Span creation and management

## Key Features

- **Span IDs**: Real OTEL-compatible span IDs (not placeholders)
- **Metrics**: Comprehensive metrics collection
- **Tracing**: Distributed tracing support

## Related Documentation

- [Architecture](../../../docs/architecture.md) - System architecture
- [Performance](../../../docs/performance.md) - Performance guide

