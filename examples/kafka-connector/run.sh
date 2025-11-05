#!/bin/bash
# Kafka Connector Pipeline Execution
# Runs ETL pipeline with Kafka connector

set -e

echo "=== Kafka Connector Pipeline Execution ==="
echo ""

# Run pipeline
echo "Running pipeline with Kafka connector..."
knhk pipeline run --connectors kafka-prod

echo ""
echo "Checking pipeline status..."
knhk pipeline status

echo ""
echo "=== Pipeline Complete ==="

