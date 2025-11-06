# OTEL/Weaver Best Practices Integration - Implementation Summary

## Overview

Comprehensive integration of OpenTelemetry (OTEL) and Weaver live-check best practices from `ggen` and `clnrm` projects, maximizing observability, semantic conventions compliance, and CI/CD validation capabilities.

## Implementation Status

### ✅ Completed

1. **OTEL Dependencies** (`rust/knhk-cli/Cargo.toml`)
   - Added all OTEL dependencies with proper feature gating
   - `opentelemetry` 0.31, `tracing-opentelemetry` 0.32, `tracing-subscriber` 0.3
   - `opentelemetry-semantic-conventions` 0.15
   - All dependencies optional with `default-features = false`
   - Created `otel` feature that enables all OTEL dependencies

2. **Tracing Initialization** (`rust/knhk-cli/src/tracing.rs`)
   - `init_tracing()` function with env-filter support
   - Supports `KNHK_TRACE` environment variable for trace level
   - Supports `OTEL_EXPORTER_OTLP_ENDPOINT` for OTLP export
   - Supports `OTEL_SERVICE_NAME` for service name configuration
   - JSON format support for structured logging
   - OTLP exporter integration with automatic span export

3. **Main Entry Point** (`rust/knhk-cli/src/main.rs`)
   - Calls `tracing::init_tracing()` at startup
   - Initializes tracing before any other operations

4. **Command Instrumentation** (`rust/knhk-cli/src/boot.rs`, `rust/knhk-cli/src/commands/boot.rs`)
   - Instrumented `boot init` command with:
     - `#[instrument]` attribute with semantic convention fields
     - Span attributes: `knhk.operation.name`, `knhk.operation.type`
     - Tracing macros: `info!`, `error!`, `debug!`
     - Metrics recording via `MetricsHelper::record_operation`
     - Latency measurement with `Instant`

5. **Weaver Live-Check Integration** (`rust/knhk-cli/src/metrics.rs`, `rust/knhk-cli/src/commands/metrics.rs`)
   - `weaver_start` verb: Start Weaver live-check subprocess
   - `weaver_stop` verb: Stop Weaver via HTTP admin endpoint
   - `weaver_validate` verb: Validate telemetry with Weaver
   - Comprehensive tracing instrumentation for all Weaver operations
   - Support for registry path, OTLP port, admin port, format, output directory

6. **Metrics Helper Extension** (`rust/knhk-otel/src/lib.rs`)
   - Added `MetricsHelper::record_operation()` method
   - Records operation execution metrics with success/failure status

## Best Practices Implemented

### 1. Dependency Management
- ✅ Optional dependencies with `default-features = false`
- ✅ Feature gating with `#[cfg(feature = "std")]` guards
- ✅ Proper feature propagation (`otel` feature enables all dependencies)

### 2. Tracing Integration
- ✅ `tracing-opentelemetry` integration for span creation
- ✅ `tracing-subscriber` with env-filter, json, ansi features
- ✅ Automatic span propagation through async contexts
- ✅ Semantic conventions compliance (`knhk.*` namespace)

### 3. Instrumentation Patterns
- ✅ `#[instrument]` attribute on command handlers
- ✅ Span attributes for operation parameters
- ✅ Metrics recording for latency, throughput, errors
- ✅ Proper span status (Ok/Error/Unset)

### 4. Weaver Live-Check Integration
- ✅ Weaver subprocess management
- ✅ HTTP admin endpoint for stopping
- ✅ OTLP gRPC endpoint configuration
- ✅ Support for semantic convention registry validation
- ✅ JSON/ANSI output format support

### 5. Environment Configuration
- ✅ `KNHK_TRACE` for trace level (error, warn, info, debug, trace)
- ✅ `OTEL_EXPORTER_OTLP_ENDPOINT` for OTLP export
- ✅ `OTEL_SERVICE_NAME` for service name

## Instrumentation Pattern for Remaining Commands

All remaining command files should follow this pattern:

### Noun-Verb Wrapper Pattern (`rust/knhk-cli/src/<noun>.rs`)

```rust
#[cfg(feature = "otel")]
use tracing::instrument;

#[cfg_attr(feature = "otel", instrument(skip_all, fields(operation = "knhk.<noun>.<verb>", param1 = %param1)))]
#[verb]
fn verb(param1: String, param2: u64) -> Result<VerbResult> {
    #[cfg(feature = "otel")]
    {
        use tracing::{info, error};
        use std::time::Instant;
        
        let start = Instant::now();
        let result = commands_impl::verb(param1.clone(), param2);
        
        let duration = start.elapsed();
        match &result {
            Ok(ref data) => {
                info!(
                    duration_ms = duration.as_millis(),
                    "operation.success"
                );
            }
            Err(ref e) => {
                error!(error = %e, duration_ms = duration.as_millis(), "operation.failed");
            }
        }
        
        result.map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed: {}", e)))
    }
    
    #[cfg(not(feature = "otel"))]
    {
        commands_impl::verb(param1, param2)
            .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed: {}", e)))
    }
}
```

### Command Implementation Pattern (`rust/knhk-cli/src/commands/<noun>.rs`)

```rust
#[cfg(feature = "otel")]
use tracing::{info, error, debug, span, Level};
#[cfg(feature = "otel")]
use knhk_otel::{Tracer, MetricsHelper, SpanStatus};

pub fn verb(param1: String, param2: u64) -> Result<VerbResult, String> {
    #[cfg(feature = "otel")]
    let _span = span!(Level::INFO, "knhk.<noun>.<verb>", 
        knhk.operation.name = "<noun>.<verb>", 
        knhk.operation.type = "operation_type");
    
    #[cfg(feature = "otel")]
    let _enter = _span.enter();
    
    // Implementation...
    
    #[cfg(feature = "otel")]
    {
        info!("operation.completed");
        
        // Record metrics
        let mut tracer = Tracer::new();
        MetricsHelper::record_operation(&mut tracer, "<noun>.<verb>", true);
    }
    
    Ok(result)
}
```

## Semantic Conventions

All spans should use the following semantic convention attributes:

- `knhk.operation.name` - Operation name (e.g., `boot.init`, `metrics.get`)
- `knhk.operation.type` - Operation type (`system`, `query`, `validation`, etc.)
- `knhk.entity.id` - Entity ID (receipt, hook, connector, etc.)
- `knhk.entity.type` - Entity type (`receipt`, `hook`, `connector`, etc.)

Span names should follow the pattern: `knhk.<noun>.<verb>` (e.g., `knhk.boot.init`, `knhk.metrics.weaver.start`)

## Remaining Work

### Commands to Instrument

The following command files need instrumentation following the patterns above:

1. `rust/knhk-cli/src/connect.rs` + `rust/knhk-cli/src/commands/connect.rs`
2. `rust/knhk-cli/src/cover.rs` + `rust/knhk-cli/src/commands/cover.rs`
3. `rust/knhk-cli/src/admit.rs` + `rust/knhk-cli/src/commands/admit.rs`
4. `rust/knhk-cli/src/reflex.rs` + `rust/knhk-cli/src/commands/reflex.rs`
5. `rust/knhk-cli/src/epoch.rs` + `rust/knhk-cli/src/commands/epoch.rs`
6. `rust/knhk-cli/src/route.rs` + `rust/knhk-cli/src/commands/route.rs`
7. `rust/knhk-cli/src/receipt.rs` + `rust/knhk-cli/src/commands/receipt.rs`
8. `rust/knhk-cli/src/pipeline.rs` + `rust/knhk-cli/src/commands/pipeline.rs`
9. `rust/knhk-cli/src/coverage.rs` + `rust/knhk-cli/src/commands/coverage.rs`
10. `rust/knhk-cli/src/context.rs` + `rust/knhk-cli/src/commands/context.rs`
11. `rust/knhk-cli/src/config.rs` + `rust/knhk-cli/src/commands/config.rs`
12. `rust/knhk-cli/src/hook.rs` + `rust/knhk-cli/src/commands/hook.rs`

### Documentation Updates

- [ ] Update `docs/weaver-integration.md` with comprehensive patterns
- [ ] Update `rust/knhk-cli/docs/README.md` with OTEL/tracing documentation
- [ ] Add examples for CI/CD integration

## Usage Examples

### Enable Tracing

```bash
export KNHK_TRACE=debug
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export OTEL_SERVICE_NAME=knhk-cli

knhk boot init schema.ttl invariants.sparql
```

### Weaver Live-Check

```bash
# Start Weaver live-check
knhk metrics weaver-start --registry ./schemas/my-registry --otlp-port 4317 --admin-port 8080

# Validate telemetry
knhk metrics weaver-validate --timeout 10

# Stop Weaver
knhk metrics weaver-stop --admin-port 8080
```

## Testing

- [x] `knhk-otel` compiles successfully with `std` feature
- [x] `knhk-cli` compiles successfully with `otel` feature
- [ ] Test tracing initialization with various `KNHK_TRACE` values
- [ ] Test Weaver live-check subprocess integration
- [ ] Test OTLP export to external collector
- [ ] Test semantic conventions compliance via Weaver validation
- [ ] Test CI/CD integration with exit codes

## Notes

- Pre-existing compilation errors in `knhk-etl` are unrelated to OTEL/Weaver integration
- All OTEL functionality is behind the `otel` feature flag
- Tracing works without OTLP endpoint (uses no-op tracer provider)
- Weaver live-check requires `weaver` binary to be available in PATH

