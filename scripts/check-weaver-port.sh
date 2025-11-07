#!/bin/bash
# Check and resolve Weaver port conflicts
set -e

echo "🔍 Checking port 4318 availability..."

# Check if port 4318 is available
if lsof -i :4318 > /dev/null 2>&1; then
    echo "❌ Port 4318 is in use by:"
    lsof -i :4318 | head -5
    echo ""
    echo "📌 Using alternative port 4319 for Weaver testing"
    export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4319
    export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://localhost:4319/v1/traces
    PORT=4319
    DOCKER_PORT_MAP="4319:4318"
else
    echo "✅ Port 4318 is available"
    export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
    export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://localhost:4318/v1/traces
    PORT=4318
    DOCKER_PORT_MAP="4318:4318"
fi

# Export for downstream scripts
export KNHK_OTEL_PORT=$PORT

echo "📝 Environment configured:"
echo "   OTEL_EXPORTER_OTLP_ENDPOINT=$OTEL_EXPORTER_OTLP_ENDPOINT"
echo "   KNHK_OTEL_PORT=$KNHK_OTEL_PORT"

# Check if collector is already running
if docker ps --filter name=knhk-weaver-otel --format '{{.Names}}' | grep -q knhk-weaver-otel; then
    echo "⚠️  Collector already running, stopping..."
    docker stop knhk-weaver-otel > /dev/null 2>&1 || true
    docker rm knhk-weaver-otel > /dev/null 2>&1 || true
fi

# Start collector on appropriate port using docker-compose
echo "🚀 Starting OTEL collector on port ${PORT}..."
cd "$(dirname "$0")/../tests/integration"
docker-compose -f docker-compose-weaver.yml up -d

# Wait for health check
echo "⏳ Waiting for collector to be healthy..."
for i in {1..30}; do
    if docker inspect knhk-weaver-otel | grep -q '"Status": "healthy"'; then
        echo "✅ OTEL collector is healthy on port ${PORT}"
        exit 0
    fi
    sleep 1
done

echo "❌ Collector failed to become healthy"
docker logs knhk-weaver-otel
exit 1
