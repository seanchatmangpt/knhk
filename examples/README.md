# KNHK Examples

This directory contains working examples for common KNHK use cases.

## Examples

### basic-hook
Basic hook execution example demonstrating hot path and warm path operations.

### kafka-connector
Kafka connector setup and configuration example.

### etl-pipeline
Complete ETL pipeline example showing all stages: Ingest → Transform → Load → Reflex → Emit.

### receipt-verification
Receipt verification and merging example.

### cli-usage
CLI usage examples demonstrating all 25 commands.

## Running Examples

Each example directory contains a README.md with specific instructions. Generally:

```bash
cd examples/<example-name>
./run.sh  # or follow README instructions
```

## Prerequisites

- KNHK CLI installed (`knhk` command available)
- Configuration file set up (`~/.knhk/config.toml`)
- Required services running (Kafka, etc.) for integration examples
