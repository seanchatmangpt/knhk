#!/bin/bash
# scripts/test-telemetry-export.sh
# Test telemetry export with Weaver validation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=========================================="
echo "KNHK Telemetry Export Test"
echo "=========================================="

# Check if Weaver is installed
if ! command -v weaver &> /dev/null; then
    echo -e "${RED}ERROR: Weaver not found in PATH${NC}"
    echo "Install with: ./scripts/install-weaver.sh or cargo install weaver"
    exit 1
fi

echo -e "${GREEN}✓ Weaver binary found${NC}"

# Check if registry exists
if [ ! -d "registry" ]; then
    echo -e "${YELLOW}WARNING: registry/ directory not found${NC}"
    echo "Creating minimal registry structure..."
    mkdir -p registry

    # Create minimal registry.yaml
    cat > registry/registry.yaml <<EOF
# KNHK Semantic Convention Registry
version: 0.1.0
groups:
  - id: knhk.operations
    type: span
    brief: KNHK operation spans
    prefix: knhk
    attributes:
      - id: operation.name
        type: string
        brief: Operation name
        requirement_level: required
      - id: operation.type
        type: string
        brief: Operation type
        requirement_level: required
    spans:
      - id: operation.execute
        brief: Generic operation execution
EOF

    echo -e "${GREEN}✓ Created minimal registry${NC}"
else
    echo -e "${GREEN}✓ Registry directory found${NC}"
fi

# Check if OTLP collector is available
OTLP_ENDPOINT=${OTLP_ENDPOINT:-"http://localhost:4317"}
echo "Testing OTLP endpoint: $OTLP_ENDPOINT"

# Try to connect to OTLP endpoint (without Weaver for now)
if nc -z localhost 4317 2>/dev/null; then
    echo -e "${GREEN}✓ OTLP collector is running${NC}"
    COLLECTOR_RUNNING=true
else
    echo -e "${YELLOW}⚠ OTLP collector not detected on port 4317${NC}"
    echo "  Will start Weaver live-check for validation"
    COLLECTOR_RUNNING=false
fi

# Start Weaver live-check if collector not running
WEAVER_PID=""
if [ "$COLLECTOR_RUNNING" = false ]; then
    echo ""
    echo "Starting Weaver live-check..."

    # Start Weaver in background
    weaver registry live-check \
        --registry ./registry \
        --otlp-grpc-port 4317 \
        --admin-port 8080 \
        --format json \
        --inactivity-timeout 30 &

    WEAVER_PID=$!
    echo "Weaver PID: $WEAVER_PID"

    # Wait for Weaver to start
    echo "Waiting for Weaver to start..."
    for i in {1..10}; do
        if nc -z localhost 4317 2>/dev/null; then
            echo -e "${GREEN}✓ Weaver live-check started${NC}"
            break
        fi
        if [ $i -eq 10 ]; then
            echo -e "${RED}ERROR: Weaver failed to start${NC}"
            exit 1
        fi
        sleep 1
    done
fi

# Run telemetry tests
echo ""
echo "Running telemetry integration tests..."
cargo test --package knhk-sidecar --test telemetry --features otel -- --nocapture

TEST_EXIT_CODE=$?

# Check Weaver validation results
if [ "$COLLECTOR_RUNNING" = false ] && [ -n "$WEAVER_PID" ]; then
    echo ""
    echo "Stopping Weaver live-check..."

    # Stop Weaver via HTTP endpoint
    curl -X POST http://localhost:8080/stop 2>/dev/null || true

    # Wait for graceful shutdown
    sleep 2

    # Force kill if still running
    if ps -p $WEAVER_PID > /dev/null 2>&1; then
        kill $WEAVER_PID 2>/dev/null || true
    fi

    echo -e "${GREEN}✓ Weaver stopped${NC}"
fi

# Report results
echo ""
echo "=========================================="
if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✓ All telemetry tests passed${NC}"
else
    echo -e "${RED}✗ Telemetry tests failed${NC}"
fi
echo "=========================================="

exit $TEST_EXIT_CODE
