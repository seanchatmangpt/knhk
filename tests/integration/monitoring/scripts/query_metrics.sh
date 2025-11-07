#!/bin/bash
set -euo pipefail

# KNHK v1.0 Metrics Query Script
# CLI tool for querying key metrics from Prometheus

PROMETHEUS_URL="${PROMETHEUS_URL:-http://localhost:9090}"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo ""
    echo "================================================"
    echo -e "${BLUE}$1${NC}"
    echo "================================================"
}

query_prometheus() {
    local query="$1"
    local format="${2:-value}"

    response=$(curl -s -G --data-urlencode "query=$query" "$PROMETHEUS_URL/api/v1/query")

    if [ "$format" = "value" ]; then
        echo "$response" | jq -r '.data.result[0].value[1]' 2>/dev/null || echo "N/A"
    else
        echo "$response" | jq -r '.data.result' 2>/dev/null || echo "N/A"
    fi
}

format_percentage() {
    local value=$1
    printf "%.2f%%" $(echo "$value * 100" | bc -l)
}

format_number() {
    local value=$1
    printf "%.2f" $value
}

# Check if Prometheus is accessible
if ! curl -s "$PROMETHEUS_URL/-/healthy" > /dev/null; then
    echo -e "${RED}❌ Error: Cannot connect to Prometheus at $PROMETHEUS_URL${NC}"
    echo "   Make sure monitoring stack is running:"
    echo "   ./start_monitoring.sh"
    exit 1
fi

print_header "KNHK v1.0 Metrics Summary"

# SLO Compliance
print_header "🎯 SLO Compliance (Target: ≥95%)"
slo=$(query_prometheus "(sum(rate(knhk_operation_duration_bucket{le=\"8\"}[5m])) / sum(rate(knhk_operation_duration_count[5m]))) * 100")
if (( $(echo "$slo >= 95" | bc -l) )); then
    echo -e "${GREEN}✅ SLO: $(format_number $slo)%${NC}"
else
    echo -e "${RED}❌ SLO: $(format_number $slo)% (VIOLATED)${NC}"
fi

# Performance Metrics
print_header "⚡ Performance Metrics"
p50=$(query_prometheus "histogram_quantile(0.50, sum(rate(knhk_operation_duration_bucket[5m])) by (le))")
p95=$(query_prometheus "histogram_quantile(0.95, sum(rate(knhk_operation_duration_bucket[5m])) by (le))")
p99=$(query_prometheus "histogram_quantile(0.99, sum(rate(knhk_operation_duration_bucket[5m])) by (le))")

echo "P50 Latency: $(format_number $p50) ticks"
if (( $(echo "$p95 <= 8" | bc -l) )); then
    echo -e "${GREEN}P95 Latency: $(format_number $p95) ticks ✅${NC}"
else
    echo -e "${RED}P95 Latency: $(format_number $p95) ticks ❌ (>8 tick budget)${NC}"
fi
if (( $(echo "$p99 <= 8" | bc -l) )); then
    echo -e "${GREEN}P99 Latency: $(format_number $p99) ticks ✅${NC}"
else
    echo -e "${YELLOW}P99 Latency: $(format_number $p99) ticks ⚠️  (>8 tick budget)${NC}"
fi

# R1 Violations
print_header "🚨 R1 Violations (Chatman Constant >8 ticks)"
violations=$(query_prometheus "sum(knhk_operation_r1_violations_total)")
violation_rate=$(query_prometheus "sum(rate(knhk_operation_r1_violations_total[5m]))")
echo "Total Violations: $(format_number $violations)"
if (( $(echo "$violation_rate > 0" | bc -l) )); then
    echo -e "${RED}Current Rate: $(format_number $violation_rate)/sec ❌${NC}"
else
    echo -e "${GREEN}Current Rate: 0/sec ✅${NC}"
fi

# Beat System
print_header "🔄 Beat System (8-Beat Epoch)"
cycles=$(query_prometheus "knhk_beat_cycles_total")
pulses=$(query_prometheus "knhk_beat_pulses_total")
cycle_rate=$(query_prometheus "rate(knhk_beat_cycles_total[1m])")
pulse_ratio=$(query_prometheus "rate(knhk_beat_pulses_total[5m]) / (rate(knhk_beat_cycles_total[5m]) / 8)")

echo "Total Cycles: $(format_number $cycles)"
echo "Total Pulses: $(format_number $pulses)"
echo "Cycle Rate: $(format_number $cycle_rate)/sec"
if (( $(echo "$pulse_ratio >= 0.9 && $pulse_ratio <= 1.1" | bc -l) )); then
    echo -e "${GREEN}Pulse:Cycle Ratio: $(format_number $pulse_ratio) (target: 1.0) ✅${NC}"
else
    echo -e "${YELLOW}Pulse:Cycle Ratio: $(format_number $pulse_ratio) (target: 1.0) ⚠️${NC}"
fi

# Fiber Metrics
print_header "🧵 Fiber Execution"
park_rate=$(query_prometheus "avg(knhk_fiber_park_rate)")
deltas_processed=$(query_prometheus "sum(rate(knhk_fiber_deltas_processed_total[5m]))")
echo "Delta Throughput: $(format_number $deltas_processed)/sec"
if (( $(echo "$park_rate <= 0.2" | bc -l) )); then
    echo -e "${GREEN}Park Rate: $(format_percentage $park_rate) ✅${NC}"
else
    echo -e "${YELLOW}Park Rate: $(format_percentage $park_rate) ⚠️  (high W1 parking)${NC}"
fi

# Receipt Metrics
print_header "🧾 Receipt Metrics"
receipts_created=$(query_prometheus "sum(knhk_receipt_created_total)")
receipts_validated=$(query_prometheus "sum(knhk_receipt_validated_total)")
hash_mismatches=$(query_prometheus "sum(knhk_receipt_hash_mismatches_total)")
receipt_p95=$(query_prometheus "histogram_quantile(0.95, sum(rate(knhk_receipt_ticks_bucket[5m])) by (le))")

echo "Receipts Created: $(format_number $receipts_created)"
echo "Receipts Validated: $(format_number $receipts_validated)"
if [ "$hash_mismatches" = "N/A" ] || (( $(echo "$hash_mismatches == 0" | bc -l) )); then
    echo -e "${GREEN}Hash Mismatches: 0 ✅${NC}"
else
    echo -e "${RED}Hash Mismatches: $(format_number $hash_mismatches) ❌ CRITICAL${NC}"
fi
echo "P95 Receipt Ticks: $(format_number $receipt_p95)"

# Active Alerts
print_header "🔔 Active Alerts"
alerts=$(query_prometheus "ALERTS" "json")
alert_count=$(echo "$alerts" | jq 'length')

if [ "$alert_count" = "0" ]; then
    echo -e "${GREEN}No active alerts ✅${NC}"
else
    echo -e "${YELLOW}Active alerts: $alert_count ⚠️${NC}"
    echo "$alerts" | jq -r '.[] | "  • \(.labels.alertname): \(.labels.severity) - \(.annotations.summary)"'
fi

echo ""
echo "================================================"
echo "📊 Full metrics: $PROMETHEUS_URL"
echo "📈 Dashboards: http://localhost:3000"
echo "================================================"
echo ""
