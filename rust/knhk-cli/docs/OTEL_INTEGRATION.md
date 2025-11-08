# OpenTelemetry Integration with knhk-cli

## Overview

knhk-cli now includes full OpenTelemetry SDK integration for observability and telemetry validation with Weaver live-check.

## Features

- **Automatic OpenTelemetry SDK initialization** on CLI startup
- **Automatic span generation** for all commands via `tracing` macros
- **OTLP export support** to sidecar collectors
- **Weaver live-check integration** for semantic convention validation
- **Metrics recording** for operations, connectors, and pipeline execution
- **Environment variable configuration** for flexible deployment

## Usage

### Basic Usage (stdout exporter)

```bash
# Run CLI with default stdout exporter (development mode)
knhk boot init --sigma schema.ttl --q invariants.sparql
```

### OTLP Export to Sidecar

```bash
# Export telemetry to sidecar collector
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
export OTEL_SERVICE_NAME=knhk-cli
export OTEL_SERVICE_VERSION=1.0.0

knhk pipeline run --connectors kafka-prod
```

### Weaver Live-Check Validation

```bash
# Start Weaver live-check
knhk metrics weaver-start --registry ./schemas --otlp-port 4317

# Run commands (telemetry automatically exported to Weaver)
knhk pipeline run --connectors kafka-prod

# Stop Weaver and get validation report
knhk metrics weaver-stop --admin-port 8080
```

## Environment Variables

### OpenTelemetry Configuration

- `OTEL_ENABLED` - Enable/disable OpenTelemetry (default: `true`)
- `OTEL_SERVICE_NAME` - Service name (default: `knhk-cli`)
- `OTEL_SERVICE_VERSION` - Service version (default: package version)
- `OTEL_EXPORTER_OTLP_ENDPOINT` - OTLP endpoint URL (optional, uses stdout if not set)
- `KNHK_TRACE` - Tracing level (`error`, `warn`, `info`, `debug`, `trace`)

### Examples

```bash
# Disable OpenTelemetry
export OTEL_ENABLED=false
knhk boot init --sigma schema.ttl --q invariants.sparql

# Use OTLP export
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
export OTEL_SERVICE_NAME=knhk-cli-prod
knhk pipeline run --connectors kafka-prod

# Debug mode
export KNHK_TRACE=debug
knhk connect list
```

## Automatic Instrumentation

All commands are automatically instrumented with OpenTelemetry spans:

- `knhk.boot.init` - System initialization
- `knhk.connect.register` - Connector registration
- `knhk.connect.list` - Connector listing
- `knhk.pipeline.run` - Pipeline execution
- `knhk.metrics.get` - Metrics retrieval
- `knhk.metrics.weaver.start` - Weaver live-check start
- `knhk.metrics.weaver.stop` - Weaver live-check stop
- `knhk.metrics.weaver.validate` - Weaver validation

## Metrics Recorded

The CLI automatically records metrics for:

- **Operations**: Success/failure counts for all commands
- **Connector Throughput**: Triple counts from connectors
- **Pipeline Execution**: Receipts written, actions sent
- **Configuration**: Load success/failure
- **Weaver Operations**: Start, stop, validate operations

## Semantic Conventions

All spans follow KNHK semantic conventions:

- **Span Names**: `knhk.<noun>.<verb>` (e.g., `knhk.pipeline.run`)
- **Attributes**:
  - `knhk.operation.name` - Operation name
  - `knhk.operation.type` - Operation type (system, etl, configuration, query)
  - `connector.name` - Connector identifier (when applicable)
  - `schema` - Schema IRI (when applicable)

## Integration with Sidecar Pattern

The CLI works seamlessly with sidecar collectors:

1. **Start sidecar collector** (e.g., OpenTelemetry Collector)
2. **Configure OTLP endpoint** via environment variable
3. **Run CLI commands** - telemetry automatically exported
4. **View traces** in your observability platform

Example:
```bash
# Start OTEL Collector (sidecar)
docker run -p 4318:4318 otel/opentelemetry-collector

# Configure CLI to export to sidecar
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318

# Run commands - telemetry automatically exported
knhk pipeline run --connectors kafka-prod
```

## Weaver Live-Check Integration

The CLI includes Weaver live-check commands for semantic convention validation:

```bash
# Start Weaver live-check
knhk metrics weaver-start \
  --registry ./schemas \
  --otlp-port 4317 \
  --admin-port 8080 \
  --format json \
  --output ./weaver-reports

# Run commands (telemetry validated by Weaver)
knhk pipeline run --connectors kafka-prod

# Stop Weaver and get validation report
knhk metrics weaver-stop --admin-port 8080

# Validate telemetry with Weaver
knhk metrics weaver-validate \
  --registry ./schemas \
  --timeout 10
```

## Lifecycle Management

The `OtelGuard` is automatically managed in `main.rs`:

- **Initialization**: OpenTelemetry SDK initialized at startup
- **Guard Lifetime**: Guard kept alive for duration of program
- **Shutdown**: Guard dropped at end of main, flushing all telemetry

This ensures all telemetry is properly flushed before program exit.

## Related Documentation

- [knhk-otel README](../knhk-otel/README.md) - OpenTelemetry crate documentation
- [Weaver Integration](../knhk-otel/docs/README.md) - Weaver live-check guide
- [Chicago TDD Tests](../knhk-otel/tests/chicago_tdd_otel_integration.rs) - Integration tests

