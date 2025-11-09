# CLI - 80/20 Guide

**Version**: 1.0  
**Status**: Production-Ready  
**Last Updated**: 2025-01-XX

---

## Overview

KNHK CLI provides a noun-verb interface for all operations. This guide covers the critical 20% of commands that provide 80% of value.

**Key Features**:
- ✅ Noun-verb interface (`knhk <noun> <verb>`)
- ✅ 25+ commands covering all operations
- ✅ Configuration file support
- ✅ Guard constraint enforcement

---

## Quick Start (80% Use Case)

### Installation

```bash
cd rust/knhk-cli
cargo build --release
cargo install --path .
```

### Basic Workflow

```bash
# 1. Initialize system
knhk boot init schema.ttl invariants.sparql

# 2. Register connector
knhk connect register kafka-prod urn:knhk:schema:default kafka://localhost:9092/triples

# 3. Define cover
knhk cover define "SELECT ?s ?p ?o WHERE { ?s ?p ?o }" "max_run_len 8"

# 4. Declare reflex
knhk reflex declare check-count ASK_SP 0xC0FFEE 0 8

# 5. Run pipeline
knhk pipeline run --connectors kafka-prod
```

---

## Core Commands (80% Value)

### System Management

**Initialize System**:
```bash
knhk boot init schema.ttl invariants.sparql
```

**Check Status**:
```bash
knhk pipeline status
knhk metrics get
```

### Connector Management

**Register Connector**:
```bash
knhk connect register <name> <schema_iri> <uri>
```

**List Connectors**:
```bash
knhk connect list
```

**Example**:
```bash
knhk connect register kafka-prod urn:knhk:schema:default kafka://localhost:9092/triples
```

### Cover Management

**Define Cover**:
```bash
knhk cover define "<sparql_query>" "<constraints>"
```

**Example**:
```bash
knhk cover define "SELECT ?s ?p ?o WHERE { ?s ?p ?o }" "max_run_len 8"
```

### Reflex Management

**Declare Reflex**:
```bash
knhk reflex declare <name> <op> <s> <p> <o> [constraints]
```

**Example**:
```bash
knhk reflex declare check-count ASK_SP 0xC0FFEE 0 8
```

### Pipeline Operations

**Run Pipeline**:
```bash
knhk pipeline run --connectors <connector1>,<connector2>
```

**Stop Pipeline**:
```bash
knhk pipeline stop
```

### Epoch Management

**Create Epoch**:
```bash
knhk epoch create <name> <ticks> <reflex1>,<reflex2>
```

**Run Epoch**:
```bash
knhk epoch run <name>
```

**Example**:
```bash
knhk epoch create epoch1 8 "check-count"
knhk epoch run epoch1
```

---

## Configuration

### Configuration File Location

- **Unix/macOS**: `~/.knhk/config.toml`
- **Windows**: `%APPDATA%/knhk/config.toml`

### Configuration Hierarchy

1. Environment variables (`KNHK_*`)
2. Configuration file (`~/.knhk/config.toml`)
3. Default values

### Example Configuration

```toml
[knhk]
schema_iri = "urn:knhk:schema:default"
max_run_len = 8
enable_reflex = true

[connectors.kafka-prod]
type = "kafka"
uri = "kafka://localhost:9092/triples"
```

---

## Guard Constraints

All commands enforce guard constraints:

- **max_run_len ≤ 8** - Run length must not exceed 8
- **τ ≤ 8** - Epoch tick budget must not exceed 8
- **Operation validation** - Reflex operations must be in H_hot set

**Violations**:
- Commands return error code `1`
- Error message describes constraint violation
- Operation is rejected before execution

---

## Error Handling

**Exit Codes**:
- `0` - Success
- `1` - Error

**Error Output**:
- Errors displayed to `stderr`
- Descriptive error messages
- Constraint violations clearly indicated

**Example**:
```bash
$ knhk cover define "SELECT ?s ?p ?o WHERE { ?s ?p ?o }" "max_run_len 10"
Error: Guard constraint violation: max_run_len 10 exceeds limit 8
```

---

## Common Workflows

### Complete Setup Workflow

```bash
# 1. Initialize
knhk boot init schema.ttl invariants.sparql

# 2. Register connectors
knhk connect register kafka-prod urn:knhk:schema:default kafka://localhost:9092/triples
knhk connect register file-input urn:knhk:schema:default file:///data/input.ttl

# 3. Define covers
knhk cover define "SELECT ?s ?p ?o WHERE { ?s ?p ?o }" "max_run_len 8"

# 4. Declare reflexes
knhk reflex declare check-count ASK_SP 0xC0FFEE 0 8
knhk reflex declare validate-type VALIDATE_DATATYPE_SP 0xC0FFEE 0xDEADBEEF

# 5. Create epoch
knhk epoch create epoch1 8 "check-count,validate-type"

# 6. Run pipeline
knhk pipeline run --connectors kafka-prod,file-input

# 7. Check status
knhk pipeline status
knhk metrics get
```

### Monitoring Workflow

```bash
# Check pipeline status
knhk pipeline status

# Get metrics
knhk metrics get

# List connectors
knhk connect list

# List covers
knhk cover list

# List reflexes
knhk reflex list
```

---

## Production Readiness

### ✅ Ready for Production

- **Core Commands**: All 25+ commands functional
- **Configuration**: File and environment variable support
- **Error Handling**: Comprehensive error messages
- **Guard Constraints**: Automatic enforcement

### ⚠️ Partial Production Readiness

- **Advanced Features**: Some advanced commands may need additional configuration
- **Workflow Commands**: Workflow-specific commands require workflow engine setup

---

## Troubleshooting

### Command Not Found

**Problem**: `knhk` command not found.

**Solution**:
- Verify installation: `cargo install --path .`
- Check PATH: `which knhk`
- Reinstall if needed

### Configuration Errors

**Problem**: Configuration file errors.

**Solution**:
- Verify TOML syntax
- Check file location (`~/.knhk/config.toml`)
- Use environment variables as override

### Guard Constraint Violations

**Problem**: Commands fail with constraint violations.

**Solution**:
- Check `max_run_len ≤ 8`
- Verify operation is in H_hot set
- Review error message for specific constraint

---

## Additional Resources

### Related Consolidated Guides
- **[API Guide](API.md)** - API interfaces and programmatic access
- **[Integration Guide](INTEGRATION.md)** - Integration patterns and workflows
- **[Workflow Engine Guide](WORKFLOW_ENGINE.md)** - Workflow execution and patterns
- **[Architecture Guide](ARCHITECTURE.md)** - System architecture and components

### Detailed Documentation
- **Complete CLI Reference**: [CLI Documentation](cli.md) - Complete command reference with all options
- **Command Examples**: `examples/cli-usage/` - Comprehensive CLI examples
- **Configuration Guide**: [Configuration Documentation](configuration.md) - Complete configuration reference

### Code Examples
- **CLI Examples**: `examples/cli-usage/` - Working CLI examples
- **Workflow Examples**: `ontology/workflows/` - Workflow examples using CLI

### Related Guides
- **Quick Start**: [Quick Start Guide](QUICK_START.md) - 5-minute setup
- **Integration Guide**: [Integration Guide](INTEGRATION.md) - Integration patterns
- **API Guide**: [API Guide](API.md) - Programmatic API usage

---

## License

MIT License

---

**Last Updated**: 2025-01-XX  
**Version**: 1.0  
**Status**: Production-Ready
