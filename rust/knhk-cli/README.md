# knhk

High-performance RDF/SPARQL processing CLI with hot path optimization.

## Installation

```bash
cargo install knhk
```

## Usage

The CLI uses noun-verb command structure:

```bash
# Initialize system (boot Σ and Q)
knhk boot init --sigma <schema_iri> --q <invariant_iri>

# Register a connector
knhk connect register --name <name> --schema <schema_iri> --source <source_uri>

# List registered connectors
knhk connect list

# Execute ETL pipeline
knhk pipeline run --connectors <connector_names> --schema <schema_iri>

# Check pipeline status
knhk pipeline status

# Admit data into system
knhk admit <data>

# Show hook details
knhk hook <hook_id>

# Show receipt details
knhk receipt <receipt_id>

# List routes
knhk route list

# Show workflow patterns
knhk workflow patterns

# Parse and register workflow
knhk workflow parse workflow.ttl
knhk workflow register workflow.ttl

# Create and execute workflow case
knhk workflow create <spec-id> --data '{"key":"value"}'
knhk workflow start <case-id>
knhk workflow execute <case-id>

## Quick Reference

### Essential Commands

```bash
# 1. Initialize system
knhk boot init --sigma <schema_iri> --q <invariant_iri>

# 2. Register a connector
knhk connect register --name my-connector --schema <schema_iri> --source <source_uri>

# 3. List connectors
knhk connect list

# 4. Run pipeline
knhk pipeline run --connectors <connector_names> --schema <schema_iri>

# 5. Check status
knhk pipeline status
```

### Source URI Formats

- **File**: `file:///path/to/data.ttl`
- **Kafka**: `kafka://broker1:9092,broker2:9092/topic-name`
- **Salesforce**: `salesforce://instance-url`
- **HTTP**: `http://api.example.com/data` or `https://api.example.com/data`

## Features

- ✅ **Sub-8-tick hot path latency** (Chatman Constant)
- ✅ **SIMD-optimized predicate matching** (4x speedup)
- ✅ **Zero-allocation buffer pooling** (>95% hit rate)
- ✅ **OpenTelemetry observability** with Weaver validation (full SDK integration)
- ✅ **Chicago TDD quality assurance** (36/36 tests passing)
- ✅ **Distributed locking** with receipt verification
- ✅ **Multi-connector support** (Kafka, Salesforce, custom)
- ✅ **Workflow engine integration** (43 YAWL patterns, REST API, CLI)
- ✅ **Enterprise workflow management** (cases, patterns, state persistence)

## Performance

```
Hot path latency: ≤8 CPU ticks
SIMD speedup: 4x over scalar baseline
Buffer pool hit rate: >95%
End-to-end latency: <1ms for typical queries
```

## Architecture

Built on lessons from simdjson:
- Buffer pooling for zero allocations
- SIMD acceleration for predicate matching
- Branchless validation for predictable performance
- Ring buffers with 64-byte padding for safe vectorization

## Observability

KNHK follows schema-first telemetry with OpenTelemetry Weaver:

```bash
# Validate telemetry schema
weaver registry check -r registry/

# Verify runtime telemetry
weaver registry live-check --registry registry/
```

## License

Licensed under MIT license.
