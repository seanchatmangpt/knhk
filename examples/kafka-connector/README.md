# Kafka Connector Example

This example demonstrates Kafka connector setup and usage.

## Overview

Shows how to:
1. Configure Kafka connector
2. Register connector
3. Use connector in pipeline

## Files

- `config.toml` - Kafka connector configuration
- `setup.sh` - Setup script
- `README.md` - This file

## Usage

```bash
# Setup Kafka connector
./setup.sh

# Register connector
knhk connect register kafka-prod urn:knhk:schema:default kafka://localhost:9092/triples

# Run pipeline with Kafka connector
knhk pipeline run --connectors kafka-prod
```

## Configuration

See `config.toml` for Kafka connector configuration.

