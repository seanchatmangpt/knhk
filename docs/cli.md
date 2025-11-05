# KNHK CLI Guide

**Version**: 0.4.0  
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

### Hook - Knowledge Hook Operations

**Create Hook**
```bash
knhk hook create <name> <op> <pred> <off> <len>
```

**List Hooks**
```bash
knhk hook list
```

**Evaluate Hook**
```bash
knhk hook eval <name>
```

**Show Hook Details**
```bash
knhk hook show <name>
```

Example:
```bash
knhk hook create auth-check ASK_SP 0xC0FFEE 0 8
knhk hook eval auth-check
```

**Note**: CONSTRUCT8 operations are automatically routed to warm path (≤500ms budget).

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
knhk context use <id>
```

Example:
```bash
knhk context create prod1 Production urn:knhk:schema:enterprise
knhk context use prod1
```

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

## Error Handling

All commands return exit codes:
- `0` - Success
- `1` - Error

Errors are displayed to stderr with descriptive messages.

## Commands

## Guard Constraints

All commands enforce guard constraints:
- **max_run_len ≤ 8** - Run length must not exceed 8
- **τ ≤ 8** - Epoch tick budget must not exceed 8
- **Operation validation** - Reflex operations must be in H_hot set

## Command Reference Table

| Noun | Verb | Description | Arguments |
|------|------|-------------|-----------|
| `boot` | `init` | Initialize Σ and Q | `<sigma.ttl> <q.sparql>` |
| `connect` | `register` | Register connector | `<name> <schema> <source>` |
| `connect` | `list` | List connectors | |
| `cover` | `define` | Define cover | `<select> <shard>` |
| `cover` | `list` | List covers | |
| `admit` | `delta` | Admit delta | `<delta_file>` |
| `reflex` | `declare` | Declare reflex | `<name> <op> <pred> <off> <len>` |
| `reflex` | `list` | List reflexes | |
| `epoch` | `create` | Create epoch | `<id> <tau> <lambda>` |
| `epoch` | `run` | Run epoch | `<id>` |
| `epoch` | `list` | List epochs | |
| `route` | `install` | Install route | `<name> <kind> <target>` |
| `route` | `list` | List routes | |
| `receipt` | `get` | Get receipt | `<id>` |
| `receipt` | `merge` | Merge receipts | `<id1,id2,id3>` |
| `receipt` | `list` | List receipts | |
| `receipt` | `verify` | Verify receipt | `<id>` |
| `receipt` | `show` | Show receipt | `<id>` |
| `pipeline` | `run` | Run pipeline | `[--connectors <ids>] [--schema <iri>]` |
| `pipeline` | `status` | Pipeline status | |
| `metrics` | `get` | Get metrics | |
| `coverage` | `get` | Get coverage | |
| `hook` | `create` | Create hook | `<name> <op> <pred> <off> <len>` |
| `hook` | `list` | List hooks | |
| `hook` | `eval` | Evaluate hook | `<name>` |
| `hook` | `show` | Show hook | `<name>` |
| `context` | `create` | Create context | `<id> <name> <schema>` |
| `context` | `list` | List contexts | |
| `context` | `current` | Show current context | |
| `context` | `use` | Switch context | `<id>` |

**Total**: 25 commands across 12 nouns

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

### Hook Execution Examples

```bash
# Create a hook for authorization check
knhk hook create auth-check ASK_SP 0xC0FFEE 0 8

# Create a CONSTRUCT8 hook (routed to warm path)
knhk hook create construct-permissions CONSTRUCT8 0xC0FFEE 0 8

# Evaluate hook
knhk hook eval auth-check

# List all hooks
knhk hook list
```

### Context Management Examples

```bash
# Create production context
knhk context create prod1 Production urn:knhk:schema:enterprise

# Create development context
knhk context create dev1 Development urn:knhk:schema:dev

# Switch to production context
knhk context use prod1

# Show current context
knhk context current

# List all contexts
knhk context list
```

### Receipt Operations Examples

```bash
# Get receipt details
knhk receipt get receipt_1234567890abcdef

# Verify receipt integrity
knhk receipt verify receipt_1234567890abcdef

# Merge multiple receipts
knhk receipt merge receipt_1,receipt_2,receipt_3

# List all receipts
knhk receipt list
```

### Pipeline Execution Examples

```bash
# Run pipeline with default connectors
knhk pipeline run

# Run pipeline with specific connector
knhk pipeline run --connectors kafka-prod

# Run pipeline with custom schema
knhk pipeline run --schema urn:knhk:schema:enterprise

# Check pipeline status
knhk pipeline status

# Get metrics
knhk metrics get

# Get coverage metrics
knhk coverage get
```
