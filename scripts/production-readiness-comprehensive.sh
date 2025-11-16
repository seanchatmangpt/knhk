#!/bin/bash
# Comprehensive Production Readiness Validation
# Implements the complete Definition of Done checklist
# Following KNHK validation hierarchy: Weaver → Compilation → Tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Results tracking
PASSED_CHECKS=0
FAILED_CHECKS=0
WARNING_CHECKS=0
TOTAL_CHECKS=0

declare -a FAILURES
declare -a WARNINGS

check() {
    local name="$1"
    local command="$2"
    local level="$3"  # CRITICAL, HIGH, MEDIUM, LOW

    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    echo -n "[$TOTAL_CHECKS] $name... "

    if eval "$command" > /tmp/check_$TOTAL_CHECKS.log 2>&1; then
        echo -e "${GREEN}✅ PASSED${NC}"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        return 0
    else
        if [ "$level" = "CRITICAL" ]; then
            echo -e "${RED}❌ FAILED (CRITICAL)${NC}"
            FAILED_CHECKS=$((FAILED_CHECKS + 1))
            FAILURES+=("$name (CRITICAL)")
        else
            echo -e "${YELLOW}⚠️  WARNING${NC}"
            WARNING_CHECKS=$((WARNING_CHECKS + 1))
            WARNINGS+=("$name")
        fi
        return 1
    fi
}

echo "=========================================="
echo "KNHK Production Readiness Validation"
echo "=========================================="
echo ""
echo "Definition of Done Validation:"
echo "  LEVEL 1: Weaver Validation (Source of Truth)"
echo "  LEVEL 2: Compilation & Code Quality (Baseline)"
echo "  LEVEL 3: Traditional Tests (Supporting Evidence)"
echo ""

cd "$PROJECT_ROOT"

# ============================================
# LEVEL 1: WEAVER VALIDATION (MANDATORY)
# ============================================
echo -e "${BLUE}=== LEVEL 1: Weaver Validation (MANDATORY - Source of Truth) ===${NC}"
echo ""

check "Weaver installed" "command -v weaver" "CRITICAL"
check "Weaver registry schema valid" "weaver registry check -r registry/" "CRITICAL"

# Note: live-check requires running application
echo ""
echo -e "${YELLOW}Note: Weaver live-check requires running application${NC}"
echo "      Run: weaver registry live-check --registry registry/"
echo "      After deploying application to validate runtime telemetry"
echo ""

# ============================================
# LEVEL 2: COMPILATION & CODE QUALITY
# ============================================
echo -e "${BLUE}=== LEVEL 2: Compilation & Code Quality (Baseline) ===${NC}"
echo ""

# Check Rust compilation
check "Cargo build (workspace)" "cargo build --workspace --all-features" "CRITICAL"
check "Cargo build (release)" "cargo build --workspace --release" "CRITICAL"

# Check C compilation
if [ -d "c" ] && [ -f "c/Makefile" ]; then
    check "C library build" "cd c && make clean && make lib" "CRITICAL"
    check "C tests compile" "cd c && make tests" "HIGH"
fi

# Code quality checks
check "Clippy (zero warnings)" "cargo clippy --workspace --all-features -- -D warnings" "CRITICAL"
check "Rustfmt check" "cargo fmt --all -- --check" "HIGH"

# Pattern checks
echo ""
echo "Checking for unsafe code patterns..."
check "No .unwrap() in production" "! grep -r '\.unwrap()' rust/*/src --include='*.rs' | grep -v test | grep -v example" "CRITICAL"
check "No .expect() in production" "! grep -r '\.expect(' rust/*/src --include='*.rs' | grep -v test | grep -v example" "CRITICAL"
check "No println! in production" "! grep -r 'println!' rust/*/src --include='*.rs' | grep -v test | grep -v example" "HIGH"
check "No unimplemented! in production" "! grep -r 'unimplemented!' rust/*/src --include='*.rs' | grep -v test" "CRITICAL"
check "No panic! in production" "! grep -r 'panic!' rust/*/src --include='*.rs' | grep -v test" "HIGH"
check "No TODO in production" "! grep -r 'TODO' rust/*/src --include='*.rs' | grep -v test" "MEDIUM"
check "No FIXME in production" "! grep -r 'FIXME' rust/*/src --include='*.rs' | grep -v test" "MEDIUM"

# ============================================
# LEVEL 3: TRADITIONAL TESTS
# ============================================
echo -e "${BLUE}=== LEVEL 3: Traditional Tests (Supporting Evidence) ===${NC}"
echo ""

# Rust tests
check "Cargo test (workspace)" "cargo test --workspace --all-features" "HIGH"

# Integration tests
if [ -d "rust/knhk-integration-tests" ]; then
    check "Integration tests" "cd rust/knhk-integration-tests && cargo test" "HIGH"
fi

# C tests
if [ -d "c/tests" ]; then
    check "C unit tests" "cd c && make test" "HIGH"
fi

# Chicago TDD tests
if [ -f "Makefile" ] && grep -q "test-chicago" Makefile; then
    check "Chicago TDD tests" "make test-chicago-v04" "HIGH"
fi

# Performance tests
if [ -f "Makefile" ] && grep -q "test-performance" Makefile; then
    check "Performance tests (≤8 ticks)" "make test-performance-v04" "CRITICAL"
fi

# ============================================
# FUNCTIONAL VALIDATION
# ============================================
echo -e "${BLUE}=== Functional Validation ===${NC}"
echo ""

# Check that commands exist and can execute
if [ -f "target/release/knhk" ]; then
    check "KNHK CLI exists" "test -f target/release/knhk" "CRITICAL"
    check "KNHK --version works" "target/release/knhk --version" "CRITICAL"
    check "KNHK --help works" "target/release/knhk --help" "HIGH"

    echo ""
    echo -e "${YELLOW}⚠️  CRITICAL: --help ≠ Working Feature${NC}"
    echo "    Execute commands with REAL arguments to validate functionality"
    echo "    Example: knhk workflow execute <file.ttl>"
    echo ""
fi

# ============================================
# INFRASTRUCTURE VALIDATION
# ============================================
echo -e "${BLUE}=== Infrastructure Validation ===${NC}"
echo ""

# Check for required dependencies
check "Docker installed" "command -v docker" "HIGH"
check "Cargo installed" "command -v cargo" "CRITICAL"
check "Make installed" "command -v make" "HIGH"

# Check registry structure
check "Registry manifest exists" "test -f registry/registry_manifest.yaml" "CRITICAL"
check "Workflow schema exists" "test -f registry/knhk-workflow-engine.yaml" "HIGH"
check "Operation schema exists" "test -f registry/knhk-operation.yaml" "HIGH"

# ============================================
# DEPLOYMENT READINESS
# ============================================
echo -e "${BLUE}=== Deployment Readiness ===${NC}"
echo ""

# Check for deployment artifacts
check "Release binary exists" "test -f target/release/knhk" "CRITICAL"
check "Documentation exists" "test -f README.md" "MEDIUM"
check "License exists" "test -f LICENSE || test -f LICENSE.md || test -f COPYING" "MEDIUM"

# Check for secrets in code
echo ""
echo "Checking for potential secrets in code..."
check "No hardcoded passwords" "! grep -ri 'password.*=.*[\"']' rust/*/src --include='*.rs' | grep -v test" "CRITICAL"
check "No hardcoded API keys" "! grep -ri 'api.*key.*=.*[\"']' rust/*/src --include='*.rs' | grep -v test" "CRITICAL"
check "No .env files in git" "! git ls-files | grep -E '\.env$'" "HIGH"

# ============================================
# CONSTITUTIONAL VALIDATION
# ============================================
echo -e "${BLUE}=== Constitutional Constraints ===${NC}"
echo ""

echo "Note: These are mathematical constraints that should be verified:"
echo "  □ A = μ(O) - Action equals hook projection"
echo "  □ μ∘μ = μ - Idempotence"
echo "  □ O ⊨ Σ - Typing satisfaction"
echo "  □ μ ⊂ τ, τ ≤ 8 - Epoch containment (Chatman Constant)"
echo "  □ hash(A) = hash(μ(O)) - Provenance"
echo ""
echo "These are validated through:"
echo "  - Performance tests (Chatman Constant)"
echo "  - Receipt verification (Provenance)"
echo "  - Schema validation (Typing)"
echo "  - Weaver live-check (Runtime behavior)"
echo ""

# ============================================
# FINAL SUMMARY
# ============================================
echo "=========================================="
echo "Production Readiness Summary"
echo "=========================================="
echo ""

PASS_RATE=$((PASSED_CHECKS * 100 / TOTAL_CHECKS))

echo "Results:"
echo "  ✅ Passed: $PASSED_CHECKS/$TOTAL_CHECKS ($PASS_RATE%)"
echo "  ❌ Failed: $FAILED_CHECKS"
echo "  ⚠️  Warnings: $WARNING_CHECKS"
echo ""

if [ $FAILED_CHECKS -eq 0 ]; then
    echo -e "${GREEN}✅✅✅ PRODUCTION READY ✅✅✅${NC}"
    echo ""
    echo "All critical checks passed!"
    echo ""

    if [ $WARNING_CHECKS -gt 0 ]; then
        echo -e "${YELLOW}Warnings (non-blocking):${NC}"
        for warning in "${WARNINGS[@]}"; do
            echo "  ⚠️  $warning"
        done
        echo ""
    fi

    echo "Next steps:"
    echo "  1. Deploy application to production environment"
    echo "  2. Run: weaver registry live-check --registry registry/"
    echo "  3. Execute commands with real arguments to validate"
    echo "  4. Monitor telemetry and performance metrics"
    echo ""
    exit 0
else
    echo -e "${RED}❌❌❌ NOT PRODUCTION READY ❌❌❌${NC}"
    echo ""
    echo "Critical failures:"
    for failure in "${FAILURES[@]}"; do
        echo "  ❌ $failure"
    done
    echo ""

    if [ $WARNING_CHECKS -gt 0 ]; then
        echo "Warnings:"
        for warning in "${WARNINGS[@]}"; do
            echo "  ⚠️  $warning"
        done
        echo ""
    fi

    echo "Check logs in /tmp/check_*.log for details"
    echo ""
    echo "Fix all critical failures before deploying to production."
    exit 1
fi
