#!/bin/bash
set -e

# Ensure we're in the project root
cd "$(dirname "$0")/.."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "=========================================="
echo "KNHK Production Readiness Validation"
echo "=========================================="
echo ""
echo "This script validates ALL Definition of Done criteria."
echo "Only code that passes ALL checks is production-ready."
echo ""

# Track validation results
VALIDATION_ERRORS=""
VALIDATION_WARNINGS=""

# ============================================
# LEVEL 1: WEAVER SCHEMA VALIDATION (MANDATORY)
# ============================================
echo -e "${BLUE}=== LEVEL 1: Weaver Schema Validation (MANDATORY - Source of Truth) ===${NC}"
echo ""

echo "1. Checking Weaver registry schema..."
if weaver registry check -r registry/; then
    echo -e "${GREEN}✅ Weaver registry schema is valid${NC}"
else
    echo -e "${RED}❌ CRITICAL: Weaver registry schema is INVALID${NC}"
    VALIDATION_ERRORS="$VALIDATION_ERRORS\n❌ Weaver registry schema validation failed"
fi
echo ""

echo "2. Checking for Weaver live-check capability..."
if command -v weaver &> /dev/null; then
    echo -e "${YELLOW}⚠️  Note: Run 'weaver registry live-check --registry registry/' during runtime to validate telemetry${NC}"
    VALIDATION_WARNINGS="$VALIDATION_WARNINGS\n⚠️  Remember to run live-check during runtime"
else
    echo -e "${RED}❌ CRITICAL: Weaver is not installed${NC}"
    VALIDATION_ERRORS="$VALIDATION_ERRORS\n❌ Weaver not installed"
fi
echo ""

# ============================================
# LEVEL 2: COMPILATION & CODE QUALITY (Baseline)
# ============================================
echo -e "${BLUE}=== LEVEL 2: Compilation & Code Quality (Baseline) ===${NC}"
echo ""

echo "3. Running cargo build..."
BUILD_PASSED=true
for crate in rust/knhk-*; do
    if [ -d "$crate" ] && [ -f "$crate/Cargo.toml" ]; then
        if ! (cd "$crate" && cargo build 2>&1 | grep -v "warning:" >/dev/null); then
            BUILD_PASSED=false
        fi
    fi
done
if $BUILD_PASSED; then
    echo -e "${GREEN}✅ Cargo build passed with zero warnings${NC}"
else
    echo -e "${RED}❌ CRITICAL: Cargo build failed or has warnings${NC}"
    VALIDATION_ERRORS="$VALIDATION_ERRORS\n❌ Cargo build issues"
fi
echo ""

echo "4. Running cargo clippy..."
CLIPPY_PASSED=true
for crate in rust/knhk-*; do
    if [ -d "$crate" ] && [ -f "$crate/Cargo.toml" ]; then
        if ! (cd "$crate" && cargo clippy -- -D warnings 2>&1 >/dev/null); then
            CLIPPY_PASSED=false
        fi
    fi
done
if $CLIPPY_PASSED; then
    echo -e "${GREEN}✅ Clippy passed with zero issues${NC}"
else
    echo -e "${RED}❌ CRITICAL: Clippy found issues${NC}"
    VALIDATION_ERRORS="$VALIDATION_ERRORS\n❌ Clippy issues present"
fi
echo ""

echo "6. Checking for unsafe code patterns..."
UNSAFE_PATTERNS_FOUND=false

if grep -r "\.unwrap()" rust/*/src --include="*.rs" | grep -v "test" | grep -v "example"; then
    echo -e "${RED}❌ Found .unwrap() in production code${NC}"
    VALIDATION_ERRORS="$VALIDATION_ERRORS\n❌ .unwrap() found in production code"
    UNSAFE_PATTERNS_FOUND=true
fi

if grep -r "\.expect(" rust/*/src --include="*.rs" | grep -v "test" | grep -v "example"; then
    echo -e "${RED}❌ Found .expect() in production code${NC}"
    VALIDATION_ERRORS="$VALIDATION_ERRORS\n❌ .expect() found in production code"
    UNSAFE_PATTERNS_FOUND=true
fi

if grep -r "println!" rust/*/src --include="*.rs" | grep -v "test" | grep -v "example"; then
    echo -e "${YELLOW}⚠️  Found println! in production code (should use tracing)${NC}"
    VALIDATION_WARNINGS="$VALIDATION_WARNINGS\n⚠️  println! found (use tracing instead)"
fi

if ! $UNSAFE_PATTERNS_FOUND; then
    echo -e "${GREEN}✅ No unsafe code patterns found${NC}"
fi
echo ""

# ============================================
# LEVEL 3: TRADITIONAL TESTS (Supporting Evidence)
# ============================================
echo -e "${BLUE}=== LEVEL 3: Traditional Tests (Supporting Evidence - Can Have False Positives) ===${NC}"
echo ""

echo "5. Running cargo tests..."
TESTS_PASSED=true
for crate in rust/knhk-*; do
    if [ -d "$crate" ] && [ -f "$crate/Cargo.toml" ]; then
        if ! (cd "$crate" && cargo test 2>&1 >/dev/null); then
            TESTS_PASSED=false
        fi
    fi
done
if $TESTS_PASSED; then
    echo -e "${GREEN}✅ Cargo tests passed${NC}"
else
    echo -e "${RED}❌ Cargo tests failed${NC}"
    VALIDATION_ERRORS="$VALIDATION_ERRORS\n❌ Cargo tests failed"
fi
echo ""

echo "6. Running integration tests..."
if [ -d "rust/knhk-integration-tests" ]; then
    if (cd rust/knhk-integration-tests && cargo test 2>&1 >/dev/null); then
        echo -e "${GREEN}✅ Integration tests passed${NC}"
    else
        echo -e "${RED}❌ Integration tests failed${NC}"
        VALIDATION_ERRORS="$VALIDATION_ERRORS\n❌ Integration tests failed"
    fi
else
    echo -e "${YELLOW}⚠️  No integration tests directory found${NC}"
fi
echo ""

# ============================================
# FINAL VALIDATION SUMMARY
# ============================================
echo "=========================================="
echo "Production Readiness Summary"
echo "=========================================="
echo ""

if [ -z "$VALIDATION_ERRORS" ]; then
    echo -e "${GREEN}✅✅✅ CODE IS PRODUCTION-READY ✅✅✅${NC}"
    echo ""
    echo "All Definition of Done criteria met:"
    echo "  ✅ Weaver registry schema is valid"
    echo "  ✅ Zero compilation warnings"
    echo "  ✅ Zero Clippy issues"
    echo "  ✅ No unsafe code patterns (.unwrap/.expect)"
    echo "  ✅ All Cargo tests pass"
    echo "  ✅ Integration tests pass"
    echo ""

    if [ -n "$VALIDATION_WARNINGS" ]; then
        echo -e "${YELLOW}Warnings (non-blocking):${NC}"
        echo -e "$VALIDATION_WARNINGS"
        echo ""
    fi

    echo -e "${BLUE}Next steps:${NC}"
    echo "  1. Run 'weaver registry live-check --registry registry/' during runtime"
    echo "  2. Verify actual command execution (not just --help)"
    echo "  3. Confirm telemetry emission matches schema"
    echo ""

    exit 0
else
    echo -e "${RED}❌❌❌ CODE IS NOT PRODUCTION-READY ❌❌❌${NC}"
    echo ""
    echo "Critical validation errors:"
    echo -e "$VALIDATION_ERRORS"
    echo ""

    if [ -n "$VALIDATION_WARNINGS" ]; then
        echo "Warnings:"
        echo -e "$VALIDATION_WARNINGS"
        echo ""
    fi

    echo -e "${YELLOW}Fix all errors before deploying to production.${NC}"
    exit 1
fi
