#!/bin/bash
# Kafka Connector Setup Example
# Demonstrates setting up a Kafka connector

set -e

echo "=== Kafka Connector Setup Example ==="
echo ""

# Create config directory if it doesn't exist
mkdir -p ~/.knhk

# Copy config file
echo "Copying configuration..."
cp config.toml ~/.knhk/config.toml

# Register connector
echo ""
echo "Registering connector: kafka-prod"
knhk connect register kafka-prod urn:knhk:schema:default kafka://localhost:9092/triples

echo ""
echo "Listing connectors:"
knhk connect list

echo ""
echo "=== Setup Complete ==="
echo "To run pipeline: ./run.sh"

