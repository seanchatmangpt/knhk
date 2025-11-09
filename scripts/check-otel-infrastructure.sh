#!/bin/bash
# Check OpenTelemetry infrastructure status

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
INTEGRATION_DIR="$PROJECT_ROOT/tests/integration"

# Check if docker-compose is available
if command -v docker-compose &> /dev/null; then
    COMPOSE_CMD="docker-compose"
elif docker compose version &> /dev/null; then
    COMPOSE_CMD="docker compose"
else
    echo "❌ docker-compose is not installed."
    exit 1
fi

cd "$INTEGRATION_DIR"

echo "🔍 Checking OpenTelemetry infrastructure status..."
echo ""

# Check if containers are running
if $COMPOSE_CMD -f docker-compose.otel.yml ps | grep -q "Up"; then
    echo "✅ Infrastructure is running:"
    $COMPOSE_CMD -f docker-compose.otel.yml ps
    echo ""
    
    # Check service endpoints
    echo "🔍 Testing service endpoints..."
    
    check_endpoint() {
        local name=$1
        local url=$2
        if curl -s "$url" > /dev/null 2>&1; then
            echo "✅ $name: $url"
        else
            echo "❌ $name: $url (not reachable)"
        fi
    }
    
    check_endpoint "OTLP HTTP" "http://localhost:4318/v1/traces"
    check_endpoint "Jaeger UI" "http://localhost:16686"
    check_endpoint "Prometheus" "http://localhost:9090/-/healthy"
    check_endpoint "Grafana" "http://localhost:3000/api/health"
    
    echo ""
    echo "📊 Service URLs:"
    echo "   - OTLP HTTP: http://localhost:4318"
    echo "   - OTLP gRPC: http://localhost:4317"
    echo "   - Jaeger UI: http://localhost:16686"
    echo "   - Prometheus: http://localhost:9090"
    echo "   - Grafana: http://localhost:3000"
else
    echo "❌ Infrastructure is not running"
    echo ""
    echo "Start it with:"
    echo "   ./scripts/setup-otel-infrastructure.sh"
fi

# Check for Weaver
echo ""
if command -v weaver &> /dev/null; then
    echo "✅ Weaver binary found: $(which weaver)"
else
    echo "⚠️  Weaver binary not found"
fi





