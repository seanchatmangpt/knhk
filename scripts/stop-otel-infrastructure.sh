#!/bin/bash
# Stop OpenTelemetry infrastructure

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

echo "🛑 Stopping OpenTelemetry infrastructure..."
$COMPOSE_CMD -f docker-compose.otel.yml down

echo "✅ Infrastructure stopped"


