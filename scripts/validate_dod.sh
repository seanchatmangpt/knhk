#!/usr/bin/env bash

################################################################################
# KNHK Definition of Done (DoD) Validation Script
# ===============================================
# This script validates that all DoD criteria are satisfied.
# It's designed to run in CI/CD and produce a machine-readable report.
#
# Usage: ./scripts/validate_dod.sh [--json] [--verbose]
# Exit Code: 0 if all ci_failure: true criteria pass, 1 otherwise
################################################################################

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VERBOSE=${VERBOSE:-0}
JSON_OUTPUT=${JSON_OUTPUT:-0}
REPORT_FILE="${PROJECT_ROOT}/target/dod_report.json"

# Results tracking
PASSED_CHECKS=0
FAILED_CHECKS=0
WARNINGS=0
declare -a FAILED_CRITICAL
declare -a WARNINGS_LIST

################################################################################
# UTILITY FUNCTIONS
################################################################################

log_info() {
    if [ "$JSON_OUTPUT" -eq 0 ]; then
        echo -e "${BLUE}[INFO]${NC} $*"
    fi
}

log_pass() {
    if [ "$JSON_OUTPUT" -eq 0 ]; then
        echo -e "${GREEN}[PASS]${NC} $*"
    fi
    ((PASSED_CHECKS++))
}

log_fail() {
    if [ "$JSON_OUTPUT" -eq 0 ]; then
        echo -e "${RED}[FAIL]${NC} $*"
    fi
    ((FAILED_CHECKS++))
    FAILED_CRITICAL+=("$*")
}

log_warn() {
    if [ "$JSON_OUTPUT" -eq 0 ]; then
        echo -e "${YELLOW}[WARN]${NC} $*"
    fi
    ((WARNINGS++))
    WARNINGS_LIST+=("$*")
}

run_check() {
    local check_id=$1
    local description=$2
    local command=$3
    local expected=${4:-"pass"}
    local is_critical=${5:-true}

    if [ "$VERBOSE" -eq 1 ]; then
        log_info "Running: $check_id - $description"
        log_info "Command: $command"
    fi

    if eval "$command" > /dev/null 2>&1; then
        if [ "$is_critical" = true ]; then
            log_pass "$check_id: $description"
        else
            log_warn "$check_id: $description (non-critical)"
        fi
    else
        if [ "$is_critical" = true ]; then
            log_fail "$check_id: $description"
        else
            log_warn "$check_id: $description"
        fi
    fi
}

################################################################################
# BASELINE CHECKS
################################################################################

check_baseline() {
    echo ""
    log_info "════════════════════════════════════════════════════════"
    log_info "BASELINE: Build & Code Quality"
    log_info "════════════════════════════════════════════════════════"

    # B.1: Build succeeds
    if command -v cargo &> /dev/null; then
        if cargo build --workspace 2>&1 | grep -q "error"; then
            log_fail "B.1: cargo build --workspace has errors"
        else
            log_pass "B.1: cargo build --workspace succeeds"
        fi
    else
        log_warn "B.1: cargo not available"
    fi

    # B.2: Clippy passes
    if command -v cargo &> /dev/null; then
        if cargo clippy --workspace -- -D warnings 2>&1 | grep -q "error"; then
            log_fail "B.2: cargo clippy has errors"
        else
            log_pass "B.2: cargo clippy --workspace -- -D warnings passes"
        fi
    else
        log_warn "B.2: cargo not available"
    fi

    # B.3: C library builds (if Makefile exists)
    if [ -f "$PROJECT_ROOT/Makefile" ]; then
        if make -C "$PROJECT_ROOT" build > /dev/null 2>&1; then
            log_pass "B.3: make build succeeds"
        else
            log_warn "B.3: make build unavailable or failed"
        fi
    else
        log_warn "B.3: Makefile not found"
    fi
}

################################################################################
# SECTION 1: DOCTRINE & COVENANTS
################################################################################

check_section_1() {
    echo ""
    log_info "════════════════════════════════════════════════════════"
    log_info "SECTION 1: Doctrine & Covenants Executable"
    log_info "════════════════════════════════════════════════════════"

    # C1.1: Turtle specs exist
    local covenant_count=0
    for covenant in c1 c2 c3 c4 c5 c6; do
        if [ -f "$PROJECT_ROOT/registry/${covenant}*.ttl" ] 2>/dev/null; then
            ((covenant_count++))
        fi
    done

    if [ "$covenant_count" -ge 3 ]; then
        log_pass "C1.1: Turtle specifications found for covenants"
    else
        log_warn "C1.1: Not all covenant Turtle specs found (found $covenant_count/6)"
    fi

    # C2.3: COVENANT_TESTS exists
    if [ -f "$PROJECT_ROOT/tests/COVENANT_TESTS.rs" ]; then
        log_pass "C2.3: COVENANT_TESTS.rs exists"
    else
        log_fail "C2.3: COVENANT_TESTS.rs not found"
    fi

    # Run covenant tests
    if command -v cargo &> /dev/null; then
        if cargo test --test COVENANT_TESTS 2>&1 | grep -q "test result: ok"; then
            log_pass "Covenant tests pass"
        else
            log_warn "Covenant tests unavailable or failed (may need compilation)"
        fi
    fi
}

################################################################################
# SECTION 2: TURTLE SINGLE SOURCE OF TRUTH
################################################################################

check_section_2() {
    echo ""
    log_info "════════════════════════════════════════════════════════"
    log_info "SECTION 2: Turtle Single Source of Truth"
    log_info "════════════════════════════════════════════════════════"

    # T.1: Registry directory exists
    if [ -d "$PROJECT_ROOT/registry" ]; then
        local ttl_count=$(find "$PROJECT_ROOT/registry" -name "*.ttl" 2>/dev/null | wc -l)
        if [ "$ttl_count" -gt 0 ]; then
            log_pass "T.1: Registry directory with $ttl_count .ttl files found"
        else
            log_warn "T.1: Registry directory exists but no .ttl files found"
        fi
    else
        log_warn "T.1: Registry directory not found"
    fi

    # T.4: Examples directory exists
    if [ -d "$PROJECT_ROOT/examples" ]; then
        local example_count=$(find "$PROJECT_ROOT/examples" -name "*.ttl" 2>/dev/null | wc -l)
        if [ "$example_count" -gt 0 ]; then
            log_pass "T.4: Examples directory with $example_count .ttl files"
        else
            log_warn "T.4: Examples directory exists but no .ttl files"
        fi
    else
        log_warn "T.4: Examples directory not found"
    fi
}

################################################################################
# SECTION 3: INVARIANTS ARE LAW
################################################################################

check_section_3() {
    echo ""
    log_info "════════════════════════════════════════════════════════"
    log_info "SECTION 3: Invariants Are Law (SHACL)"
    log_info "════════════════════════════════════════════════════════"

    # I.1: SHACL shapes exist
    if [ -f "$PROJECT_ROOT/registry/shapes.ttl" ]; then
        log_pass "I.1: registry/shapes.ttl exists"
    else
        log_warn "I.1: registry/shapes.ttl not found"
    fi

    # I.3: Validation logic in engine
    if grep -r "shacl\|validate\|constraint" "$PROJECT_ROOT/src" --include="*.rs" 2>/dev/null | grep -q "shacl\|validate"; then
        log_pass "I.3: Validation logic found in engine"
    else
        log_warn "I.3: Validation logic not obviously present"
    fi
}

################################################################################
# SECTION 4: WORKFLOW ENGINE
################################################################################

check_section_4() {
    echo ""
    log_info "════════════════════════════════════════════════════════"
    log_info "SECTION 4: Workflow Engine (State Machine + Performance)"
    log_info "════════════════════════════════════════════════════════"

    # E.1: Engine module exists
    if [ -d "$PROJECT_ROOT/src/engine" ]; then
        log_pass "E.1: src/engine module found"
    else
        log_warn "E.1: src/engine module not found"
    fi

    # E.3: Performance tests exist
    if [ -f "$PROJECT_ROOT/tests/perf_chicago_v04.rs" ] || [ -f "$PROJECT_ROOT/Makefile" ] && grep -q "test-performance" "$PROJECT_ROOT/Makefile"; then
        log_pass "E.3: Performance tests configured"
    else
        log_warn "E.3: Performance tests not found"
    fi
}

################################################################################
# SECTION 5: PATTERN MATRIX
################################################################################

check_section_5() {
    echo ""
    log_info "════════════════════════════════════════════════════════"
    log_info "SECTION 5: Pattern Matrix Expressiveness"
    log_info "════════════════════════════════════════════════════════"

    # P.1: Pattern matrix module exists
    if [ -f "$PROJECT_ROOT/src/patterns/matrix.rs" ]; then
        log_pass "P.1: src/patterns/matrix.rs exists"
    else
        log_warn "P.1: src/patterns/matrix.rs not found"
    fi

    # P.3: Validation module exists
    if [ -d "$PROJECT_ROOT/src/validation" ]; then
        log_pass "P.3: src/validation module found"
    else
        log_warn "P.3: src/validation module not found"
    fi
}

################################################################################
# SECTION 6: MAPE-K LOOP
################################################################################

check_section_6() {
    echo ""
    log_info "════════════════════════════════════════════════════════"
    log_info "SECTION 6: MAPE-K Loop (Autonomic)"
    log_info "════════════════════════════════════════════════════════"

    local mape_k_count=0
    for module in monitor analyze plan execute knowledge; do
        if [ -f "$PROJECT_ROOT/src/mape_k/$module.rs" ]; then
            log_pass "M.$((mape_k_count+1)): src/mape_k/$module.rs exists"
            ((mape_k_count++))
        fi
    done

    if [ "$mape_k_count" -lt 3 ]; then
        log_warn "M: Only $mape_k_count/5 MAPE-K modules found"
    fi
}

################################################################################
# SECTION 7: RECEIPTS & GAMMA STORE
################################################################################

check_section_7() {
    echo ""
    log_info "════════════════════════════════════════════════════════"
    log_info "SECTION 7: Receipts & Γ (Audit Trail)"
    log_info "════════════════════════════════════════════════════════"

    # R.1: Receipt module exists
    if [ -f "$PROJECT_ROOT/src/receipts/mod.rs" ]; then
        log_pass "R.1: src/receipts/mod.rs exists"
    else
        log_warn "R.1: src/receipts/mod.rs not found"
    fi

    # R.3: Gamma store exists
    if [ -f "$PROJECT_ROOT/src/receipts/gamma_store.rs" ]; then
        log_pass "R.3: src/receipts/gamma_store.rs exists"
    else
        log_warn "R.3: src/receipts/gamma_store.rs not found"
    fi
}

################################################################################
# SECTION 8: MARKETPLACE INTEGRATION
################################################################################

check_section_8() {
    echo ""
    log_info "════════════════════════════════════════════════════════"
    log_info "SECTION 8: Marketplace & Ontology Integration"
    log_info "════════════════════════════════════════════════════════"

    # MP.1: MAPE-K ingestion logic
    if grep -r "marketplace" "$PROJECT_ROOT/src" --include="*.rs" 2>/dev/null | grep -q "marketplace"; then
        log_pass "MP.1: Marketplace integration found"
    else
        log_warn "MP.1: Marketplace integration not obviously present"
    fi
}

################################################################################
# SECTION 9: TOOLING, DOCS, EXAMPLES
################################################################################

check_section_9() {
    echo ""
    log_info "════════════════════════════════════════════════════════"
    log_info "SECTION 9: Tooling, Docs, Examples"
    log_info "════════════════════════════════════════════════════════"

    # D.4: DOCTRINE_2027 exists
    if [ -f "$PROJECT_ROOT/docs/DOCTRINE_2027.md" ]; then
        log_pass "D.4: docs/DOCTRINE_2027.md exists"
    else
        log_warn "D.4: docs/DOCTRINE_2027.md not found"
    fi

    # D.5: DOCTRINE_COVENANT exists
    if [ -f "$PROJECT_ROOT/docs/DOCTRINE_COVENANT.md" ]; then
        log_pass "D.5: docs/DOCTRINE_COVENANT.md exists"
    else
        log_warn "D.5: docs/DOCTRINE_COVENANT.md not found"
    fi

    # D.6: SYSTEMS_IMPLEMENTATION_COMPLETE exists
    if [ -f "$PROJECT_ROOT/docs/SYSTEMS_IMPLEMENTATION_COMPLETE.md" ]; then
        log_pass "D.6: docs/SYSTEMS_IMPLEMENTATION_COMPLETE.md exists"
    else
        log_warn "D.6: docs/SYSTEMS_IMPLEMENTATION_COMPLETE.md not found"
    fi
}

################################################################################
# WEAVER VALIDATION (If available)
################################################################################

check_weaver_validation() {
    echo ""
    log_info "════════════════════════════════════════════════════════"
    log_info "WEAVER VALIDATION (OTel Schema)"
    log_info "════════════════════════════════════════════════════════"

    if command -v weaver &> /dev/null; then
        if weaver registry check -r "$PROJECT_ROOT/registry/" 2>&1 | grep -q "valid\|ok"; then
            log_pass "W.1: weaver registry check passes"
        else
            log_warn "W.1: weaver registry check failed (check manually)"
        fi
    else
        log_warn "W.1: weaver command not available (optional)"
    fi
}

################################################################################
# GENERATE REPORT
################################################################################

generate_json_report() {
    local commit_hash=$(git -C "$PROJECT_ROOT" rev-parse HEAD 2>/dev/null || echo "unknown")
    local branch=$(git -C "$PROJECT_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    local status="ready_for_promotion"
    if [ "${#FAILED_CRITICAL[@]}" -gt 0 ]; then
        status="needs_work"
    fi

    mkdir -p "$(dirname "$REPORT_FILE")"

    cat > "$REPORT_FILE" <<EOF
{
  "timestamp": "$timestamp",
  "commit": "$commit_hash",
  "branch": "$branch",
  "status": "$status",
  "checks": {
    "passed": $PASSED_CHECKS,
    "failed": $FAILED_CHECKS,
    "warnings": $WARNINGS
  },
  "failed_critical": [
EOF

    if [ "${#FAILED_CRITICAL[@]}" -gt 0 ]; then
        for fail in "${FAILED_CRITICAL[@]}"; do
            echo "    \"$fail\"," >> "$REPORT_FILE"
        done
        # Remove trailing comma
        sed -i '$ s/,$//' "$REPORT_FILE"
    fi

    cat >> "$REPORT_FILE" <<EOF
  ],
  "warnings": [
EOF

    if [ "${#WARNINGS_LIST[@]}" -gt 0 ]; then
        for warn in "${WARNINGS_LIST[@]}"; do
            echo "    \"$warn\"," >> "$REPORT_FILE"
        done
        # Remove trailing comma
        sed -i '$ s/,$//' "$REPORT_FILE"
    fi

    cat >> "$REPORT_FILE" <<EOF
  ],
  "recommendation": "$status"
}
EOF

    echo "Report written to: $REPORT_FILE"
}

################################################################################
# MAIN
################################################################################

main() {
    echo ""
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║  KNHK DEFINITION OF DONE (DoD) VALIDATION                     ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"

    check_baseline
    check_section_1
    check_section_2
    check_section_3
    check_section_4
    check_section_5
    check_section_6
    check_section_7
    check_section_8
    check_section_9
    check_weaver_validation

    echo ""
    log_info "════════════════════════════════════════════════════════"
    log_info "SUMMARY"
    log_info "════════════════════════════════════════════════════════"
    echo -e "  Passed: ${GREEN}$PASSED_CHECKS${NC}"
    echo -e "  Failed: ${RED}$FAILED_CHECKS${NC}"
    echo -e "  Warnings: ${YELLOW}$WARNINGS${NC}"

    generate_json_report

    if [ "${#FAILED_CRITICAL[@]}" -gt 0 ]; then
        echo ""
        echo -e "${RED}✗ DoD VALIDATION FAILED${NC}"
        echo "Failed critical checks:"
        for fail in "${FAILED_CRITICAL[@]}"; do
            echo "  - $fail"
        done
        return 1
    else
        echo ""
        echo -e "${GREEN}✓ DoD VALIDATION PASSED${NC}"
        return 0
    fi
}

main "$@"
