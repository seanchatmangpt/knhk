#!/bin/bash
set -euo pipefail

# KNHK v1.0 Monitoring Stack Shutdown Script
# Gracefully stops all monitoring services

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MONITORING_DIR="$(dirname "$SCRIPT_DIR")"
INTEGRATION_DIR="$(dirname "$MONITORING_DIR")"

echo "🛑 Stopping KNHK v1.0 Monitoring Stack..."
echo "================================================"

# Stop monitoring services first
echo "📊 Stopping monitoring services..."
cd "$MONITORING_DIR"
docker-compose -f docker-compose.monitoring.yml down

# Optionally stop base infrastructure
read -p "🤔 Stop base infrastructure (Kafka, Postgres, OTEL, Redis)? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🏗️  Stopping base infrastructure..."
    cd "$INTEGRATION_DIR"
    docker-compose down

    read -p "🗑️  Remove volumes (data will be lost)? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "🗑️  Removing volumes..."
        docker-compose down -v
        docker volume rm prometheus-data grafana-data alertmanager-data 2>/dev/null || true
    fi
fi

echo ""
echo "================================================"
echo "✅ Monitoring Stack Stopped"
echo "================================================"
echo ""
echo "🔄 To restart:"
echo "   $SCRIPT_DIR/start_monitoring.sh"
echo ""
