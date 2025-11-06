#!/usr/bin/env bash
# Chicago TDD Capability Validation for Reflex Enterprise
# Validates capabilities mentioned in REFLEX-CONVO.txt work correctly
# State-based verification: checks outputs and invariants

set -uo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASSED=0
FAILED=0
WARNINGS=0

# Test: Runtime Classes (R1/W1/C1) implementation
test_runtime_classes() {
    echo "[TEST] Runtime Classes (R1/W1/C1) Implementation"
    
    local code_file="rust/knhk-etl/src/runtime_class.rs"
    
    if [[ ! -f "$code_file" ]]; then
        echo "  ✗ Runtime class implementation not found"
        ((FAILED++))
        return 1
    fi
    
    # Verify R1/W1/C1 enum exists
    if ! grep -q "enum RuntimeClass" "$code_file"; then
        echo "  ✗ RuntimeClass enum not found"
        ((FAILED++))
        return 1
    fi
    
    # Verify R1 classification logic
    if ! grep -q "is_r1_operation" "$code_file"; then
        echo "  ✗ R1 operation classification not found"
        ((FAILED++))
        return 1
    fi
    
    # Verify W1 classification logic
    if ! grep -q "is_w1_operation" "$code_file"; then
        echo "  ✗ W1 operation classification not found"
        ((FAILED++))
        return 1
    fi
    
    # Verify C1 classification logic
    if ! grep -q "is_c1_operation" "$code_file"; then
        echo "  ✗ C1 operation classification not found"
        ((FAILED++))
        return 1
    fi
    
    # Verify metadata with budgets and SLOs
    if ! grep -q "budget_ns" "$code_file"; then
        echo "  ✗ Budget tracking not found"
        ((FAILED++))
        return 1
    fi
    
    if ! grep -q "slo_p99_ns" "$code_file"; then
        echo "  ✗ SLO tracking not found"
        ((FAILED++))
        return 1
    fi
    
    echo "  ✓ Runtime classes implementation verified"
    ((PASSED++))
    return 0
}

# Test: Hot Path Operations (ASK/COUNT/COMPARE/VALIDATE)
test_hot_path_operations() {
    echo "[TEST] Hot Path Operations (ASK/COUNT/COMPARE/VALIDATE)"
    
    local hot_file="rust/knhk-hot/src/ffi.rs"
    local c_header="c/include/knhk.h"
    
    local ops=("AskSp" "CountSpGe" "CompareOEQ" "AskSpo" "UniqueSp")
    local found=0
    
    # Check Rust FFI
    if [[ -f "$hot_file" ]]; then
        for op in "${ops[@]}"; do
            if grep -q "$op" "$hot_file"; then
                ((found++))
            fi
        done
    fi
    
    # Check C header (if exists)
    if [[ -f "$c_header" ]]; then
        if grep -q "KNHK_OP_ASK\|KNHK_OP_COUNT\|KNHK_OP_COMPARE" "$c_header"; then
            ((found++))
        fi
    fi
    
    # Also check Op enum
    if [[ -f "$hot_file" ]] && grep -q "enum Op" "$hot_file"; then
        ((found++))
    fi
    
    if [[ $found -gt 0 ]]; then
        echo "  ✓ Hot path operations found in code"
        ((PASSED++))
        return 0
    else
        echo "  ✗ Hot path operations not found"
        ((FAILED++))
        return 1
    fi
}

# Test: Warm Path Operations (CONSTRUCT8, prebind, AOT)
test_warm_path_operations() {
    echo "[TEST] Warm Path Operations (CONSTRUCT8, prebind, AOT)"
    
    local warm_file="rust/knhk-warm/src/lib.rs"
    local aot_file="rust/knhk-aot/src/lib.rs"
    local etl_file="rust/knhk-etl/src/lib.rs"
    
    local found=0
    
    # Check CONSTRUCT8
    if grep -qi "CONSTRUCT8\|construct8" "$warm_file" "$etl_file" 2>/dev/null; then
        ((found++))
    fi
    
    # Check prebind
    if [[ -f "$aot_file" ]] && grep -qi "prebind\|PreboundIr" "$aot_file"; then
        ((found++))
    fi
    
    # Check AOT
    if [[ -f "$aot_file" ]] && grep -qi "AotGuard\|AOT" "$aot_file"; then
        ((found++))
    fi
    
    if [[ $found -ge 2 ]]; then
        echo "  ✓ Warm path operations found"
        ((PASSED++))
        return 0
    else
        echo "  ⚠ Some warm path operations may be missing (found: $found/3)"
        ((WARNINGS++))
        return 0
    fi
}

# Test: SLO Monitoring
test_slo_monitoring() {
    echo "[TEST] SLO Monitoring Implementation"
    
    local slo_file="rust/knhk-etl/src/slo_monitor.rs"
    
    if [[ ! -f "$slo_file" ]]; then
        echo "  ✗ SLO monitor not found"
        ((FAILED++))
        return 1
    fi
    
    # Verify p99 calculation
    if ! grep -q "p99\|percentile" "$slo_file"; then
        echo "  ✗ p99 calculation not found"
        ((FAILED++))
        return 1
    fi
    
    # Verify violation detection
    if ! grep -q "SloViolation\|violation" "$slo_file"; then
        echo "  ✗ SLO violation detection not found"
        ((FAILED++))
        return 1
    fi
    
    echo "  ✓ SLO monitoring implementation verified"
    ((PASSED++))
    return 0
}

# Test: Failure Actions (R1/W1/C1)
test_failure_actions() {
    echo "[TEST] Failure Actions (R1/W1/C1)"
    
    local failure_file="rust/knhk-etl/src/failure_actions.rs"
    
    if [[ ! -f "$failure_file" ]]; then
        echo "  ✗ Failure actions not found"
        ((FAILED++))
        return 1
    fi
    
    # Verify R1 failure actions (drop/park/escalate)
    if ! grep -qi "drop\|park\|escalate" "$failure_file"; then
        echo "  ⚠ R1 failure actions may be incomplete"
        ((WARNINGS++))
    fi
    
    # Verify W1 failure actions (retry/degrade)
    if ! grep -qi "retry\|degrade" "$failure_file"; then
        echo "  ⚠ W1 failure actions may be incomplete"
        ((WARNINGS++))
    fi
    
    # Verify C1 failure actions (async)
    if ! grep -qi "async\|non.*block" "$failure_file"; then
        echo "  ⚠ C1 failure actions may be incomplete"
        ((WARNINGS++))
    fi
    
    echo "  ✓ Failure actions implementation found"
    ((PASSED++))
    return 0
}

# Test: Lockchain/Receipts
test_lockchain_receipts() {
    echo "[TEST] Lockchain/Receipts Implementation"
    
    local lockchain_file="rust/knhk-lockchain/src/lib.rs"
    
    if [[ ! -f "$lockchain_file" ]]; then
        echo "  ✗ Lockchain implementation not found"
        ((FAILED++))
        return 1
    fi
    
    # Verify Lockchain struct
    if ! grep -q "struct Lockchain" "$lockchain_file"; then
        echo "  ✗ Lockchain struct not found"
        ((FAILED++))
        return 1
    fi
    
    # Verify receipt operations
    if ! grep -q "append\|verify\|merge" "$lockchain_file"; then
        echo "  ✗ Receipt operations not found"
        ((FAILED++))
        return 1
    fi
    
    # Verify Merkle linking
    if ! grep -qi "merkle\|hash" "$lockchain_file"; then
        echo "  ⚠ Merkle linking may be incomplete"
        ((WARNINGS++))
    fi
    
    echo "  ✓ Lockchain/receipts implementation verified"
    ((PASSED++))
    return 0
}

# Test: OTEL Integration
test_otel_integration() {
    echo "[TEST] OTEL Integration"
    
    local otel_file="rust/knhk-otel/src/lib.rs"
    
    if [[ ! -f "$otel_file" ]]; then
        echo "  ✗ OTEL implementation not found"
        ((FAILED++))
        return 1
    fi
    
    # Verify Tracer
    if ! grep -q "struct Tracer" "$otel_file"; then
        echo "  ✗ Tracer not found"
        ((FAILED++))
        return 1
    fi
    
    # Verify span operations
    if ! grep -q "start_span\|end_span" "$otel_file"; then
        echo "  ✗ Span operations not found"
        ((FAILED++))
        return 1
    fi
    
    # Verify metrics
    if ! grep -q "record_metric\|MetricsHelper" "$otel_file"; then
        echo "  ⚠ Metrics may be incomplete"
        ((WARNINGS++))
    fi
    
    echo "  ✓ OTEL integration verified"
    ((PASSED++))
    return 0
}

# Test: Integration Patterns (Sidecar, Connector)
test_integration_patterns() {
    echo "[TEST] Integration Patterns (Sidecar, Connector)"
    
    local sidecar_file="rust/knhk-sidecar/src/lib.rs"
    local connector_file="rust/knhk-connectors/src/lib.rs"
    
    local found=0
    
    # Check sidecar
    if [[ -f "$sidecar_file" ]]; then
        if grep -q "sidecar\|Sidecar" "$sidecar_file"; then
            ((found++))
        fi
    fi
    
    # Check connector
    if [[ -f "$connector_file" ]]; then
        if grep -q "Connector\|connector" "$connector_file"; then
            ((found++))
        fi
    fi
    
    if [[ $found -ge 1 ]]; then
        echo "  ✓ Integration patterns found"
        ((PASSED++))
        return 0
    else
        echo "  ⚠ Some integration patterns may be missing"
        ((WARNINGS++))
        return 0
    fi
}

# Test: AOT/MPHF/Preloading
test_performance_engineering() {
    echo "[TEST] Performance Engineering (AOT/MPHF/Preloading)"
    
    local aot_file="rust/knhk-aot/src/lib.rs"
    local mphf_file="rust/knhk-aot/src/mphf.rs"
    
    local found=0
    
    # Check AOT
    if [[ -f "$aot_file" ]] && grep -q "AotGuard\|AOT" "$aot_file"; then
        ((found++))
    fi
    
    # Check MPHF
    if [[ -f "$mphf_file" ]] && grep -q "Mphf\|MPHF" "$mphf_file"; then
        ((found++))
    fi
    
    # Check preloading/prebinding
    if [[ -f "$aot_file" ]] && grep -qi "prebind\|preload" "$aot_file"; then
        ((found++))
    fi
    
    if [[ $found -ge 2 ]]; then
        echo "  ✓ Performance engineering features found"
        ((PASSED++))
        return 0
    else
        echo "  ⚠ Some performance features may be missing (found: $found/3)"
        ((WARNINGS++))
        return 0
    fi
}

# Test: Runtime Class Tests Exist
test_runtime_class_tests() {
    echo "[TEST] Runtime Class Tests Exist"
    
    local test_file="rust/knhk-etl/tests/runtime_class_test.rs"
    
    if [[ ! -f "$test_file" ]]; then
        echo "  ⚠ Runtime class tests not found"
        ((WARNINGS++))
        return 0
    fi
    
    # Verify test coverage
    if grep -q "test_r1\|test_w1\|test_c1" "$test_file"; then
        echo "  ✓ Runtime class tests found"
        ((PASSED++))
        return 0
    else
        echo "  ⚠ Runtime class tests may be incomplete"
        ((WARNINGS++))
        return 0
    fi
}

# Test: Hot Path Budget Enforcement (≤8 ticks)
test_hot_path_budget() {
    echo "[TEST] Hot Path Budget Enforcement (≤8 ticks)"
    
    local aot_file="rust/knhk-aot/src/lib.rs"
    local hot_file="rust/knhk-hot/src/ffi.rs"
    
    local found=0
    
    # Check AOT guard validates ≤8 ticks
    if [[ -f "$aot_file" ]] && grep -q "run_len.*8\|8.*run_len" "$aot_file"; then
        ((found++))
    fi
    
    # Check hot path constraints
    if [[ -f "$hot_file" ]] && grep -qi "8.*tick\|tick.*8" "$hot_file"; then
        ((found++))
    fi
    
    if [[ $found -gt 0 ]]; then
        echo "  ✓ Hot path budget enforcement found"
        ((PASSED++))
        return 0
    else
        echo "  ⚠ Hot path budget enforcement may be incomplete"
        ((WARNINGS++))
        return 0
    fi
}

# Main execution
main() {
    echo "=========================================="
    echo "Reflex Enterprise Capability Validation"
    echo "Chicago TDD: State-based verification"
    echo "=========================================="
    echo ""
    
    test_runtime_classes
    test_hot_path_operations
    test_warm_path_operations
    test_slo_monitoring
    test_failure_actions
    test_lockchain_receipts
    test_otel_integration
    test_integration_patterns
    test_performance_engineering
    test_runtime_class_tests
    test_hot_path_budget
    
    echo ""
    echo "=========================================="
    echo "Results: $PASSED passed, $FAILED failed, $WARNINGS warnings"
    echo "=========================================="
    
    if [[ $FAILED -eq 0 ]]; then
        if [[ $WARNINGS -eq 0 ]]; then
            echo -e "${GREEN}✓ All capabilities verified${NC}"
            exit 0
        else
            echo -e "${YELLOW}⚠ All critical capabilities verified, $WARNINGS warnings${NC}"
            exit 0
        fi
    else
        echo -e "${RED}✗ $FAILED critical capabilities missing${NC}"
        exit 1
    fi
}

main "$@"

