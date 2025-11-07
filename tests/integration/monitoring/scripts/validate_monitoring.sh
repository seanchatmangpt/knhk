#!/bin/bash
set -euo pipefail

# KNHK v1.0 Monitoring Stack Validation Script
# Validates that all monitoring components are working correctly

PROMETHEUS_URL="${PROMETHEUS_URL:-http://localhost:9090}"
GRAFANA_URL="${GRAFANA_URL:-http://localhost:3000}"
ALERTMANAGER_URL="${ALERTMANAGER_URL:-http://localhost:9093}"
OTEL_METRICS_URL="${OTEL_METRICS_URL:-http://localhost:8888}"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

ERRORS=0

print_header() {
    echo ""
    echo "================================================"
    echo -e "${BLUE}$1${NC}"
    echo "================================================"
}

check_service() {
    local name=$1
    local url=$2
    local path=${3:-"/"}

    if curl -s -f "$url$path" > /dev/null; then
        echo -e "${GREEN}✅ $name is reachable${NC}"
        return 0
    else
        echo -e "${RED}❌ $name is NOT reachable at $url$path${NC}"
        ERRORS=$((ERRORS + 1))
        return 1
    fi
}

check_metrics() {
    local metric=$1
    local url="$PROMETHEUS_URL/api/v1/query"

    response=$(curl -s -G --data-urlencode "query=$metric" "$url")
    result_type=$(echo "$response" | jq -r '.data.resultType')

    if [ "$result_type" = "vector" ]; then
        result_count=$(echo "$response" | jq '.data.result | length')
        if [ "$result_count" -gt 0 ]; then
            echo -e "${GREEN}✅ Metric '$metric' exists ($result_count series)${NC}"
            return 0
        else
            echo -e "${YELLOW}⚠️  Metric '$metric' exists but has no data${NC}"
            return 0
        fi
    else
        echo -e "${RED}❌ Metric '$metric' not found${NC}"
        ERRORS=$((ERRORS + 1))
        return 1
    fi
}

check_alert_rules() {
    local url="$PROMETHEUS_URL/api/v1/rules"

    response=$(curl -s "$url")
    rule_count=$(echo "$response" | jq '.data.groups | length')

    if [ "$rule_count" -gt 0 ]; then
        echo -e "${GREEN}✅ Alert rules loaded: $rule_count groups${NC}"

        # Count individual rules
        total_rules=$(echo "$response" | jq '[.data.groups[].rules | length] | add')
        echo "   Total rules: $total_rules"

        # Show firing alerts
        firing=$(echo "$response" | jq -r '.data.groups[].rules[] | select(.state=="firing") | .name' | wc -l)
        pending=$(echo "$response" | jq -r '.data.groups[].rules[] | select(.state=="pending") | .name' | wc -l)

        if [ "$firing" -gt 0 ]; then
            echo -e "${RED}   Firing alerts: $firing${NC}"
        else
            echo -e "${GREEN}   Firing alerts: 0${NC}"
        fi

        if [ "$pending" -gt 0 ]; then
            echo -e "${YELLOW}   Pending alerts: $pending${NC}"
        fi

        return 0
    else
        echo -e "${RED}❌ No alert rules loaded${NC}"
        ERRORS=$((ERRORS + 1))
        return 1
    fi
}

check_grafana_datasources() {
    local url="$GRAFANA_URL/api/datasources"

    response=$(curl -s -u admin:knhk_admin "$url")
    ds_count=$(echo "$response" | jq 'length')

    if [ "$ds_count" -gt 0 ]; then
        echo -e "${GREEN}✅ Grafana data sources configured: $ds_count${NC}"

        # Check if Prometheus is configured
        prom_ds=$(echo "$response" | jq -r '.[] | select(.type=="prometheus") | .name' | head -1)
        if [ -n "$prom_ds" ]; then
            echo "   Prometheus data source: $prom_ds"
        fi

        return 0
    else
        echo -e "${RED}❌ No Grafana data sources configured${NC}"
        ERRORS=$((ERRORS + 1))
        return 1
    fi
}

check_grafana_dashboards() {
    local url="$GRAFANA_URL/api/search?type=dash-db"

    response=$(curl -s -u admin:knhk_admin "$url")
    dashboard_count=$(echo "$response" | jq 'length')

    if [ "$dashboard_count" -ge 3 ]; then
        echo -e "${GREEN}✅ Grafana dashboards provisioned: $dashboard_count${NC}"

        # List dashboards
        echo "$response" | jq -r '.[] | "   • \(.title) (UID: \(.uid))"'

        return 0
    else
        echo -e "${YELLOW}⚠️  Expected 3 dashboards, found: $dashboard_count${NC}"
        echo "$response" | jq -r '.[] | "   • \(.title)"'
        return 0
    fi
}

print_header "KNHK v1.0 Monitoring Stack Validation"

# Check services are reachable
print_header "1. Service Health Checks"
check_service "OTEL Collector Metrics" "$OTEL_METRICS_URL" "/metrics"
check_service "Prometheus" "$PROMETHEUS_URL" "/-/healthy"
check_service "Grafana" "$GRAFANA_URL" "/api/health"
check_service "Alertmanager" "$ALERTMANAGER_URL" "/-/healthy"

# Check Docker containers
print_header "2. Container Health"
containers=(
    "knhk-prometheus"
    "knhk-grafana"
    "knhk-alertmanager"
    "knhk-node-exporter"
    "knhk-test-otel-collector"
)

for container in "${containers[@]}"; do
    if docker ps --filter "name=$container" --filter "health=healthy" | grep -q "$container"; then
        echo -e "${GREEN}✅ Container $container is healthy${NC}"
    elif docker ps --filter "name=$container" | grep -q "$container"; then
        echo -e "${YELLOW}⚠️  Container $container is running but not healthy${NC}"
    else
        echo -e "${RED}❌ Container $container is not running${NC}"
        ERRORS=$((ERRORS + 1))
    fi
done

# Check KNHK metrics exist
print_header "3. KNHK Metrics Availability"
echo "Checking key KNHK metrics..."

metrics=(
    "knhk_beat_cycles_total"
    "knhk_beat_pulses_total"
    "knhk_operation_duration_bucket"
    "knhk_fiber_ticks_per_unit"
    "knhk_fiber_park_rate"
)

for metric in "${metrics[@]}"; do
    check_metrics "$metric" || true
done

# Check Prometheus targets
print_header "4. Prometheus Scrape Targets"
response=$(curl -s "$PROMETHEUS_URL/api/v1/targets")
active_targets=$(echo "$response" | jq '.data.activeTargets | length')
echo "Active targets: $active_targets"

# Check OTEL collector target
otel_target=$(echo "$response" | jq -r '.data.activeTargets[] | select(.labels.job=="otel-collector")')
if [ -n "$otel_target" ]; then
    health=$(echo "$otel_target" | jq -r '.health')
    if [ "$health" = "up" ]; then
        echo -e "${GREEN}✅ OTEL Collector target is UP${NC}"
    else
        echo -e "${RED}❌ OTEL Collector target is DOWN${NC}"
        ERRORS=$((ERRORS + 1))
    fi
else
    echo -e "${RED}❌ OTEL Collector target not found${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check alert rules
print_header "5. Alert Rules"
check_alert_rules

# Check Grafana
print_header "6. Grafana Configuration"
check_grafana_datasources
check_grafana_dashboards

# Summary
print_header "Validation Summary"

if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed! Monitoring stack is operational.${NC}"
    echo ""
    echo "📊 Access Points:"
    echo "   Grafana:       $GRAFANA_URL (admin/knhk_admin)"
    echo "   Prometheus:    $PROMETHEUS_URL"
    echo "   Alertmanager:  $ALERTMANAGER_URL"
    echo ""
    exit 0
else
    echo -e "${RED}❌ Validation failed with $ERRORS error(s)${NC}"
    echo ""
    echo "Troubleshooting steps:"
    echo "1. Check container logs:"
    echo "   docker-compose -f monitoring/docker-compose.monitoring.yml logs"
    echo ""
    echo "2. Restart monitoring stack:"
    echo "   ./scripts/stop_monitoring.sh"
    echo "   ./scripts/start_monitoring.sh"
    echo ""
    echo "3. Verify OTEL collector config:"
    echo "   docker exec knhk-test-otel-collector cat /etc/otelcol/config.yaml"
    echo ""
    exit 1
fi
