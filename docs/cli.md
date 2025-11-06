# KNHK CLI Guide

**Version**: 0.5.0  
**Status**: ✅ Production Ready  
**CLI Reference**: Complete command reference for all operations

**Formal Foundation**: CLI commands enforce formal laws:
- **Typing**: O ⊨ Σ - Schema validation before operations
- **Law**: A = μ(O) - Actions are deterministic projections
- **Epoch Containment**: μ ⊂ τ - Time bounds enforced
- **Provenance**: hash(A) = hash(μ(O)) - Receipt generation

See [Formal Mathematical Foundations](formal-foundations.md) for complete treatment.

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

KNHK uses TOML configuration files with environment variable override support.

### Configuration File Location

- **Unix/macOS**: `~/.knhk/config.toml`
- **Windows**: `%APPDATA%/knhk/config.toml`

### Configuration Hierarchy

Configuration values are loaded in this order (highest priority first):
1. Environment variables (`KNHK_*`)
2. Configuration file (`~/.knhk/config.toml`)
3. Default values

### Example Configuration File

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

Environment variables override configuration file values:

- `KNHK_CONTEXT` - Set default context
- `KNHK_CONNECTOR_<NAME>_BOOTSTRAP_SERVERS` - Connector bootstrap servers (comma-separated)
- `KNHK_CONNECTOR_<NAME>_TOPIC` - Connector topic
- `KNHK_CONNECTOR_<NAME>_SCHEMA` - Connector schema
- `KNHK_CONNECTOR_<NAME>_MAX_RUN_LEN` - Connector max run length (must be ≤ 8)
- `KNHK_EPOCH_<NAME>_TAU` - Epoch tau value (must be ≤ 8)
- `KNHK_EPOCH_<NAME>_ORDERING` - Epoch ordering

Example:
```bash
export KNHK_CONTEXT=production
export KNHK_CONNECTOR_KAFKA_PROD_BOOTSTRAP_SERVERS=localhost:9092,localhost:9093
export KNHK_EPOCH_DEFAULT_TAU=8
```

## Commands

### Boot - System Initialization

**Initialize Σ and Q**
```bash
knhk boot init <sigma.ttl> <q.sparql>
```

Initializes the system with schema (Σ) and invariants (Q).

Example:
```bash
knhk boot init schema.ttl invariants.sparql
```

### Connect - Connector Management

**Register Connector**
```bash
knhk connect register <name> <schema> <source>
```

Registers a connector for data ingestion. Supports Kafka, Salesforce, HTTP, and file sources.

**List Connectors**
```bash
knhk connect list
```

Examples:
```bash
# Register Kafka connector
knhk connect register kafka-prod urn:knhk:schema:default kafka://localhost:9092/triples

# Register Salesforce connector
knhk connect register sf-prod urn:knhk:schema:salesforce salesforce://instance.salesforce.com

# List connectors
knhk connect list
```

### Cover - Cover Definition

**Define Cover**
```bash
knhk cover define <select> <shard>
```

Defines a cover over the ontology O with shard constraints.

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

Admits a delta (Δ) into the ontology O.

Example:
```bash
knhk admit delta delta.json
```

### Reflex - Reflex Declaration

**Declare Reflex**
```bash
knhk reflex declare <name> <op> <pred> <off> <len>
```

Declares a reflex (hook) with operation, predicate, offset, and length.

**List Reflexes**
```bash
knhk reflex list
```

Example:
```bash
knhk reflex declare check-count ASK_SP 0xC0FFEE 0 8
```

Valid operations (H_hot set):
- **Hot Path** (≤8 ticks): ASK_SP, ASK_SPO, ASK_OP, COUNT_SP_GE/LE/EQ, COUNT_OP_GE/LE/EQ, UNIQUE_SP, COMPARE_O_EQ/GT/LT/GE/LE, VALIDATE_DATATYPE_SP/SPO
- **Warm Path** (≤500ms): CONSTRUCT8

### Epoch - Epoch Operations

**Create Epoch**
```bash
knhk epoch create <id> <tau> <lambda>
```

Creates an epoch with tau (≤8) and lambda (ordering).

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

Installs a route for action delivery.

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

**Verify Receipt**
```bash
knhk receipt verify <id>
```

**Show Receipt**
```bash
knhk receipt show <id>
```

Example:
```bash
knhk receipt get receipt-123
knhk receipt merge receipt-1,receipt-2,receipt-3
knhk receipt verify receipt-123
```

### Pipeline - ETL Pipeline

**Run Pipeline**
```bash
knhk pipeline run [--connectors <ids>] [--schema <iri>]
```

Executes the ETL pipeline: Ingest → Transform → Load → Reflex → Emit

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

Retrieves OpenTelemetry metrics.

### Coverage - Dark Matter Coverage

**Get Coverage**
```bash
knhk coverage get
```

Retrieves 80/20 coverage metrics.

### Hook - Knowledge Hook Operations

**Create Hook**
```bash
knhk hook create <name> <op> <pred> <off> <len> [s] [p] [o] [k]
```

Creates a knowledge hook with operation and parameters.

**List Hooks**
```bash
knhk hook list
```

**Evaluate Hook**
```bash
knhk hook eval <name>
```

Evaluates a hook (routes to hot path or warm path based on operation).

**Show Hook**
```bash
knhk hook show <name>
```

Example:
```bash
# Create hot path hook (ASK_SP)
knhk hook create check-exists ASK_SP 0xC0FFEE 0 8 0x1234

# Create warm path hook (CONSTRUCT8)
knhk hook create emit-triples CONSTRUCT8 0xC0FFEE 0 8 0x1234 0x5678

# Evaluate hook
knhk hook eval check-exists
```

### Context - Context Management

**Create Context**
```bash
knhk context create <id> <name> <schema>
```

**List Contexts**
```bash
knhk context list
```

**Show Current Context**
```bash
knhk context current
```

**Switch Context**
```bash
knhk context switch <id>
```

Example:
```bash
knhk context create prod "Production" urn:knhk:schema:enterprise
knhk context switch prod
knhk context current
```

## Command Reference Table

| Noun | Verb | Description | Path |
|------|------|-------------|------|
| boot | init | Initialize Σ and Q | - |
| connect | register | Register connector | - |
| connect | list | List connectors | - |
| cover | define | Define cover | - |
| cover | list | List covers | - |
| admit | delta | Admit delta | - |
| reflex | declare | Declare reflex | Hot/Warm |
| reflex | list | List reflexes | - |
| epoch | create | Create epoch | - |
| epoch | run | Run epoch | Hot/Warm |
| epoch | list | List epochs | - |
| route | install | Install route | - |
| route | list | List routes | - |
| receipt | get | Get receipt | - |
| receipt | merge | Merge receipts | - |
| receipt | list | List receipts | - |
| receipt | verify | Verify receipt | - |
| receipt | show | Show receipt | - |
| pipeline | run | Run pipeline | - |
| pipeline | status | Pipeline status | - |
| metrics | get | Get metrics | - |
| coverage | get | Get coverage | - |
| hook | create | Create hook | - |
| hook | list | List hooks | - |
| hook | eval | Evaluate hook | Hot/Warm |
| hook | show | Show hook | - |
| context | create | Create context | - |
| context | list | List contexts | - |
| context | current | Show current context | - |
| context | switch | Switch context | - |

**Total**: 25 commands

## Troubleshooting

### Configuration Issues

**Problem**: Config file not found
- **Solution**: Config file will be created automatically on first use, or create `~/.knhk/config.toml` manually

**Problem**: Environment variables not working
- **Solution**: Ensure environment variables use `KNHK_` prefix and correct naming pattern (e.g., `KNHK_CONNECTOR_NAME_FIELD`)

**Problem**: Validation errors (max_run_len > 8, tau > 8)
- **Solution**: Ensure guard constraints are met: max_run_len ≤ 8, tau ≤ 8

### Path Routing Issues

**Problem**: CONSTRUCT8 operations timing out
- **Solution**: CONSTRUCT8 is routed to warm path (≤500ms). If timing out, check system load and warm path metrics

**Problem**: Hot path operations exceeding 8 ticks
- **Solution**: Verify data is in L1 cache, check predicate run size ≤ 8, ensure branchless operations

### Connector Issues

**Problem**: Kafka connector not connecting
- **Solution**: Check bootstrap servers are reachable, verify topic exists, check network connectivity

**Problem**: Connector not found
- **Solution**: Ensure connector is registered with `knhk connect register`, check connector name spelling

## Examples

See `examples/` directory for complete working examples:
- `basic-hook/` - Basic hook execution
- `kafka-connector/` - Kafka connector setup
- `etl-pipeline/` - Full ETL pipeline
- `receipt-verification/` - Receipt verification
- `cli-usage/` - CLI usage examples

## See Also

- [Architecture](architecture.md) - System architecture
- [API Reference](api.md) - API documentation
- [v0.5.0 Status](archived/v0.4.0/v0.4.0-status.md) - Release status (v0.5.0 updates in progress)