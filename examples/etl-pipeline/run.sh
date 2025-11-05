#!/bin/bash
# examples/etl-pipeline/run.sh

set -e

echo "=========================================="
echo "KNHK ETL Pipeline Example"
echo "=========================================="

# Initialize system
knhk boot init schema.ttl invariants.sparql || echo "⚠ Already initialized"

# Register connector
knhk connect register test-connector urn:knhk:schema:default file:///tmp/triples.ttl || echo "⚠ Connector exists"

# Create hook
knhk hook create test-hook ASK_SP 0xC0FFEE 0 8 || echo "⚠ Hook exists"

# Run pipeline
echo "Running pipeline..."
knhk pipeline run --connectors test-connector

# Check status
echo "Pipeline status:"
knhk pipeline status

echo ""
echo "=========================================="
echo "Pipeline complete!"
echo "=========================================="

