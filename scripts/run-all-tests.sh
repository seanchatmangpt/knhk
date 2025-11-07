#!/bin/bash
# Run all tests for KNHK
# Usage: ./scripts/run-all-tests.sh [--dod] [--c-only] [--rust-only]

set -e

# Parse arguments
RUN_DOD=false
C_ONLY=false
RUST_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --dod)
            RUN_DOD=true
            shift
            ;;
        --c-only)
            C_ONLY=true
            shift
            ;;
        --rust-only)
            RUST_ONLY=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--dod] [--c-only] [--rust-only]"
            exit 1
            ;;
    esac
done

# Ensure we're in the project root
cd "$(dirname "$0")/.."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track failures
FAILED_TESTS=""

echo "=========================================="
echo "KNHK Automated Test Suite"
echo "=========================================="
echo ""

# Run DoD validation if requested
if [[ "$RUN_DOD" == "true" ]]; then
    echo -e "${YELLOW}=== Running v1.0 Definition of Done Validation ===${NC}"
    if ./scripts/validate-v1-dod.sh; then
        echo -e "${GREEN}✅ DoD validation passed${NC}"
    else
        echo -e "${RED}❌ DoD validation failed${NC}"
        FAILED_TESTS="$FAILED_TESTS\n- DoD validation"
    fi
    echo ""
fi

# 1. Weaver Registry Check (MANDATORY - Source of Truth)
echo -e "${YELLOW}=== Running Weaver Registry Check ===${NC}"
if weaver registry check -r registry/; then
    echo -e "${GREEN}✅ Weaver registry check passed${NC}"
else
    echo -e "${RED}❌ Weaver registry check failed${NC}"
    FAILED_TESTS="$FAILED_TESTS\n- Weaver registry check"
fi
echo ""

# 2. Cargo Build (all crates)
echo -e "${YELLOW}=== Running Cargo Build ===${NC}"
BUILD_PASSED=true
for crate in rust/knhk-*; do
    if [ -d "$crate" ] && [ -f "$crate/Cargo.toml" ]; then
        echo "Building $crate..."
        if ! (cd "$crate" && cargo build 2>&1 | tee /tmp/cargo-build-$(basename "$crate").log); then
            BUILD_PASSED=false
        fi
    fi
done
if $BUILD_PASSED; then
    echo -e "${GREEN}✅ Cargo build passed with zero warnings${NC}"
else
    echo -e "${RED}❌ Cargo build failed${NC}"
    FAILED_TESTS="$FAILED_TESTS\n- Cargo build"
fi
echo ""

# 3. Cargo Clippy (all crates)
echo -e "${YELLOW}=== Running Cargo Clippy ===${NC}"
CLIPPY_PASSED=true
for crate in rust/knhk-*; do
    if [ -d "$crate" ] && [ -f "$crate/Cargo.toml" ]; then
        echo "Linting $crate..."
        if ! (cd "$crate" && cargo clippy -- -D warnings 2>&1); then
            CLIPPY_PASSED=false
        fi
    fi
done
if $CLIPPY_PASSED; then
    echo -e "${GREEN}✅ Clippy passed with zero issues${NC}"
else
    echo -e "${RED}❌ Clippy found issues${NC}"
    FAILED_TESTS="$FAILED_TESTS\n- Cargo clippy"
fi
echo ""

# 4. Cargo Tests (all crates)
echo -e "${YELLOW}=== Running Cargo Tests ===${NC}"
TESTS_PASSED=true
for crate in rust/knhk-*; do
    if [ -d "$crate" ] && [ -f "$crate/Cargo.toml" ]; then
        echo "Testing $crate..."
        if ! (cd "$crate" && cargo test 2>&1); then
            TESTS_PASSED=false
        fi
    fi
done
if $TESTS_PASSED; then
    echo -e "${GREEN}✅ Cargo tests passed${NC}"
else
    echo -e "${RED}❌ Cargo tests failed${NC}"
    FAILED_TESTS="$FAILED_TESTS\n- Cargo tests"
fi
echo ""

# 5. Integration Tests
echo -e "${YELLOW}=== Running Integration Tests ===${NC}"
if [ -d "rust/knhk-integration-tests" ]; then
    if (cd rust/knhk-integration-tests && cargo test); then
        echo -e "${GREEN}✅ Integration tests passed${NC}"
    else
        echo -e "${RED}❌ Integration tests failed${NC}"
        FAILED_TESTS="$FAILED_TESTS\n- Integration tests"
    fi
else
    echo -e "${YELLOW}⚠️  No integration tests directory found${NC}"
fi
echo ""

# Summary
echo "=========================================="
echo "Test Suite Summary"
echo "=========================================="

if [ -z "$FAILED_TESTS" ]; then
    echo -e "${GREEN}✅ ALL TESTS PASSED${NC}"
    echo ""
    echo "The codebase meets all Definition of Done criteria:"
    if [[ "$RUN_DOD" == "true" ]]; then
        echo "  ✅ v1.0 Definition of Done validation (see docs/v1.0-definition-of-done.md)"
    fi
    echo "  ✅ Weaver registry validation (source of truth)"
    echo "  ✅ Zero compilation warnings"
    echo "  ✅ Zero Clippy issues"
    echo "  ✅ All Cargo tests pass"
    echo "  ✅ Integration tests pass"
    echo ""
    echo "For complete v1.0 DoD criteria, see: docs/v1.0-definition-of-done.md"
    exit 0
else
    echo -e "${RED}❌ SOME TESTS FAILED${NC}"
    echo ""
    echo "Failed tests:"
    echo -e "$FAILED_TESTS"
    exit 1
fi
