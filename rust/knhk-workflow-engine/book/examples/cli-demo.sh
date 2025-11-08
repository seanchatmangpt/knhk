#!/bin/bash
# Example usage of knhk-workflow CLI

set -e

echo "=== KNHK Workflow Engine CLI Examples ==="
echo ""

# Change to workflow engine directory
cd "$(dirname "$0")/.."

# Build the CLI if needed
echo "Building CLI..."
cargo build --bin knhk-workflow --release 2>&1 | tail -3

echo ""
echo "=== 1. List all registered patterns ==="
cargo run --release --bin knhk-workflow -- list-patterns || echo "Note: Requires built binary"

echo ""
echo "=== 2. Parse a workflow from Turtle file ==="
echo "Command: cargo run --release --bin knhk-workflow -- parse --file examples/simple-sequence.ttl"
echo "This would parse the workflow and display it as JSON"

echo ""
echo "=== 3. Register a workflow ==="
echo "Command: cargo run --release --bin knhk-workflow -- register --file examples/simple-sequence.ttl"
echo "This would register the workflow in the state store"

echo ""
echo "=== 4. Create a case ==="
echo "Command: cargo run --release --bin knhk-workflow -- create-case <spec-id> --data '{\"key\":\"value\"}'"
echo "This would create a new workflow case"

echo ""
echo "=== 5. Start REST API server ==="
echo "Command: cargo run --release --bin knhk-workflow -- serve --port 8080"
echo "This would start the REST API server on port 8080"

echo ""
echo "=== Available Commands ==="
cargo run --release --bin knhk-workflow -- --help 2>&1 | grep -A 50 "Commands:" || echo "Run: cargo run --release --bin knhk-workflow -- --help"

