# Kafka Connector Example

This example demonstrates Kafka connector setup and configuration.

## Files

- `config.toml` - Example Kafka connector configuration
- `setup.sh` - Setup script
- `README.md` - This file

## Prerequisites

- Kafka running on localhost:9092
- Topic `triples` created

## Usage

```bash
./setup.sh
```

## Configuration

Example `config.toml`:
```toml
[connectors.kafka-example]
type = "kafka"
bootstrap_servers = ["localhost:9092"]
topic = "triples"
schema = "urn:knhk:schema:example"
max_run_len = 8
max_batch_size = 1000
```

## Steps

1. Register Kafka connector
2. Verify connector registration
3. Test connector connectivity
