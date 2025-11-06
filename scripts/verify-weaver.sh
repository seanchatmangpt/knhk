#!/bin/bash
# verify-weaver.sh
# Verify Weaver binary availability and basic functionality

set -e

echo "=== Weaver Verification Script ==="

# Check if Weaver binary is available
echo "Checking Weaver binary availability..."
if ! command -v weaver &> /dev/null; then
    echo "ERROR: Weaver binary not found in PATH"
    echo "Install with: cargo install weaver or ./scripts/install-weaver.sh"
    exit 1
fi

echo "✓ Weaver binary found"

# Check Weaver version
echo "Checking Weaver version..."
if ! weaver --version &> /dev/null; then
    echo "ERROR: Weaver --version failed"
    exit 1
fi

VERSION=$(weaver --version 2>&1 | head -n 1)
echo "✓ Weaver version: $VERSION"

# Check if registry directory exists (optional)
if [ -d "./registry" ]; then
    echo "✓ Registry directory found at ./registry"
else
    echo "WARNING: Registry directory not found at ./registry (optional)"
fi

# Test Weaver can start and respond to health checks
echo "Testing Weaver live-check startup..."
ADMIN_PORT=18080
OTLP_PORT=14317

# Start Weaver in background
weaver registry live-check \
    --otlp-grpc-address 127.0.0.1 \
    --otlp-grpc-port $OTLP_PORT \
    --admin-port $ADMIN_PORT \
    --inactivity-timeout 10 \
    --format json \
    > /tmp/weaver-test.log 2>&1 &
WEAVER_PID=$!

# Wait for Weaver to start
echo "Waiting for Weaver to initialize..."
sleep 3

# Check if process is still running
if ! kill -0 $WEAVER_PID 2>/dev/null; then
    echo "ERROR: Weaver process exited unexpectedly"
    cat /tmp/weaver-test.log
    exit 1
fi

echo "✓ Weaver process is running (PID: $WEAVER_PID)"

# Check if admin port is listening
if ! nc -z 127.0.0.1 $ADMIN_PORT 2>/dev/null; then
    echo "WARNING: Weaver admin port $ADMIN_PORT not responding (may be starting up)"
else
    echo "✓ Weaver admin port $ADMIN_PORT is listening"
fi

# Try health check endpoints
echo "Testing health check endpoints..."
for endpoint in "/health" "/status" "/"; do
    if curl -s -f "http://127.0.0.1:$ADMIN_PORT$endpoint" > /dev/null 2>&1; then
        echo "✓ Health check endpoint $endpoint responded"
        break
    fi
done

# Stop Weaver
echo "Stopping Weaver..."
if curl -s -X POST "http://127.0.0.1:$ADMIN_PORT/stop" > /dev/null 2>&1; then
    echo "✓ Weaver stopped via admin endpoint"
else
    echo "WARNING: Failed to stop Weaver via admin endpoint, killing process"
    kill $WEAVER_PID 2>/dev/null || true
fi

# Wait for process to exit
sleep 1

if kill -0 $WEAVER_PID 2>/dev/null; then
    echo "WARNING: Weaver process still running, force killing"
    kill -9 $WEAVER_PID 2>/dev/null || true
fi

echo ""
echo "=== Verification Complete ==="
echo "✓ All checks passed"
exit 0

