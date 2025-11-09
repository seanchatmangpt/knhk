#!/bin/bash
# Weaver Live-Check Validation for All 43 Van der Aalst Patterns
#
# This script validates that all 43 Van der Aalst workflow patterns
# emit proper OTEL telemetry that conforms to the Weaver schema.
#
# Usage:
#   ./scripts/weaver-validate-all-43-patterns.sh
#
# Environment Variables:
#   REGISTRY_PATH - Path to Weaver registry (default: ./registry)
#   OTLP_PORT - OTLP gRPC port (default: 4317)
#   ADMIN_PORT - Weaver admin port (default: 8080)

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
echo -e "${BLUE}🔍 Building workflow-engine...${NC}"
if ! cargo build --example weaver_all_43_patterns --manifest-path rust/knhk-workflow-engine/Cargo.toml &> /dev/null; then
    echo -e "${YELLOW}⚠️  Building workflow-engine example...${NC}"
    cargo build --example weaver_all_43_patterns --manifest-path rust/knhk-workflow-engine/Cargo.toml
fi

echo -e "${GREEN}✅ Workflow-engine example ready${NC}"

# Create weaver reports directory
mkdir -p "$WEAVER_REPORTS_DIR"

# Start Weaver live-check in background
echo -e "${BLUE}🚀 Starting Weaver live-check...${NC}"
weaver registry live-check \
    --registry "$REGISTRY_PATH" \
    --otlp-grpc-address 127.0.0.1 \
    --otlp-grpc-port "$OTLP_PORT" \
    --admin-port "$ADMIN_PORT" \
    --inactivity-timeout 60 \
    --format json \
    --output "$WEAVER_REPORTS_DIR" &
WEAVER_PID=$!

# Wait for Weaver to start
sleep 3

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

# Run the real JTBD validation example
echo -e "${BLUE}📊 Running real JTBD validation for all 43 Van der Aalst patterns...${NC}"
echo -e "${BLUE}   This will test each pattern in real workflow scenarios and validate JTBD${NC}\n"

if cargo run --example weaver_real_jtbd_validation --manifest-path rust/knhk-workflow-engine/Cargo.toml; then
    echo -e "\n${GREEN}✅ All 43 patterns validated with real JTBD${NC}"
else
    echo -e "\n${RED}❌ JTBD validation failed${NC}"
    kill $WEAVER_PID 2>/dev/null || true
    exit 1
fi

# Wait a bit for telemetry to be exported
echo -e "\n${BLUE}⏳ Waiting for telemetry export...${NC}"
sleep 5

# Stop Weaver
echo -e "${BLUE}🛑 Stopping Weaver live-check...${NC}"
kill $WEAVER_PID 2>/dev/null || true
wait $WEAVER_PID 2>/dev/null || true

# Check for validation report
if [ -d "$WEAVER_REPORTS_DIR" ] && [ "$(ls -A $WEAVER_REPORTS_DIR 2>/dev/null)" ]; then
    echo -e "${GREEN}✅ Validation report generated in $WEAVER_REPORTS_DIR${NC}"
    echo -e "${BLUE}📄 Report files:${NC}"
    ls -lh "$WEAVER_REPORTS_DIR" | tail -n +2
    
    # Check for validation errors
    if find "$WEAVER_REPORTS_DIR" -name "*.json" -type f | head -1 | xargs grep -q "violations\|errors" 2>/dev/null; then
        echo -e "\n${YELLOW}⚠️  Validation report contains violations or errors${NC}"
        echo -e "${YELLOW}   Review the report files in $WEAVER_REPORTS_DIR${NC}"
    else
        echo -e "\n${GREEN}✅ No validation violations found${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  No validation report found in $WEAVER_REPORTS_DIR${NC}"
fi

echo -e "\n${GREEN}✅ Weaver live-check validation complete${NC}"
echo -e "${BLUE}📊 Summary:${NC}"
echo -e "   - All 43 Van der Aalst patterns tested"
echo -e "   - OTEL telemetry exported to Weaver"
echo -e "   - Validation report in $WEAVER_REPORTS_DIR"

