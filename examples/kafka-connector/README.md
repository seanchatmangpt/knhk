# Kafka Connector Example

This example demonstrates how to set up and use a Kafka connector for data ingestion.

## Overview

The Kafka connector allows KNHK to ingest RDF triples from Kafka topics. This example shows:
1. Setting up Kafka connector configuration
2. Registering the connector
3. Running the pipeline with Kafka input

## Prerequisites

- Kafka broker running (default: `localhost:9092`)
- Kafka topic created (default: `triples`)
- RDF/Turtle data producer writing to Kafka topic

## Configuration

### TOML Configuration (`config.toml`)

```toml
[connectors.kafka-prod]
type = "kafka"
bootstrap_servers = ["localhost:9092"]
topic = "triples"
schema = "urn:knhk:schema:default"
max_run_len = 8
max_batch_size = 1000
```

### Environment Variables

```bash
export KNHK_CONNECTOR_KAFKA_BOOTSTRAP_SERVERS=localhost:9092
export KNHK_CONNECTOR_KAFKA_TOPIC=triples
export KNHK_CONNECTOR_KAFKA_MAX_RUN_LEN=8
```

## Usage

### 1. Register Connector

```bash
knhk connect register kafka-prod urn:knhk:schema:default kafka://localhost:9092/triples
```

### 2. Verify Connector

```bash
knhk connect list
```

### 3. Run Pipeline

```bash
knhk pipeline run --connectors kafka-prod
```

## Running the Example

```bash
# Start Kafka (if not running)
# docker run -p 9092:9092 apache/kafka:latest

# Make script executable
chmod +x setup.sh

# Run setup
./setup.sh

# Run pipeline
./run.sh
```

## Expected Output

```
Registering connector: kafka-prod
  ✓ Schema: urn:knhk:schema:default
  ✓ Source: kafka://localhost:9092/triples
✓ Connector registered

Running pipeline with Kafka connector...
Pipeline executed successfully
  Actions sent: 5
  Receipts written: 5
```

## Files

- `config.toml` - Connector configuration
- `setup.sh` - Setup script
- `run.sh` - Execution script
- `README.md` - This file

