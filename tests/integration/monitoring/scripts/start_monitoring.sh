#!/bin/bash
set -euo pipefail

# KNHK v1.0 Monitoring Stack Startup Script
# Brings up full monitoring infrastructure with Prometheus, Grafana, and Alertmanager

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MONITORING_DIR="$(dirname "$SCRIPT_DIR")"
INTEGRATION_DIR="$(dirname "$MONITORING_DIR")"

echo "🚀 Starting KNHK v1.0 Monitoring Stack..."
echo "================================================"

# Create docker network if it doesn't exist
if ! docker network inspect knhk-test-network &> /dev/null; then
    echo "📡 Creating knhk-test-network..."
    docker network create knhk-test-network
else
    echo "✅ knhk-test-network already exists"
fi

# Start base infrastructure first
echo ""
echo "🏗️  Starting base infrastructure (Kafka, Postgres, OTEL, Redis)..."
cd "$INTEGRATION_DIR"
docker-compose up -d

# Wait for base services to be healthy
echo ""
echo "⏳ Waiting for base services to be healthy..."
for i in {1..30}; do
    if docker ps --filter "name=knhk-test-kafka" --filter "health=healthy" | grep -q knhk-test-kafka && \
       docker ps --filter "name=knhk-test-postgres" --filter "health=healthy" | grep -q knhk-test-postgres && \
       docker ps --filter "name=knhk-test-redis" --filter "health=healthy" | grep -q knhk-test-redis; then
        echo "✅ Base services healthy"
        break
    fi
    echo "   Attempt $i/30: Waiting for services..."
    sleep 2
done

# Update OTEL collector config to export to Prometheus
echo ""
echo "📝 Updating OTEL collector configuration for Prometheus export..."
if [ -f "$INTEGRATION_DIR/otel-collector-config-with-prometheus.yaml" ]; then
    docker cp "$INTEGRATION_DIR/otel-collector-config-with-prometheus.yaml" knhk-test-otel-collector:/etc/otelcol/config.yaml
    docker restart knhk-test-otel-collector
    echo "✅ OTEL collector updated and restarted"
else
    echo "⚠️  Warning: otel-collector-config-with-prometheus.yaml not found, using default config"
fi

# Start monitoring stack
echo ""
echo "📊 Starting monitoring services (Prometheus, Grafana, Alertmanager)..."
cd "$MONITORING_DIR"
docker-compose -f docker-compose.monitoring.yml up -d

# Wait for monitoring services to be healthy
echo ""
echo "⏳ Waiting for monitoring services to be healthy..."
for i in {1..30}; do
    HEALTHY=true

    if ! docker ps --filter "name=knhk-prometheus" --filter "health=healthy" | grep -q knhk-prometheus; then
        HEALTHY=false
    fi

    if ! docker ps --filter "name=knhk-grafana" --filter "health=healthy" | grep -q knhk-grafana; then
        HEALTHY=false
    fi

    if ! docker ps --filter "name=knhk-alertmanager" --filter "health=healthy" | grep -q knhk-alertmanager; then
        HEALTHY=false
    fi

    if [ "$HEALTHY" = true ]; then
        echo "✅ Monitoring services healthy"
        break
    fi

    echo "   Attempt $i/30: Waiting for monitoring services..."
    sleep 2
done

echo ""
echo "================================================"
echo "✅ KNHK v1.0 Monitoring Stack Started!"
echo "================================================"
echo ""
echo "📊 Access Points:"
echo "   Grafana:       http://localhost:3000"
echo "                  Username: admin"
echo "                  Password: knhk_admin"
echo ""
echo "   Prometheus:    http://localhost:9090"
echo "   Alertmanager:  http://localhost:9093"
echo "   OTEL Metrics:  http://localhost:8888/metrics"
echo ""
echo "📈 Pre-configured Dashboards:"
echo "   • KNHK Beat System (8-Beat Epoch)"
echo "   • KNHK Performance & SLO Compliance"
echo "   • KNHK Receipt Metrics"
echo ""
echo "🔔 Alerting:"
echo "   • Tick budget violations (>8 ticks)"
echo "   • SLO violations (<95% compliance)"
echo "   • Receipt hash mismatches"
echo "   • System health issues"
echo ""
echo "📝 Logs:"
echo "   docker-compose -f $MONITORING_DIR/docker-compose.monitoring.yml logs -f"
echo ""
echo "🛑 To stop:"
echo "   $SCRIPT_DIR/stop_monitoring.sh"
echo ""
