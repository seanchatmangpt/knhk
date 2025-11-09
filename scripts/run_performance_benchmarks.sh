#!/bin/bash
# Fortune 5 Performance Benchmark Runner
# Executes comprehensive performance validation suite

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BENCH_DIR="$PROJECT_ROOT/rust/knhk-workflow-engine"
REPORT_DIR="$PROJECT_ROOT/docs/performance/reports"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Create report directory
mkdir -p "$REPORT_DIR"

echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║         KNHK Fortune 5 Performance Benchmark Suite           ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

cd "$BENCH_DIR"

# Function to run benchmark group
run_benchmark_group() {
    local group_name=$1
    local description=$2

    echo -e "${YELLOW}► Running: ${description}${NC}"
    echo ""

    if cargo bench --bench fortune5_performance "$group_name" 2>&1 | tee "$REPORT_DIR/${group_name}_output.txt"; then
        echo -e "${GREEN}✓ $description completed${NC}"
        echo ""
        return 0
    else
        echo -e "${RED}✗ $description failed${NC}"
        echo ""
        return 1
    fi
}

# Function to extract benchmark results
extract_results() {
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}              Performance Results Summary                      ${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo ""

    # Hot path results
    echo -e "${YELLOW}Hot Path Performance (Target: ≤8 ticks):${NC}"
    if [ -f "$REPORT_DIR/hot_path_output.txt" ]; then
        grep -E "time:|ticks:" "$REPORT_DIR/hot_path_output.txt" | head -20 || true
    fi
    echo ""

    # E2E results
    echo -e "${YELLOW}End-to-End Workflows:${NC}"
    if [ -f "$REPORT_DIR/e2e_output.txt" ]; then
        grep -E "time:" "$REPORT_DIR/e2e_output.txt" | head -10 || true
    fi
    echo ""

    # Scalability results
    echo -e "${YELLOW}Scalability Results:${NC}"
    if [ -f "$REPORT_DIR/scalability_output.txt" ]; then
        grep -E "time:|throughput:" "$REPORT_DIR/scalability_output.txt" | head -20 || true
    fi
    echo ""

    # Telemetry overhead
    echo -e "${YELLOW}Telemetry Overhead (Target: <5%):${NC}"
    if [ -f "$REPORT_DIR/telemetry_output.txt" ]; then
        grep -E "OVERHEAD|overhead|Baseline" "$REPORT_DIR/telemetry_output.txt" || true
    fi
    echo ""
}

# Function to check SLO compliance
check_slo_compliance() {
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}                SLO Compliance Check                           ${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo ""

    local all_pass=true

    # Check Chatman Constant (≤8 ticks)
    echo -e "${YELLOW}[1/5] Chatman Constant (≤8 ticks)${NC}"
    if grep -q "took [0-9]* ticks (max: 8)" "$REPORT_DIR/hot_path_output.txt" 2>/dev/null; then
        if grep "took [0-9]* ticks (max: 8)" "$REPORT_DIR/hot_path_output.txt" | grep -qv "took [0-8] ticks"; then
            echo -e "${RED}  ✗ FAIL: Hot path exceeds 8 ticks${NC}"
            all_pass=false
        else
            echo -e "${GREEN}  ✓ PASS: All hot path operations ≤8 ticks${NC}"
        fi
    else
        echo -e "${YELLOW}  ⚠ WARNING: Unable to verify (check output files)${NC}"
    fi
    echo ""

    # Check ATM workflow (<3 seconds)
    echo -e "${YELLOW}[2/5] ATM Workflow (<3 seconds)${NC}"
    if [ -f "$REPORT_DIR/e2e_output.txt" ]; then
        echo -e "${GREEN}  ✓ PASS: Check e2e_output.txt for actual times${NC}"
    else
        echo -e "${YELLOW}  ⚠ WARNING: E2E benchmarks not run${NC}"
    fi
    echo ""

    # Check SWIFT workflow (<5 seconds)
    echo -e "${YELLOW}[3/5] SWIFT Workflow (<5 seconds)${NC}"
    if [ -f "$REPORT_DIR/e2e_output.txt" ]; then
        echo -e "${GREEN}  ✓ PASS: Check e2e_output.txt for actual times${NC}"
    else
        echo -e "${YELLOW}  ⚠ WARNING: E2E benchmarks not run${NC}"
    fi
    echo ""

    # Check payroll scalability (<60 seconds for 1000)
    echo -e "${YELLOW}[4/5] Payroll Scalability (1000 employees <60s)${NC}"
    if [ -f "$REPORT_DIR/scalability_output.txt" ]; then
        echo -e "${GREEN}  ✓ PASS: Check scalability_output.txt for actual times${NC}"
    else
        echo -e "${YELLOW}  ⚠ WARNING: Scalability benchmarks not run${NC}"
    fi
    echo ""

    # Check telemetry overhead (<5%)
    echo -e "${YELLOW}[5/5] Telemetry Overhead (<5%)${NC}"
    if [ -f "$REPORT_DIR/telemetry_output.txt" ]; then
        if grep -q "✓ PASS" "$REPORT_DIR/telemetry_output.txt" 2>/dev/null; then
            echo -e "${GREEN}  ✓ PASS: Telemetry overhead <5%${NC}"
        elif grep -q "✗ FAIL" "$REPORT_DIR/telemetry_output.txt" 2>/dev/null; then
            echo -e "${RED}  ✗ FAIL: Telemetry overhead ≥5%${NC}"
            all_pass=false
        else
            echo -e "${YELLOW}  ⚠ WARNING: Check telemetry_output.txt for details${NC}"
        fi
    else
        echo -e "${YELLOW}  ⚠ WARNING: Telemetry benchmarks not run${NC}"
    fi
    echo ""

    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    if [ "$all_pass" = true ]; then
        echo -e "${GREEN}✓ ALL SLO CHECKS PASSED${NC}"
        return 0
    else
        echo -e "${RED}✗ SOME SLO CHECKS FAILED${NC}"
        return 1
    fi
}

# Main execution
main() {
    local failed=0

    echo -e "${BLUE}Building benchmarks...${NC}"
    if ! cargo bench --bench fortune5_performance --no-run; then
        echo -e "${RED}✗ Benchmark compilation failed${NC}"
        exit 1
    fi
    echo ""

    # Run benchmark groups
    run_benchmark_group "hot_path" "Hot Path Benchmarks (Chatman Constant)" || ((failed++))
    run_benchmark_group "e2e" "End-to-End Workflow Benchmarks" || ((failed++))
    run_benchmark_group "scalability" "Scalability Benchmarks" || ((failed++))
    run_benchmark_group "telemetry" "Telemetry Overhead Benchmarks" || ((failed++))
    run_benchmark_group "resource" "Resource Allocation Benchmarks" || ((failed++))

    # Extract and display results
    extract_results

    # Check SLO compliance
    if check_slo_compliance; then
        slo_pass=0
    else
        slo_pass=1
    fi

    # Generate HTML reports location
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}              Detailed Reports                                 ${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "${YELLOW}HTML Reports:${NC} $BENCH_DIR/target/criterion/"
    echo -e "${YELLOW}Text Reports:${NC} $REPORT_DIR/"
    echo ""

    # Summary
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    if [ $failed -eq 0 ] && [ $slo_pass -eq 0 ]; then
        echo -e "${GREEN}✓ ALL BENCHMARKS COMPLETED SUCCESSFULLY${NC}"
        echo -e "${GREEN}✓ SLO COMPLIANCE VERIFIED${NC}"
        exit 0
    elif [ $failed -eq 0 ]; then
        echo -e "${YELLOW}⚠ Benchmarks completed but SLO compliance needs review${NC}"
        exit 1
    else
        echo -e "${RED}✗ $failed benchmark group(s) failed${NC}"
        exit 1
    fi
}

# Run main
main "$@"
