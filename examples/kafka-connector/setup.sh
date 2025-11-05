#!/bin/bash
# examples/kafka-connector/setup.sh

set -e

echo "Setting up Kafka connector configuration..."

mkdir -p ~/.knhk

cat > ~/.knhk/config.toml <<EOF
[connectors.kafka-prod]
type = "kafka"
bootstrap_servers = ["localhost:9092"]
topic = "triples"
schema = "urn:knhk:schema:default"
max_run_len = 8
max_batch_size = 1000
EOF

echo "✓ Configuration created at ~/.knhk/config.toml"
echo "✓ Kafka connector configured"

