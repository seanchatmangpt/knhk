#!/bin/bash
# Weaver Live-Check Script for Workflow Engine
#
# This script runs Weaver live-check validation for the entire workflow engine.
# It starts Weaver, runs workflow operations, and generates a validation report.
#
# Usage:
#   ./scripts/workflow-engine-weaver-check.sh
#
# Environment Variables:
#   REGISTRY_PATH - Path to Weaver registry (default: ./registry)
#   OTLP_PORT - OTLP gRPC port (default: 4317)
#   ADMIN_PORT - Weaver admin port (default: 8080)
#   WORKFLOW_EXAMPLE - Path to workflow example file (optional)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REGISTRY_PATH="${REGISTRY_PATH:-./registry}"
OTLP_PORT="${OTLP_PORT:-4317}"
ADMIN_PORT="${ADMIN_PORT:-8080}"
WORKFLOW_EXAMPLE="${WORKFLOW_EXAMPLE:-rust/knhk-workflow-engine/examples/simple-sequence.ttl}"
WEAVER_REPORTS_DIR="./weaver-reports"

# Check if Weaver is installed
echo -e "${BLUE}🔍 Checking Weaver installation...${NC}"
if ! command -v weaver &> /dev/null; then
    echo -e "${RED}❌ Weaver not found in PATH${NC}"
    echo -e "${YELLOW}   Install with: cargo install weaver${NC}"
    echo -e "${YELLOW}   Or run: ./scripts/install-weaver.sh${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Weaver found: $(weaver --version 2>&1 | head -n1)${NC}"

# Check if registry exists
if [ ! -d "$REGISTRY_PATH" ]; then
    echo -e "${RED}❌ Registry path does not exist: $REGISTRY_PATH${NC}"
    exit 1
fi

# Check if workflow-engine binary exists
echo -e "${BLUE}🔍 Checking workflow-engine binary...${NC}"
if ! cargo build --bin knhk-workflow --manifest-path rust/knhk-workflow-engine/Cargo.toml &> /dev/null; then
    echo -e "${YELLOW}⚠️  Building workflow-engine...${NC}"
    cargo build --bin knhk-workflow --manifest-path rust/knhk-workflow-engine/Cargo.toml
fi

echo -e "${GREEN}✅ Workflow-engine binary ready${NC}"

# Create weaver reports directory
mkdir -p "$WEAVER_REPORTS_DIR"

# Start Weaver live-check in background
echo -e "${BLUE}🚀 Starting Weaver live-check...${NC}"
cargo run --bin knhk-workflow --manifest-path rust/knhk-workflow-engine/Cargo.toml -- \
    weaver-check \
    --registry "$REGISTRY_PATH" \
    --otlp-port "$OTLP_PORT" \
    --admin-port "$ADMIN_PORT" \
    --enable &
WEAVER_PID=$!

# Wait for Weaver to start
sleep 2

# Check if Weaver is running
if ! kill -0 $WEAVER_PID 2>/dev/null; then
    echo -e "${RED}❌ Weaver failed to start${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Weaver live-check started (PID: $WEAVER_PID)${NC}"
echo -e "${BLUE}📡 OTLP endpoint: http://127.0.0.1:$OTLP_PORT${NC}"

# Set OTLP endpoint for telemetry export
export OTEL_EXPORTER_OTLP_ENDPOINT="http://127.0.0.1:$OTLP_PORT"
export OTEL_SERVICE_NAME="knhk-workflow-engine"
export OTEL_SERVICE_VERSION="1.0.0"

# Run workflow operations to generate telemetry
echo -e "${BLUE}📊 Running workflow operations to generate telemetry...${NC}"

# Create temporary state store
TEMP_STORE=$(mktemp -d)
trap "rm -rf $TEMP_STORE" EXIT

# Run workflow example if it exists
if [ -f "$WORKFLOW_EXAMPLE" ]; then
    echo -e "${BLUE}   Registering workflow from $WORKFLOW_EXAMPLE...${NC}"
    cargo run --bin knhk-workflow --manifest-path rust/knhk-workflow-engine/Cargo.toml -- \
        --state-store "$TEMP_STORE" \
        register --file "$WORKFLOW_EXAMPLE" || true
fi

# Run the Weaver live-check example
echo -e "${BLUE}   Running Weaver live-check example...${NC}"
cargo run --example workflow_weaver_livecheck --manifest-path rust/knhk-workflow-engine/Cargo.toml || true

# Wait a bit for telemetry to be exported
sleep 2

# Stop Weaver
echo -e "${BLUE}🛑 Stopping Weaver live-check...${NC}"
kill $WEAVER_PID 2>/dev/null || true
wait $WEAVER_PID 2>/dev/null || true

# Check for validation report
if [ -d "$WEAVER_REPORTS_DIR" ] && [ "$(ls -A $WEAVER_REPORTS_DIR 2>/dev/null)" ]; then
    echo -e "${GREEN}✅ Validation report generated in $WEAVER_REPORTS_DIR${NC}"
    echo -e "${BLUE}📄 Report files:${NC}"
    ls -lh "$WEAVER_REPORTS_DIR" | tail -n +2
else
    echo -e "${YELLOW}⚠️  No validation report found in $WEAVER_REPORTS_DIR${NC}"
fi

echo -e "${GREEN}✅ Weaver live-check complete${NC}"

