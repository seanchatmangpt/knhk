# CLI Guide

**Version**: 0.4.0  
**Principle**: 80/20 - Essential commands that provide 80% of value

## Overview

The KNHK CLI provides a noun-verb interface based on the CONVO.txt API specification. All commands follow the pattern:

```bash
knhk <noun> <verb> [arguments]
```

## Installation

```bash
cd rust/knhk-cli
cargo build --release
cargo install --path .
```

## Commands Reference

See [Commands](commands.md) for complete command reference.

## Configuration

Configuration is stored in:
- Unix: `~/.knhk/`
- Windows: `%APPDATA%/knhk/`

Files:
- `sigma.ttl` - Schema registry
- `q.sparql` - Invariant registry
- `connectors.json` - Connector registry
- `covers.json` - Cover definitions
- `reflexes.json` - Reflex definitions
- `epochs.json` - Epoch definitions
- `routes.json` - Route definitions

## Guard Constraints

All commands enforce guard constraints:
- **max_run_len ≤ 8** - Run length must not exceed 8
- **τ ≤ 8** - Epoch tick budget must not exceed 8
- **Operation validation** - Reflex operations must be in H_hot set

## Error Handling

All commands return exit codes:
- `0` - Success
- `1` - Error

Errors are displayed to stderr with descriptive messages.

## Examples

### Complete Workflow

```bash
# Initialize system
knhk boot init schema.ttl invariants.sparql

# Register connector
knhk connect register kafka-prod urn:knhk:schema:default kafka://localhost:9092/triples

# Define cover
knhk cover define "SELECT ?s ?p ?o WHERE { ?s ?p ?o }" "max_run_len 8"

# Declare reflex
knhk reflex declare check-count ASK_SP 0xC0FFEE 0 8

# Create epoch
knhk epoch create epoch1 8 "check-count"

# Run pipeline
knhk pipeline run --connectors kafka-prod

# Check status
knhk pipeline status
knhk metrics get
```

## See Also

- [Architecture](../architecture/README.md) - System architecture
- [API Reference](../api/README.md) - API documentation
- [Integration Guide](../integration.md) - Integration examples

