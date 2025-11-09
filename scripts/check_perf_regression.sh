#!/bin/bash
# Performance Regression Detector
# Compares current benchmark results against baseline and fails if regression exceeds threshold

set -euo pipefail

THRESHOLD=${1:-10}  # Default 10% regression threshold
BENCH_DIR="rust/knhk-workflow-engine"
CRITERION_DIR="$BENCH_DIR/target/criterion"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Performance Regression Check (Threshold: ${THRESHOLD}%)${NC}"
echo ""

# Check if baseline exists
if [ ! -d "$CRITERION_DIR" ]; then
    echo -e "${RED}✗ No criterion results found. Run benchmarks first.${NC}"
    exit 1
fi

regression_found=0
total_benchmarks=0

# Function to check single benchmark
check_benchmark() {
    local benchmark_name=$1
    local estimates_file="$CRITERION_DIR/$benchmark_name/base/estimates.json"
    local change_file="$CRITERION_DIR/$benchmark_name/change/estimates.json"

    if [ ! -f "$estimates_file" ]; then
        return
    fi

    total_benchmarks=$((total_benchmarks + 1))

    # Extract mean time (in nanoseconds)
    mean_time=$(jq -r '.mean.point_estimate' "$estimates_file" 2>/dev/null || echo "0")

    # Check for change file (comparison to baseline)
    if [ -f "$change_file" ]; then
        change_percent=$(jq -r '.mean.point_estimate' "$change_file" 2>/dev/null || echo "0")

        # Convert to absolute value for comparison
        abs_change=$(echo "$change_percent" | awk '{print ($1<0)?-$1:$1}')

        if (( $(echo "$change_percent > $THRESHOLD" | bc -l) )); then
            echo -e "${RED}✗ $benchmark_name: +${change_percent}% regression (>${THRESHOLD}%)${NC}"
            regression_found=1
        elif (( $(echo "$abs_change > 5" | bc -l) )); then
            if (( $(echo "$change_percent < 0" | bc -l) )); then
                echo -e "${GREEN}✓ $benchmark_name: ${change_percent}% improvement${NC}"
            else
                echo -e "${YELLOW}⚠ $benchmark_name: +${change_percent}% regression (<${THRESHOLD}%)${NC}"
            fi
        else
            echo -e "${GREEN}✓ $benchmark_name: ${change_percent}% (acceptable)${NC}"
        fi
    else
        echo -e "${YELLOW}⚠ $benchmark_name: No baseline for comparison${NC}"
    fi
}

# Check all hot path benchmarks (CRITICAL)
echo -e "${YELLOW}Hot Path Benchmarks (CRITICAL - Chatman Constant):${NC}"
for bench in pattern_execution state_transition condition_evaluation task_lookup_hot_read; do
    check_benchmark "hot_path/$bench"
done
echo ""

# Check E2E benchmarks
echo -e "${YELLOW}End-to-End Benchmarks:${NC}"
for bench in atm_withdrawal swift_payment; do
    check_benchmark "e2e_workflows/$bench"
done
echo ""

# Check scalability benchmarks
echo -e "${YELLOW}Scalability Benchmarks:${NC}"
for count in 10 100 500 1000; do
    check_benchmark "scalability/payroll_employees/$count"
done
echo ""

# Summary
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"
if [ $total_benchmarks -eq 0 ]; then
    echo -e "${YELLOW}⚠ No benchmarks found to check${NC}"
    exit 0
elif [ $regression_found -eq 1 ]; then
    echo -e "${RED}✗ PERFORMANCE REGRESSION DETECTED${NC}"
    echo -e "${RED}  Some benchmarks exceed ${THRESHOLD}% regression threshold${NC}"
    exit 1
else
    echo -e "${GREEN}✓ NO SIGNIFICANT REGRESSIONS${NC}"
    echo -e "${GREEN}  All benchmarks within ${THRESHOLD}% threshold${NC}"
    exit 0
fi
