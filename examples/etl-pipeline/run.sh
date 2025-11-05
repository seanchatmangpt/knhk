#!/bin/bash
# examples/etl-pipeline/run.sh
# ETL pipeline execution example

set -e

echo "=========================================="
echo "ETL Pipeline Example"
echo "=========================================="

# Run pipeline
echo "Running ETL pipeline..."
knhk pipeline run --connectors kafka-example

# Check pipeline status
echo ""
echo "Checking pipeline status..."
knhk pipeline status

echo ""
echo "=========================================="
echo "Pipeline execution complete"
echo "=========================================="
