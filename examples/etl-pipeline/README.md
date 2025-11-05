# ETL Pipeline Example

This example demonstrates a complete ETL pipeline execution:
1. Ingest - Data ingestion from connectors
2. Transform - Schema validation and IRI hashing
3. Load - SoA array preparation
4. Reflex - Hook execution (hot/warm path)
5. Emit - Receipt writing and action routing

## Overview

The ETL pipeline processes RDF triples through five stages:
- **Ingest**: Poll connectors for new data
- **Transform**: Validate against schema (Σ), hash IRIs
- **Load**: Group by predicate, prepare SoA arrays
- **Reflex**: Execute hooks (hot path ≤8 ticks, warm path ≤500ms)
- **Emit**: Write receipts to lockchain, send actions to downstream APIs

## Configuration

### Pipeline Configuration (`pipeline-config.toml`)

```toml
[knhk]
version = "0.5.0"
context = "production"

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

[routes.webhook]
kind = "webhook"
target = "https://api.example.com/webhook"
encode = "json-ld"

[routes.kafka-actions]
kind = "kafka"
target = "kafka://localhost:9092/actions"
encode = "json-ld"
```

## Usage

### 1. Initialize System

```bash
knhk boot init schema.ttl invariants.sparql
```

### 2. Register Connectors

```bash
knhk connect register kafka-prod urn:knhk:schema:enterprise kafka://localhost:9092/triples
```

### 3. Define Covers

```bash
knhk cover define "SELECT ?s ?p ?o WHERE { ?s ?p ?o }" "max_run_len 8"
```

### 4. Declare Reflexes

```bash
knhk reflex declare check-count ASK_SP 0xC0FFEE 0 8
```

### 5. Create Epoch

```bash
knhk epoch create epoch1 8 "check-count"
```

### 6. Install Routes

```bash
knhk route install webhook webhook https://api.example.com/webhook
knhk route install kafka-actions kafka kafka://localhost:9092/actions
```

### 7. Run Pipeline

```bash
knhk pipeline run --connectors kafka-prod --schema urn:knhk:schema:enterprise
```

## Running the Example

```bash
# Make script executable
chmod +x run.sh

# Run example
./run.sh
```

## Expected Output

```
Running pipeline...
Ingest: 100 triples loaded
Transform: 100 triples validated
Load: 12 predicate runs created
Reflex: 12 hooks executed (max ticks: 6)
Emit: 12 receipts written, 12 actions sent

Pipeline executed successfully
  Actions sent: 12
  Receipts written: 12
```

## Files

- `pipeline-config.toml` - Pipeline configuration
- `schema.ttl` - Schema definition (Σ)
- `invariants.sparql` - Invariant queries (Q)
- `run.sh` - Execution script
- `README.md` - This file

