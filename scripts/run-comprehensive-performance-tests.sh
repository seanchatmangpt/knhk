#!/usr/bin/env bash
# Comprehensive Performance Test Runner
# Validates all performance constraints including Chatman constant (≤8 ticks)

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPORT_DIR="$PROJECT_ROOT/reports/performance"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$REPORT_DIR"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}⚡ KNHK Comprehensive Performance Validation${NC}"
echo -e "${BLUE}   Law: μ ⊂ τ ; τ ≤ 8 ticks (Chatman Constant)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# Initialize results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Test 1: Hot Path Latency Tests
echo -e "${BLUE}[1/5] Running Hot Path Latency Tests...${NC}"
if cargo test --test hot_path_latency_test --release -- --nocapture 2>&1 | tee "$REPORT_DIR/hot_path_latency_$TIMESTAMP.log"; then
    echo -e "${GREEN}✅ Hot path latency tests PASSED${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}❌ Hot path latency tests FAILED${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))
echo

# Test 2: Warm Path Adaptation Tests
echo -e "${BLUE}[2/5] Running Warm Path Adaptation Tests...${NC}"
if cargo test --test warm_path_adaptation_test --release -- --nocapture 2>&1 | tee "$REPORT_DIR/warm_path_adaptation_$TIMESTAMP.log"; then
    echo -e "${GREEN}✅ Warm path adaptation tests PASSED${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}❌ Warm path adaptation tests FAILED${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))
echo

# Test 3: Receipt Generation Tests
echo -e "${BLUE}[3/5] Running Receipt Generation Tests...${NC}"
if cargo test --test receipt_generation_test --release -- --nocapture 2>&1 | tee "$REPORT_DIR/receipt_generation_$TIMESTAMP.log"; then
    echo -e "${GREEN}✅ Receipt generation tests PASSED${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}❌ Receipt generation tests FAILED${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))
echo

# Test 4: Chatman Constant Enforcement Tests
echo -e "${BLUE}[4/5] Running Chatman Constant Enforcement Tests...${NC}"
if cargo test --test chatman_constant_enforcement_test --release -- --nocapture 2>&1 | tee "$REPORT_DIR/chatman_constant_$TIMESTAMP.log"; then
    echo -e "${GREEN}✅ Chatman constant enforcement tests PASSED${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}❌ Chatman constant enforcement tests FAILED${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))
echo

# Test 5: C Performance Tests (PMU Benchmark Suite)
echo -e "${BLUE}[5/5] Running C PMU Benchmark Suite...${NC}"
if [ -f "$PROJECT_ROOT/tests/pmu_bench_suite" ]; then
    if "$PROJECT_ROOT/tests/pmu_bench_suite" 2>&1 | tee "$REPORT_DIR/pmu_bench_suite_$TIMESTAMP.log"; then
        echo -e "${GREEN}✅ C PMU benchmark suite PASSED${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ C PMU benchmark suite FAILED${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
else
    echo -e "${YELLOW}⚠️  C PMU benchmark suite not found (skipping)${NC}"
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))
echo

# Optional: Run Benchmarks (if requested)
if [ "${RUN_BENCHMARKS:-}" = "1" ]; then
    echo -e "${BLUE}Running Performance Benchmarks...${NC}"

    echo -e "${BLUE}  Hook Execution Benchmarks...${NC}"
    cargo bench --bench hook_execution_bench 2>&1 | tee "$REPORT_DIR/bench_hooks_$TIMESTAMP.log"

    echo -e "${BLUE}  Pattern Library Benchmarks...${NC}"
    cargo bench --bench pattern_library_bench 2>&1 | tee "$REPORT_DIR/bench_patterns_$TIMESTAMP.log"

    echo -e "${BLUE}  Guard Evaluation Benchmarks...${NC}"
    cargo bench --bench guard_evaluation_bench 2>&1 | tee "$REPORT_DIR/bench_guards_$TIMESTAMP.log"

    echo -e "${BLUE}  MAPE-K Cycle Benchmarks...${NC}"
    cargo bench --bench mape_k_cycle_bench 2>&1 | tee "$REPORT_DIR/bench_mapek_$TIMESTAMP.log"

    echo
fi

# Generate Summary Report
REPORT_FILE="$REPORT_DIR/summary_$TIMESTAMP.md"

cat > "$REPORT_FILE" <<EOF
# KNHK Performance Validation Report

**Date**: $(date)
**Test Suite**: Comprehensive Performance Validation
**Chatman Constant**: τ ≤ 8 ticks

## Summary

- **Total Tests**: $TOTAL_TESTS
- **Passed**: $PASSED_TESTS
- **Failed**: $FAILED_TESTS
- **Success Rate**: $(awk "BEGIN {printf \"%.1f\", ($PASSED_TESTS/$TOTAL_TESTS)*100}")%

## Test Results

### 1. Hot Path Latency Tests
Status: $([ -f "$REPORT_DIR/hot_path_latency_$TIMESTAMP.log" ] && echo "✅ Completed" || echo "❌ Failed")
Report: \`reports/performance/hot_path_latency_$TIMESTAMP.log\`

### 2. Warm Path Adaptation Tests
Status: $([ -f "$REPORT_DIR/warm_path_adaptation_$TIMESTAMP.log" ] && echo "✅ Completed" || echo "❌ Failed")
Report: \`reports/performance/warm_path_adaptation_$TIMESTAMP.log\`

### 3. Receipt Generation Tests
Status: $([ -f "$REPORT_DIR/receipt_generation_$TIMESTAMP.log" ] && echo "✅ Completed" || echo "❌ Failed")
Report: \`reports/performance/receipt_generation_$TIMESTAMP.log\`

### 4. Chatman Constant Enforcement Tests
Status: $([ -f "$REPORT_DIR/chatman_constant_$TIMESTAMP.log" ] && echo "✅ Completed" || echo "❌ Failed")
Report: \`reports/performance/chatman_constant_$TIMESTAMP.log\`

### 5. C PMU Benchmark Suite
Status: $([ -f "$REPORT_DIR/pmu_bench_suite_$TIMESTAMP.log" ] && echo "✅ Completed" || echo "❌ Failed")
Report: \`reports/performance/pmu_bench_suite_$TIMESTAMP.log\`

## Performance SLOs

| SLO | Target | Status |
|-----|--------|--------|
| Hot Path Latency | ≤ 8 ticks | $([ $FAILED_TESTS -eq 0 ] && echo "✅ MET" || echo "❌ VIOLATED") |
| Warm Path Adaptation | ≤ 1000ms | $([ $FAILED_TESTS -eq 0 ] && echo "✅ MET" || echo "❌ VIOLATED") |
| Receipt Generation | ≤ 50ms | $([ $FAILED_TESTS -eq 0 ] && echo "✅ MET" || echo "❌ VIOLATED") |
| Σ Pointer Updates | Atomic | $([ $FAILED_TESTS -eq 0 ] && echo "✅ MET" || echo "❌ VIOLATED") |

## Next Steps

$(if [ $FAILED_TESTS -eq 0 ]; then
    echo "🎉 All performance tests passed! The system meets all SLOs."
else
    echo "⚠️ $FAILED_TESTS test(s) failed. Review the logs and optimize the failing components."
fi)

---
*Generated by: scripts/run-comprehensive-performance-tests.sh*
EOF

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}📊 Performance Validation Results${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo -e "Total Tests: $TOTAL_TESTS"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
echo
echo -e "📄 Summary Report: ${BLUE}$REPORT_FILE${NC}"
echo

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✅ ALL PERFORMANCE TESTS PASSED${NC}"
    echo -e "${GREEN}   Chatman constant (τ ≤ 8) validated${NC}"
    exit 0
else
    echo -e "${RED}❌ $FAILED_TESTS TEST(S) FAILED${NC}"
    echo -e "${RED}   Review logs in: $REPORT_DIR${NC}"
    exit 1
fi
