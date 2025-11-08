#!/bin/bash
# Setup OpenTelemetry infrastructure for E2E tests
#
# This script sets up:
# 1. OTLP collector (Docker)
# 2. Jaeger (tracing backend)
# 3. Prometheus (metrics backend)
# 4. Grafana (visualization)
# 5. Weaver (optional, if available)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
INTEGRATION_DIR="$PROJECT_ROOT/tests/integration"

echo "🔧 Setting up OpenTelemetry infrastructure for E2E tests..."

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

if ! docker info &> /dev/null; then
    echo "❌ Docker daemon is not running. Please start Docker first."
    exit 1
fi

# Check if docker-compose is available
if command -v docker-compose &> /dev/null; then
    COMPOSE_CMD="docker-compose"
elif docker compose version &> /dev/null; then
    COMPOSE_CMD="docker compose"
else
    echo "❌ docker-compose is not installed. Please install docker-compose first."
    exit 1
fi

# Navigate to integration directory
cd "$INTEGRATION_DIR"

# Start OTLP collector and related services
echo "🚀 Starting OTLP collector, Jaeger, Prometheus, and Grafana..."
$COMPOSE_CMD -f docker-compose.otel.yml up -d

# Wait for services to be healthy
echo "⏳ Waiting for services to be healthy..."
sleep 5

# Check if services are running
echo "🔍 Checking service health..."

check_service() {
    local service=$1
    local port=$2
    local max_attempts=30
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        if docker exec "$service" wget --spider -q "http://localhost:$port" 2>/dev/null || \
           curl -s "http://localhost:$port" > /dev/null 2>&1; then
            echo "✅ $service is healthy"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 1
    done

    echo "❌ $service failed to become healthy"
    return 1
}

# Check OTLP collector
if check_service "knhk-otel-collector" "8888"; then
    echo "✅ OTLP collector is ready at http://localhost:4318 (HTTP) and http://localhost:4317 (gRPC)"
else
    echo "⚠️  OTLP collector may not be fully ready yet"
fi

# Check Jaeger
if check_service "knhk-jaeger" "16686"; then
    echo "✅ Jaeger UI is ready at http://localhost:16686"
else
    echo "⚠️  Jaeger may not be fully ready yet"
fi

# Check Prometheus
if check_service "knhk-prometheus" "9090"; then
    echo "✅ Prometheus is ready at http://localhost:9090"
else
    echo "⚠️  Prometheus may not be fully ready yet"
fi

# Check Grafana
if check_service "knhk-grafana" "3000"; then
    echo "✅ Grafana is ready at http://localhost:3000 (admin/admin)"
else
    echo "⚠️  Grafana may not be fully ready yet"
fi

# Check for Weaver
echo ""
echo "🔍 Checking for Weaver binary..."
if command -v weaver &> /dev/null; then
    echo "✅ Weaver binary found: $(which weaver)"
    weaver version 2>/dev/null || echo "⚠️  Weaver version check failed"
else
    echo "⚠️  Weaver binary not found. Install with: cargo install weaver"
    echo "   Or download from: https://github.com/open-telemetry/opentelemetry-collector-contrib/releases"
fi

echo ""
echo "✅ Infrastructure setup complete!"
echo ""
echo "📊 Service URLs:"
echo "   - OTLP HTTP: http://localhost:4318"
echo "   - OTLP gRPC: http://localhost:4317"
echo "   - Jaeger UI: http://localhost:16686"
echo "   - Prometheus: http://localhost:9090"
echo "   - Grafana: http://localhost:3000 (admin/admin)"
echo ""
echo "🧪 Run E2E tests with:"
echo "   cargo test --test chicago_tdd_e2e_validation --features std -- --ignored"
echo ""
echo "🛑 Stop infrastructure with:"
echo "   cd $INTEGRATION_DIR && $COMPOSE_CMD -f docker-compose.otel.yml down"

