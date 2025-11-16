#!/usr/bin/env bash
#
# Chicago TDD Performance Harness - Complete Benchmark Suite
#
# Runs all benchmarks and enforces Chatman Constant (≤8 ticks) for hot path.
# Exits with error if any bounds are violated.
#
# Covenant 5: The Chatman Constant Guards All Complexity

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
CHICAGO_DIR="$ROOT_DIR/rust/chicago-tdd"
RESULTS_DIR="$ROOT_DIR/chicago-tdd-results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Create results directory
mkdir -p "$RESULTS_DIR"

echo -e "${BLUE}${BOLD}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}${BOLD}║                   Chicago TDD Performance Harness                         ║${NC}"
echo -e "${BLUE}${BOLD}║              Enforcing Chatman Constant (≤8 ticks hot path)              ║${NC}"
echo -e "${BLUE}${BOLD}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Change to chicago-tdd directory
cd "$CHICAGO_DIR"

# Track overall status
OVERALL_PASS=true
BENCHMARKS_RUN=0
BENCHMARKS_FAILED=0

# Function to run a benchmark
run_benchmark() {
    local bench_name="$1"
    local description="$2"

    echo -e "${YELLOW}┌─────────────────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${YELLOW}│ Running: ${BOLD}$description${NC}${YELLOW}"
    echo -e "${YELLOW}└─────────────────────────────────────────────────────────────────────────┘${NC}"
    echo ""

    BENCHMARKS_RUN=$((BENCHMARKS_RUN + 1))

    # Run benchmark and capture output
    local output_file="$RESULTS_DIR/${bench_name}_${TIMESTAMP}.txt"

    if cargo bench --bench "$bench_name" -- --nocapture 2>&1 | tee "$output_file"; then
        echo -e "${GREEN}✓ PASS${NC} - $description"
        echo ""
        return 0
    else
        echo -e "${RED}✗ FAIL${NC} - $description"
        echo ""
        OVERALL_PASS=false
        BENCHMARKS_FAILED=$((BENCHMARKS_FAILED + 1))
        return 1
    fi
}

# Run all benchmarks
echo -e "${BLUE}Running all Chicago TDD benchmarks...${NC}"
echo ""

run_benchmark "executor_latency" "Executor Hot Path Latency"
run_benchmark "task_dispatch" "Task Dispatch Latency"
run_benchmark "decision_point" "Decision Point Latency"
run_benchmark "join_operation" "Join Operation Latency"
run_benchmark "mape_k_latency" "MAPE-K Autonomic Loop Latency"

# Run integration tests
echo -e "${YELLOW}┌─────────────────────────────────────────────────────────────────────────┐${NC}"
echo -e "${YELLOW}│ Running: ${BOLD}Integration Tests${NC}${YELLOW}"
echo -e "${YELLOW}└─────────────────────────────────────────────────────────────────────────┘${NC}"
echo ""

if cargo test --test integration_tests -- --nocapture 2>&1 | tee "$RESULTS_DIR/integration_tests_${TIMESTAMP}.txt"; then
    echo -e "${GREEN}✓ PASS${NC} - Integration Tests"
    echo ""
else
    echo -e "${RED}✗ FAIL${NC} - Integration Tests"
    echo ""
    OVERALL_PASS=false
fi

# Run bounds tests
echo -e "${YELLOW}┌─────────────────────────────────────────────────────────────────────────┐${NC}"
echo -e "${YELLOW}│ Running: ${BOLD}Bounds Enforcement Tests${NC}${YELLOW}"
echo -e "${YELLOW}└─────────────────────────────────────────────────────────────────────────┘${NC}"
echo ""

if cargo test --test integration_tests bounds_tests -- --nocapture 2>&1 | tee "$RESULTS_DIR/bounds_tests_${TIMESTAMP}.txt"; then
    echo -e "${GREEN}✓ PASS${NC} - Bounds Tests"
    echo ""
else
    echo -e "${RED}✗ FAIL${NC} - Bounds Tests"
    echo ""
    OVERALL_PASS=false
fi

# Generate summary report
echo ""
echo -e "${BLUE}${BOLD}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}${BOLD}║                            SUMMARY REPORT                                 ║${NC}"
echo -e "${BLUE}${BOLD}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  Total Benchmarks Run:    ${BOLD}$BENCHMARKS_RUN${NC}"
echo -e "  Benchmarks Failed:       ${BOLD}$BENCHMARKS_FAILED${NC}"
echo -e "  Results Directory:       ${BOLD}$RESULTS_DIR${NC}"
echo ""

if [ "$OVERALL_PASS" = true ]; then
    echo -e "${GREEN}${BOLD}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}${BOLD}║                           ✓ ALL TESTS PASSED                              ║${NC}"
    echo -e "${GREEN}${BOLD}║         All operations satisfy Chatman Constant (≤8 ticks)               ║${NC}"
    echo -e "${GREEN}${BOLD}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    exit 0
else
    echo -e "${RED}${BOLD}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}${BOLD}║                           ✗ TESTS FAILED                                  ║${NC}"
    echo -e "${RED}${BOLD}║              One or more operations violated bounds                       ║${NC}"
    echo -e "${RED}${BOLD}║                     BUILD MUST BE BLOCKED                                 ║${NC}"
    echo -e "${RED}${BOLD}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${RED}Action Required:${NC}"
    echo -e "  1. Review benchmark results in: $RESULTS_DIR"
    echo -e "  2. Identify operations exceeding bounds"
    echo -e "  3. Optimize hot path code or move operations off critical path"
    echo -e "  4. Re-run benchmarks after fixes"
    echo ""
    exit 1
fi
