#!/bin/bash
# ETL Pipeline Execution Example
# Demonstrates complete ETL pipeline execution

set -e

echo "=== ETL Pipeline Execution Example ==="
echo ""

# Initialize system
echo "Initializing system..."
knhk boot init schema.ttl invariants.sparql

# Register connector
echo ""
echo "Registering connector..."
knhk connect register kafka-prod urn:knhk:schema:enterprise kafka://localhost:9092/triples

# Define cover
echo ""
echo "Defining cover..."
knhk cover define "SELECT ?s ?p ?o WHERE { ?s ?p ?o }" "max_run_len 8"

# Declare reflex
echo ""
echo "Declaring reflex..."
knhk reflex declare check-count ASK_SP 0xC0FFEE 0 8

# Create epoch
echo ""
echo "Creating epoch..."
knhk epoch create epoch1 8 "check-count"

# Install routes
echo ""
echo "Installing routes..."
knhk route install webhook webhook https://api.example.com/webhook
knhk route install kafka-actions kafka kafka://localhost:9092/actions

# Run pipeline
echo ""
echo "Running pipeline..."
knhk pipeline run --connectors kafka-prod --schema urn:knhk:schema:enterprise

# Check status
echo ""
echo "Checking pipeline status..."
knhk pipeline status

echo ""
echo "=== Pipeline Complete ==="

