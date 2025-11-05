#!/bin/bash
# examples/kafka-connector/setup.sh
# Kafka connector setup example

set -e

echo "=========================================="
echo "Kafka Connector Example"
echo "=========================================="

# Register Kafka connector
echo "Registering Kafka connector..."
knhk connect register kafka-example urn:knhk:schema:example kafka://localhost:9092/triples

# List connectors
echo ""
echo "Listing connectors..."
knhk connect list

echo ""
echo "=========================================="
echo "Connector setup complete"
echo "=========================================="
echo ""
echo "Note: Ensure Kafka is running on localhost:9092"
echo "      and topic 'triples' exists"
