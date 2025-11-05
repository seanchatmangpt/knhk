# KNHK CLI Guide

**Version**: 0.5.0  
**Status**: ✅ Production Ready  
**CLI Reference**: Complete command reference for all operations

**See [v0.4.0 Status](v0.4.0-status.md) for complete status and limitations.**

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

## Configuration

KNHK supports configuration via TOML file and environment variables. Configuration is loaded from:

1. **Environment Variables** (highest priority) - `KNHK_*` prefix
2. **Config File** - `~/.knhk/config.toml` (or `%APPDATA%/knhk/config.toml` on Windows)
3. **Default Configuration** (lowest priority)

### Configuration File

Create `~/.knhk/config.toml`:

```toml
[knhk]
version = "0.5.0"
context = "default"

[connectors.kafka-prod]
type = "kafka"
bootstrap_servers = ["localhost:9092"]
topic = "triples"
schema = "urn:knhk:schema:enterprise"
max_run_len = 8
max_batch_size = 1000

[epochs.default]
tau = 8
ordering = "deterministic"

[hooks]
max_count = 100

[routes.webhook-1]
kind = "webhook"
target = "https://api.example.com/webhook"
encode = "json-ld"
```

### Environment Variables

Environment variables override config file values:

```bash
export KNHK_CONTEXT=production
export KNHK_CONNECTOR_KAFKA_BOOTSTRAP_SERVERS=localhost:9092
export KNHK_EPOCH_DEFAULT_TAU=8
```

### Show Configuration

View current configuration (including environment variable overrides):

```bash
knhk config show
```

## Commands

### Boot - System Initialization

**Initialize Σ and Q**
```bash
knhk boot init <sigma.ttl> <q.sparql>
```

Example:
```bash
knhk boot init schema.ttl invariants.sparql
```

### Connect - Connector Management

**Register Connector**
```bash
knhk connect register <name> <schema> <source>
```

**List Connectors**
```bash
knhk connect list
```

Example:
```bash
knhk connect register kafka-prod urn:knhk:schema:default kafka://localhost:9092/triples
knhk connect list
```

### Cover - Cover Definition

**Define Cover**
```bash
knhk cover define <select> <shard>
```

**List Covers**
```bash
knhk cover list
```

Example:
```bash
knhk cover define "SELECT ?s ?p ?o WHERE { ?s ?p ?o }" "max_run_len 8"
```

### Admit - Delta Admission

**Admit Delta**
```bash
knhk admit delta <delta_file>
```

Example:
```bash
knhk admit delta delta.json
```

### Reflex - Reflex Declaration

**Declare Reflex**
```bash
knhk reflex declare <name> <op> <pred> <off> <len>
```

**List Reflexes**
```bash
knhk reflex list
```

Example:
```bash
knhk reflex declare check-count ASK_SP 0xC0FFEE 0 8
```

Valid operations (H_hot set):
- ASK_SP, COUNT_SP_GE, COUNT_SP_LE, COUNT_SP_EQ
- ASK_SPO, ASK_OP, UNIQUE_SP
- COUNT_OP_GE, COUNT_OP_LE, COUNT_OP_EQ
- COMPARE_O_EQ, COMPARE_O_GT, COMPARE_O_LT, COMPARE_O_GE, COMPARE_O_LE
- CONSTRUCT8

### Epoch - Epoch Operations

**Create Epoch**
```bash
knhk epoch create <id> <tau> <lambda>
```

**Run Epoch**
```bash
knhk epoch run <id>
```

**List Epochs**
```bash
knhk epoch list
```

Example:
```bash
knhk epoch create epoch1 8 "reflex1,reflex2"
knhk epoch run epoch1
```

### Route - Action Routing

**Install Route**
```bash
knhk route install <name> <kind> <target>
```

**List Routes**
```bash
knhk route list
```

Route kinds:
- `webhook` - HTTP webhook (http:// or https://)
- `kafka` - Kafka topic (kafka://brokers/topic)
- `grpc` - gRPC endpoint (grpc://host:port/service/method)
- `lockchain` - Git lockchain (file:// or git://)

Example:
```bash
knhk route install webhook1 webhook https://api.example.com/webhook
knhk route install kafka1 kafka kafka://localhost:9092/actions
```

### Receipt - Receipt Operations

**Get Receipt**
```bash
knhk receipt get <id>
```

**Merge Receipts**
```bash
knhk receipt merge <id1,id2,id3>
```

**List Receipts**
```bash
knhk receipt list
```

### Pipeline - ETL Pipeline

**Run Pipeline**
```bash
knhk pipeline run [--connectors <ids>] [--schema <iri>]
```

**Pipeline Status**
```bash
knhk pipeline status
```

Example:
```bash
knhk pipeline run --connectors kafka-prod
knhk pipeline status
```

### Metrics - OTEL Metrics

**Get Metrics**
```bash
knhk metrics get
```

### Coverage - Dark Matter Coverage

**Get Coverage**
```bash
knhk coverage get
```

### Config - Configuration Management

**Show Configuration**
```bash
knhk config show
```

Example:
```bash
knhk config show
```

This command displays the current configuration including:
- KNHK version and context
- Connectors configuration
- Epochs configuration
- Routes configuration
- Environment variable overrides

## Troubleshooting

### Common Issues

**Configuration not loading**
- Check that `~/.knhk/config.toml` exists and is valid TOML
- Verify environment variables are set correctly
- Run `knhk config show` to see current configuration

**Command not found**
- Ensure KNHK CLI is installed: `cargo install --path rust/knhk-cli`
- Check PATH includes cargo bin directory

**Permission errors**
- Ensure write access to `~/.knhk/` directory
- Check file permissions on config files

## Command Reference Table

| Noun | Verb | Description |
|------|------|-------------|
| boot | init | Initialize Σ and Q registries |
| connect | register | Register a connector |
| connect | list | List connectors |
| cover | define | Define cover over O |
| cover | list | List covers |
| admit | delta | Admit Δ into O |
| reflex | declare | Declare a reflex |
| reflex | list | List reflexes |
| epoch | create | Create an epoch |
| epoch | run | Run an epoch |
| epoch | list | List epochs |
| route | install | Install a route |
| route | list | List routes |
| receipt | get | Get receipt |
| receipt | merge | Merge receipts |
| receipt | list | List receipts |
| receipt | verify | Verify receipt |
| receipt | show | Show receipt details |
| pipeline | run | Run ETL pipeline |
| pipeline | status | Get pipeline status |
| metrics | get | Get OTEL metrics |
| coverage | get | Get Dark Matter coverage |
| hook | create | Create a hook |
| hook | list | List hooks |
| hook | eval | Evaluate a hook |
| hook | show | Show hook details |
| context | create | Create context |
| context | list | List contexts |
| context | switch | Switch context |
| config | show | Show current configuration |
- `covers.json` - Cover definitions
- `reflexes.json` - Reflex definitions
- `epochs.json` - Epoch definitions
- `routes.json` - Route definitions

## Guard Constraints

All commands enforce guard constraints:
- **max_run_len ≤ 8** - Run length must not exceed 8
- **τ ≤ 8** - Epoch tick budget must not exceed 8
- **Operation validation** - Reflex operations must be in H_hot set

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

- [Architecture](architecture.md) - System architecture
- [API Reference](api.md) - API documentation
- [Integration Guide](integration.md) - Integration examples
