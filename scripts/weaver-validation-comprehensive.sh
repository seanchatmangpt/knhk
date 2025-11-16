#!/bin/bash
# Comprehensive Weaver Validation Script
# This script validates BOTH schema AND runtime telemetry
# Following KNHK principle: "Only Weaver validation is truth"

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "=========================================="
echo "KNHK Comprehensive Weaver Validation"
echo "=========================================="
echo ""
echo "This validates the ONLY source of truth:"
echo "1. Schema validation (weaver registry check)"
echo "2. Runtime validation (weaver registry live-check)"
echo ""

# Track validation results
SCHEMA_VALID=false
RUNTIME_VALID=false
VALIDATION_ERRORS=""

# ============================================
# PHASE 1: Install Weaver (if needed)
# ============================================
echo -e "${BLUE}=== Phase 1: Weaver Installation ===${NC}"
echo ""

if ! command -v weaver &> /dev/null; then
    echo "Weaver not found. Installing..."
    if [ -f "$SCRIPT_DIR/install-weaver.sh" ]; then
        bash "$SCRIPT_DIR/install-weaver.sh"
    else
        echo -e "${RED}❌ install-weaver.sh not found${NC}"
        echo "Install manually: curl --proto '=https' --tlsv1.2 -sSfL https://github.com/open-telemetry/weaver/releases/latest/download/install.sh | sh"
        exit 1
    fi
fi

WEAVER_VERSION=$(weaver --version 2>&1 || echo "unknown")
echo -e "${GREEN}✅ Weaver installed: $WEAVER_VERSION${NC}"
echo ""

# ============================================
# PHASE 2: Schema Validation (MANDATORY)
# ============================================
echo -e "${BLUE}=== Phase 2: Schema Validation (MANDATORY - Proves schema is well-formed) ===${NC}"
echo ""

cd "$PROJECT_ROOT"

echo "Validating registry schemas..."
echo "Registry files:"
ls -1 registry/*.yaml | sed 's/^/  - /'
echo ""

if weaver registry check -r registry/; then
    echo -e "${GREEN}✅ SCHEMA VALIDATION PASSED${NC}"
    echo "   All registry files are well-formed"
    SCHEMA_VALID=true
else
    echo -e "${RED}❌ SCHEMA VALIDATION FAILED${NC}"
    VALIDATION_ERRORS="$VALIDATION_ERRORS\n❌ Schema validation failed"
    echo ""
    echo "Schema validation is MANDATORY. Fix schema errors before proceeding."
    exit 1
fi
echo ""

# ============================================
# PHASE 3: Build Application with Telemetry
# ============================================
echo -e "${BLUE}=== Phase 3: Build Application with Telemetry ===${NC}"
echo ""

echo "Building Rust workspace with OTEL features..."
if cargo build --workspace --features otel,tokio-runtime 2>&1 | tee /tmp/knhk-build.log | tail -20; then
    echo -e "${GREEN}✅ Build succeeded${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    echo "See /tmp/knhk-build.log for details"
    VALIDATION_ERRORS="$VALIDATION_ERRORS\n❌ Build failed - cannot validate runtime"
    # Continue anyway to show schema results
fi
echo ""

# ============================================
# PHASE 4: Setup OTEL Collector
# ============================================
echo -e "${BLUE}=== Phase 4: Setup OTEL Collector ===${NC}"
echo ""

# Check for port conflicts
OTEL_PORT=4318
if lsof -i :4318 > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Port 4318 occupied (likely Docker Desktop OTLP analytics)${NC}"
    echo "   Using alternative port 4319"
    OTEL_PORT=4319
fi

echo "Starting OTEL Collector on port $OTEL_PORT..."

# Stop existing collector
docker stop knhk-weaver-otel 2>/dev/null || true
docker rm knhk-weaver-otel 2>/dev/null || true

# Create collector config if not exists
COLLECTOR_CONFIG="$PROJECT_ROOT/tests/otel-collector-config.yaml"
if [ ! -f "$COLLECTOR_CONFIG" ]; then
    echo "Creating OTEL collector config at $COLLECTOR_CONFIG"
    mkdir -p "$(dirname "$COLLECTOR_CONFIG")"
    cat > "$COLLECTOR_CONFIG" << 'EOF'
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 1s
    send_batch_size: 1024

exporters:
  logging:
    loglevel: debug
    sampling_initial: 100
    sampling_thereafter: 100
  file:
    path: /tmp/otel-export.json

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [logging, file]
    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [logging, file]
    logs:
      receivers: [otlp]
      processors: [batch]
      exporters: [logging, file]
EOF
fi

# Start collector
docker run -d \
  --name knhk-weaver-otel \
  -p 4317:4317 \
  -p $OTEL_PORT:4318 \
  -v "$COLLECTOR_CONFIG:/etc/otelcol/config.yaml" \
  -v /tmp:/tmp \
  otel/opentelemetry-collector:latest \
  --config=/etc/otelcol/config.yaml

echo "Waiting for collector startup..."
sleep 5

if docker ps | grep -q knhk-weaver-otel; then
    echo -e "${GREEN}✅ OTEL Collector running on port $OTEL_PORT${NC}"
else
    echo -e "${RED}❌ OTEL Collector failed to start${NC}"
    docker logs knhk-weaver-otel 2>&1 | tail -20
    VALIDATION_ERRORS="$VALIDATION_ERRORS\n❌ OTEL Collector failed to start"
fi
echo ""

# ============================================
# PHASE 5: Execute Application to Emit Telemetry
# ============================================
echo -e "${BLUE}=== Phase 5: Execute Application (CRITICAL - Must emit telemetry) ===${NC}"
echo ""

export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:$OTEL_PORT"
export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="http://localhost:$OTEL_PORT/v1/traces"
export OTEL_EXPORTER_OTLP_METRICS_ENDPOINT="http://localhost:$OTEL_PORT/v1/metrics"
export OTEL_SERVICE_NAME="knhk-validation"

echo "CRITICAL: Application must execute with REAL arguments (not just --help)"
echo ""

# Try to run actual application commands
echo "Attempting to execute KNHK commands with real arguments..."
echo ""

# Test 1: Run a simple workflow if CLI exists
if [ -f "$PROJECT_ROOT/target/release/knhk" ]; then
    echo "Test 1: Running 'knhk --version'..."
    timeout 5 "$PROJECT_ROOT/target/release/knhk" --version || true
    echo ""

    echo "Test 2: Running workflow execution (if examples exist)..."
    if [ -d "$PROJECT_ROOT/examples" ]; then
        # Try to execute a workflow example
        timeout 10 "$PROJECT_ROOT/target/release/knhk" workflow execute examples/*.ttl 2>&1 || true
    fi
else
    echo -e "${YELLOW}⚠️  KNHK CLI not built yet${NC}"
    echo "   Building CLI..."
    cd "$PROJECT_ROOT/rust/knhk-cli" && cargo build --release
fi
echo ""

# Test 2: Run integration tests that emit telemetry
echo "Test 3: Running integration tests..."
if [ -d "$PROJECT_ROOT/rust/knhk-integration-tests" ]; then
    cd "$PROJECT_ROOT/rust/knhk-integration-tests"
    timeout 30 cargo test --features otel 2>&1 | tail -20 || true
else
    echo -e "${YELLOW}⚠️  No integration tests found${NC}"
fi
echo ""

echo "Waiting for telemetry to be exported..."
sleep 5

# Check if telemetry was received
echo "Checking collector logs for received telemetry..."
TELEMETRY_COUNT=$(docker logs knhk-weaver-otel 2>&1 | grep -c "Span\|Metric\|LogRecord" || echo "0")
if [ "$TELEMETRY_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✅ Telemetry received: $TELEMETRY_COUNT events${NC}"
    docker logs knhk-weaver-otel 2>&1 | grep "Span\|Metric" | head -10
else
    echo -e "${RED}❌ NO TELEMETRY RECEIVED${NC}"
    echo "   This means:"
    echo "   1. Application didn't execute (check build errors)"
    echo "   2. Application executed but didn't emit telemetry (missing instrumentation)"
    echo "   3. Telemetry emitted but collector didn't receive it (network issue)"
    echo ""
    echo "Collector logs:"
    docker logs knhk-weaver-otel 2>&1 | tail -30
    VALIDATION_ERRORS="$VALIDATION_ERRORS\n❌ No telemetry received"
fi
echo ""

# ============================================
# PHASE 6: Runtime Validation (SOURCE OF TRUTH)
# ============================================
echo -e "${BLUE}=== Phase 6: Runtime Validation (CRITICAL - Source of Truth) ===${NC}"
echo ""

echo "Running Weaver live-check..."
echo "This validates that ACTUAL runtime telemetry matches schema."
echo ""

if [ "$OTEL_PORT" != "4318" ]; then
    echo -e "${YELLOW}⚠️  Weaver live-check requires port 4318${NC}"
    echo "   Current collector on port $OTEL_PORT"
    echo "   Restarting collector on port 4318 for live-check..."

    docker stop knhk-weaver-otel 2>/dev/null || true
    docker run -d \
      --name knhk-weaver-otel \
      -p 4317:4317 \
      -p 4318:4318 \
      -v "$COLLECTOR_CONFIG:/etc/otelcol/config.yaml" \
      -v /tmp:/tmp \
      otel/opentelemetry-collector:latest \
      --config=/etc/otelcol/config.yaml

    sleep 5
fi

cd "$PROJECT_ROOT"

if weaver registry live-check --registry registry/ 2>&1 | tee /tmp/weaver-live-check.log; then
    echo -e "${GREEN}✅ RUNTIME VALIDATION PASSED${NC}"
    echo "   Actual telemetry conforms to schema"
    RUNTIME_VALID=true
else
    echo -e "${RED}❌ RUNTIME VALIDATION FAILED${NC}"
    echo "   Telemetry does NOT match schema"
    echo ""
    echo "Common causes:"
    echo "1. Schema defines spans/metrics that code doesn't emit"
    echo "2. Code emits spans/metrics not defined in schema"
    echo "3. Attribute names/types don't match schema"
    echo "4. No telemetry emitted (application didn't run)"
    echo ""
    echo "Live-check output:"
    cat /tmp/weaver-live-check.log
    VALIDATION_ERRORS="$VALIDATION_ERRORS\n❌ Runtime validation failed"
fi
echo ""

# ============================================
# PHASE 7: Cleanup
# ============================================
echo -e "${BLUE}=== Phase 7: Cleanup ===${NC}"
echo ""

echo "Stopping OTEL Collector..."
docker stop knhk-weaver-otel 2>/dev/null || true
docker rm knhk-weaver-otel 2>/dev/null || true
echo -e "${GREEN}✅ Cleanup complete${NC}"
echo ""

# ============================================
# FINAL VALIDATION SUMMARY
# ============================================
echo "=========================================="
echo "Weaver Validation Summary"
echo "=========================================="
echo ""

if [ "$SCHEMA_VALID" = true ] && [ "$RUNTIME_VALID" = true ]; then
    echo -e "${GREEN}✅✅✅ WEAVER VALIDATION PASSED ✅✅✅${NC}"
    echo ""
    echo "Both validations succeeded:"
    echo "  ✅ Schema validation: PASSED (schema is well-formed)"
    echo "  ✅ Runtime validation: PASSED (telemetry matches schema)"
    echo ""
    echo -e "${GREEN}This is the ONLY proof that features actually work!${NC}"
    echo ""
    exit 0
elif [ "$SCHEMA_VALID" = true ]; then
    echo -e "${YELLOW}⚠️  PARTIAL VALIDATION${NC}"
    echo ""
    echo "Results:"
    echo "  ✅ Schema validation: PASSED"
    echo "  ❌ Runtime validation: FAILED or INCOMPLETE"
    echo ""
    echo "Schema is valid, but runtime telemetry doesn't match."
    echo "This means:"
    echo "  - Code may not be emitting telemetry"
    echo "  - Code emits different telemetry than schema declares"
    echo "  - Application didn't execute properly"
    echo ""
    echo -e "${YELLOW}Fix runtime issues and re-run validation.${NC}"
    exit 1
else
    echo -e "${RED}❌❌❌ WEAVER VALIDATION FAILED ❌❌❌${NC}"
    echo ""
    echo "Critical errors:"
    echo -e "$VALIDATION_ERRORS"
    echo ""
    echo -e "${RED}Fix schema errors before proceeding.${NC}"
    exit 1
fi
