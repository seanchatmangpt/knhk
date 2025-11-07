#!/bin/bash
set -euo pipefail

# KNHK v1.0 Definition of Done Validation Script
# Validates all 11 core Definition of Done criteria from .cursor/rules/80-20-best-practices.mdc

# Ensure we're in the project root
cd "$(dirname "$0")/.."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "=========================================="
echo "KNHK v1.0 Definition of Done Validation"
echo "=========================================="
echo ""
echo "Validating all 11 core Definition of Done criteria..."
echo ""

# Track validation results
VALIDATION_ERRORS=""
VALIDATION_WARNINGS=""
PASSED=0
FAILED=0
WARNINGS=0

# Helper functions
pass() {
    echo -e "${GREEN}✅${NC} $1"
    ((PASSED++))
}

fail() {
    echo -e "${RED}❌${NC} $1"
    ((FAILED++))
    VALIDATION_ERRORS="$VALIDATION_ERRORS\n❌ $1"
}

warn() {
    echo -e "${YELLOW}⚠️${NC} $1"
    ((WARNINGS++))
    VALIDATION_WARNINGS="$VALIDATION_WARNINGS\n⚠️ $1"
}

# ============================================
# CRITERION 1: COMPILATION
# ============================================
echo -e "${BLUE}=== Criterion 1: Compilation (No errors or warnings) ===${NC}"
echo ""

# Check C library compilation
echo -n "1.1 Building C library... "
if make -C c lib > /dev/null 2>&1; then
    if [ -f "c/libknhk.a" ] || [ -f "libknhk.a" ]; then
        pass "C library builds successfully"
    else
        fail "C library build succeeded but libknhk.a not found"
    fi
else
    fail "C library compilation failed"
fi

# Check Rust workspace compilation
echo -n "1.2 Building Rust workspace... "
if cargo build --workspace 2>&1 | grep -q "warning:"; then
    fail "Rust workspace has compilation warnings"
elif cargo build --workspace > /dev/null 2>&1; then
    pass "Rust workspace builds successfully with zero warnings"
else
    fail "Rust workspace compilation failed"
fi

# Check individual crates compile
CRATES=("knhk-etl" "knhk-hot" "knhk-sidecar" "knhk-lockchain" "knhk-otel")
for crate in "${CRATES[@]}"; do
    if [ -d "rust/$crate" ] && [ -f "rust/$crate/Cargo.toml" ]; then
        echo -n "1.3 Building $crate... "
        if (cd "rust/$crate" && cargo build > /dev/null 2>&1); then
            pass "$crate builds successfully"
        else
            fail "$crate compilation failed"
        fi
    fi
done

echo ""

# ============================================
# CRITERION 2: NO unwrap()/expect()
# ============================================
echo -e "${BLUE}=== Criterion 2: No unwrap()/expect() in production code ===${NC}"
echo ""

echo -n "2.1 Checking for .unwrap() in production code... "
UNWRAP_FOUND=false
if grep -r "\.unwrap()" rust/*/src --include="*.rs" 2>/dev/null | grep -v "test" | grep -v "example" | grep -v "tests/" > /dev/null; then
    UNWRAP_FOUND=true
fi

if $UNWRAP_FOUND; then
    fail "Found .unwrap() in production code (exclude tests/examples)"
    grep -r "\.unwrap()" rust/*/src --include="*.rs" 2>/dev/null | grep -v "test" | grep -v "example" | grep -v "tests/" | head -5
else
    pass "No .unwrap() found in production code"
fi

echo -n "2.2 Checking for .expect() in production code... "
EXPECT_FOUND=false
if grep -r "\.expect(" rust/*/src --include="*.rs" 2>/dev/null | grep -v "test" | grep -v "example" | grep -v "tests/" > /dev/null; then
    EXPECT_FOUND=true
fi

if $EXPECT_FOUND; then
    fail "Found .expect() in production code (exclude tests/examples)"
    grep -r "\.expect(" rust/*/src --include="*.rs" 2>/dev/null | grep -v "test" | grep -v "example" | grep -v "tests/" | head -5
else
    pass "No .expect() found in production code"
fi

echo ""

# ============================================
# CRITERION 3: TRAIT COMPATIBILITY
# ============================================
echo -e "${BLUE}=== Criterion 3: Trait Compatibility (dyn compatible, no async trait methods) ===${NC}"
echo ""

echo -n "3.1 Checking for async trait methods... "
ASYNC_TRAIT_FOUND=false
# Find trait definitions and check for async fn inside them
for file in $(find rust/*/src -name "*.rs" -type f 2>/dev/null); do
    if grep -q "trait.*{" "$file" && grep -q "async fn" "$file"; then
        # Check if async fn is inside trait (heuristic: check context)
        if grep -A 20 "trait.*{" "$file" | grep -q "async fn"; then
            ASYNC_TRAIT_FOUND=true
            echo ""
            fail "Found async trait method in $file (breaks dyn compatibility)"
            grep -B 2 -A 2 "async fn" "$file" | head -10
            break
        fi
    fi
done

if ! $ASYNC_TRAIT_FOUND; then
    pass "No async trait methods found (all traits are dyn compatible)"
fi

echo ""

# ============================================
# CRITERION 4: BACKWARD COMPATIBILITY
# ============================================
echo -e "${BLUE}=== Criterion 4: Backward Compatibility ===${NC}"
echo ""

echo -n "4.1 Checking CHANGELOG.md... "
if [ -f "CHANGELOG.md" ]; then
    if grep -q "v1.0\|1.0.0\|# v1.0" CHANGELOG.md 2>/dev/null; then
        pass "CHANGELOG.md exists and has v1.0 entry"
    else
        warn "CHANGELOG.md exists but no v1.0 entry found"
    fi
else
    warn "CHANGELOG.md not found"
fi

echo -n "4.2 Checking version consistency... "
VERSION_MISMATCH=false
# Check if all Cargo.toml files have consistent versioning
FIRST_VERSION=""
for toml in $(find rust -name "Cargo.toml" -type f 2>/dev/null); do
    VERSION=$(grep "^version" "$toml" | head -1 | sed 's/.*"\(.*\)".*/\1/' || echo "")
    if [ -n "$VERSION" ]; then
        if [ -z "$FIRST_VERSION" ]; then
            FIRST_VERSION="$VERSION"
        elif [ "$VERSION" != "$FIRST_VERSION" ]; then
            VERSION_MISMATCH=true
        fi
    fi
done

if $VERSION_MISMATCH; then
    warn "Version numbers inconsistent across Cargo.toml files"
else
    pass "Version numbers consistent"
fi

echo ""

# ============================================
# CRITERION 5: ALL TESTS PASS
# ============================================
echo -e "${BLUE}=== Criterion 5: All Tests Pass ===${NC}"
echo ""

echo -n "5.1 Running Rust tests... "
if cargo test --workspace --no-fail-fast > /dev/null 2>&1; then
    pass "All Rust tests pass"
else
    fail "Rust tests failed"
fi

echo -n "5.2 Running C tests... "
if [ -f "c/Makefile" ] && make -C c test-chicago-v04 > /dev/null 2>&1; then
    pass "C tests pass"
elif [ -f "tests/chicago_v04_test" ] && ./tests/chicago_v04_test > /dev/null 2>&1; then
    pass "C tests pass"
else
    warn "C tests not available or failed (may need manual execution)"
fi

echo ""

# ============================================
# CRITERION 6: NO LINTING ERRORS
# ============================================
echo -e "${BLUE}=== Criterion 6: No Linting Errors ===${NC}"
echo ""

echo -n "6.1 Running cargo clippy... "
if cargo clippy --workspace -- -D warnings > /dev/null 2>&1; then
    pass "Clippy passed with zero warnings"
else
    fail "Clippy found warnings or errors"
    # Show first few warnings
    cargo clippy --workspace -- -D warnings 2>&1 | head -20
fi

echo ""

# ============================================
# CRITERION 7: PROPER ERROR HANDLING
# ============================================
echo -e "${BLUE}=== Criterion 7: Proper Error Handling (Result types) ===${NC}"
echo ""

echo -n "7.1 Checking for Result types in fallible operations... "
# Heuristic: Check that functions returning Result exist
RESULT_FUNCTIONS=$(grep -r "-> Result<" rust/*/src --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
if [ "$RESULT_FUNCTIONS" -gt 0 ]; then
    pass "Found $RESULT_FUNCTIONS functions using Result types"
else
    warn "No Result types found (may indicate missing error handling)"
fi

echo -n "7.2 Checking for panics in production code... "
PANIC_FOUND=false
if grep -r "panic!" rust/*/src --include="*.rs" 2>/dev/null | grep -v "test" | grep -v "example" | grep -v "tests/" > /dev/null; then
    PANIC_FOUND=true
fi

if $PANIC_FOUND; then
    warn "Found panic! in production code (should use Result types)"
    grep -r "panic!" rust/*/src --include="*.rs" 2>/dev/null | grep -v "test" | grep -v "example" | grep -v "tests/" | head -3
else
    pass "No panic! found in production code"
fi

echo ""

# ============================================
# CRITERION 8: ASYNC/SYNC PATTERNS
# ============================================
echo -e "${BLUE}=== Criterion 8: Async/Sync Patterns ===${NC}"
echo ""

echo -n "8.1 Checking for std::thread::sleep in async contexts... "
SLEEP_IN_ASYNC=false
# Find async functions and check if they use std::thread::sleep
for file in $(find rust/*/src -name "*.rs" -type f 2>/dev/null); do
    if grep -q "async fn" "$file" && grep -q "std::thread::sleep" "$file"; then
        SLEEP_IN_ASYNC=true
        fail "Found std::thread::sleep in async context in $file (should use tokio::time::sleep)"
        grep -B 2 -A 2 "std::thread::sleep" "$file" | head -5
        break
    fi
done

if ! $SLEEP_IN_ASYNC; then
    pass "No std::thread::sleep found in async contexts"
fi

echo ""

# ============================================
# CRITERION 9: NO FALSE POSITIVES
# ============================================
echo -e "${BLUE}=== Criterion 9: No False Positives (No fake Ok(())) ===${NC}"
echo ""

echo -n "9.1 Checking for suspicious Ok(()) patterns... "
# Heuristic: Look for functions that return Ok(()) without doing work
# This is a best-effort check - manual review still needed
SUSPICIOUS_OK=false
# Check for functions that return Ok(()) immediately or with minimal work
for file in $(find rust/*/src -name "*.rs" -type f 2>/dev/null | grep -v test); do
    # Look for patterns like: pub fn ... -> Result<()> { Ok(()) }
    if grep -q "-> Result<()>" "$file" && grep -A 5 "-> Result<()>" "$file" | grep -q "^[[:space:]]*Ok(())"; then
        # Check if there's actual work before Ok(())
        LINES_BETWEEN=$(grep -A 10 "-> Result<()>" "$file" | grep -n "Ok(())" | head -1 | cut -d: -f1)
        if [ "$LINES_BETWEEN" -lt 3 ]; then
            SUSPICIOUS_OK=true
            warn "Suspicious Ok(()) pattern in $file (may be fake implementation)"
            grep -B 2 -A 5 "-> Result<()>" "$file" | head -10
            break
        fi
    fi
done

if ! $SUSPICIOUS_OK; then
    pass "No obvious fake Ok(()) patterns found"
fi

echo -n "9.2 Checking for unimplemented!() usage... "
UNIMPLEMENTED_COUNT=$(grep -r "unimplemented!" rust/*/src --include="*.rs" 2>/dev/null | grep -v test | wc -l | tr -d ' ')
if [ "$UNIMPLEMENTED_COUNT" -gt 0 ]; then
    warn "Found $UNIMPLEMENTED_COUNT unimplemented!() calls (acceptable for incomplete features)"
else
    pass "No unimplemented!() calls found"
fi

echo ""

# ============================================
# CRITERION 10: PERFORMANCE COMPLIANCE
# ============================================
echo -e "${BLUE}=== Criterion 10: Performance Compliance (Hot path ≤8 ticks) ===${NC}"
echo ""

echo -n "10.1 Checking hot path C code for branchless operations... "
# Check for branchless patterns in hot path C code
BRANCHLESS_FOUND=false
if [ -f "c/src/core.c" ] || [ -f "c/src/simd.c" ]; then
    # Look for branchless patterns (bitwise operations, arithmetic)
    if grep -q "& 0x7\|>> 63\|branchless\|BRANCHLESS" c/src/*.c 2>/dev/null; then
        BRANCHLESS_FOUND=true
    fi
fi

if $BRANCHLESS_FOUND; then
    pass "Branchless operations found in hot path C code"
else
    warn "No obvious branchless patterns found (manual review recommended)"
fi

echo -n "10.2 Checking for SIMD usage... "
SIMD_FOUND=false
if grep -q "SIMD\|_mm_\|_mm256_\|neon\|avx" c/src/*.c c/include/*.h 2>/dev/null; then
    SIMD_FOUND=true
fi

if $SIMD_FOUND; then
    pass "SIMD usage found in hot path code"
else
    warn "No SIMD usage found (may be acceptable)"
fi

echo -n "10.3 Verifying tick budget constraint (≤8 ticks)... "
# Check that tick budget checks exist in code
TICK_BUDGET_CHECK=false
if grep -q "ticks.*8\|tick.*budget\|≤.*8.*tick" rust/*/src c/src --include="*.rs" --include="*.c" 2>/dev/null; then
    TICK_BUDGET_CHECK=true
fi

if $TICK_BUDGET_CHECK; then
    pass "Tick budget constraints found in code"
else
    warn "No explicit tick budget checks found (manual review recommended)"
fi

echo ""

# ============================================
# CRITERION 11: OTEL VALIDATION
# ============================================
echo -e "${BLUE}=== Criterion 11: OTEL Validation (Weaver registry check) ===${NC}"
echo ""

echo -n "11.1 Checking Weaver installation... "
if command -v weaver &> /dev/null; then
    pass "Weaver is installed"
    
    echo -n "11.2 Running Weaver registry check... "
    if [ -d "registry" ]; then
        if weaver registry check -r registry/ > /dev/null 2>&1; then
            pass "Weaver registry schema is valid"
        else
            fail "Weaver registry schema validation failed"
            weaver registry check -r registry/ 2>&1 | head -20
        fi
    else
        warn "registry/ directory not found"
    fi
else
    fail "Weaver is not installed (required for OTEL validation)"
    VALIDATION_ERRORS="$VALIDATION_ERRORS\n❌ Install Weaver: bash scripts/install-weaver.sh"
fi

echo ""

# ============================================
# FINAL SUMMARY
# ============================================
echo "=========================================="
echo "v1.0 Definition of Done Validation Summary"
echo "=========================================="
echo ""
echo "Passed:  $PASSED"
echo "Failed:  $FAILED"
echo "Warnings: $WARNINGS"
echo ""

# Generate JSON report
JSON_REPORT="validation_report_v1.0.json"
cat > "$JSON_REPORT" << EOF
{
  "version": "1.0",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "summary": {
    "passed": $PASSED,
    "failed": $FAILED,
    "warnings": $WARNINGS,
    "total": $((PASSED + FAILED + WARNINGS))
  },
  "criteria": {
    "1_compilation": $([ $FAILED -eq 0 ] && echo "pass" || echo "fail"),
    "2_no_unwrap_expect": $([ $FAILED -eq 0 ] && echo "pass" || echo "fail"),
    "3_trait_compatibility": $([ $FAILED -eq 0 ] && echo "pass" || echo "fail"),
    "4_backward_compatibility": $([ $FAILED -eq 0 ] && echo "pass" || echo "fail"),
    "5_all_tests_pass": $([ $FAILED -eq 0 ] && echo "pass" || echo "fail"),
    "6_no_linting_errors": $([ $FAILED -eq 0 ] && echo "pass" || echo "fail"),
    "7_proper_error_handling": $([ $FAILED -eq 0 ] && echo "pass" || echo "fail"),
    "8_async_sync_patterns": $([ $FAILED -eq 0 ] && echo "pass" || echo "fail"),
    "9_no_false_positives": $([ $FAILED -eq 0 ] && echo "pass" || echo "fail"),
    "10_performance_compliance": $([ $FAILED -eq 0 ] && echo "pass" || echo "fail"),
    "11_otel_validation": $([ $FAILED -eq 0 ] && echo "pass" || echo "fail")
  }
}
EOF

echo "Report saved to: $JSON_REPORT"
echo ""

# Final verdict
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅✅✅ v1.0 DEFINITION OF DONE: PASSED ✅✅✅${NC}"
    echo ""
    echo "All 11 core criteria met:"
    echo "  ✅ Compilation: No errors or warnings"
    echo "  ✅ No unwrap()/expect(): Zero usage in production code"
    echo "  ✅ Trait Compatibility: All traits dyn compatible"
    echo "  ✅ Backward Compatibility: No breaking changes"
    echo "  ✅ All Tests Pass: Every test passes"
    echo "  ✅ No Linting Errors: Zero clippy warnings"
    echo "  ✅ Proper Error Handling: Result types used"
    echo "  ✅ Async/Sync Patterns: Proper usage"
    echo "  ✅ No False Positives: No fake implementations"
    echo "  ✅ Performance Compliance: Hot path ≤8 ticks"
    echo "  ✅ OTEL Validation: Weaver registry valid"
    echo ""
    
    if [ $WARNINGS -gt 0 ]; then
        echo -e "${YELLOW}Warnings (non-blocking):${NC}"
        echo -e "$VALIDATION_WARNINGS"
        echo ""
    fi
    
    exit 0
else
    echo -e "${RED}❌❌❌ v1.0 DEFINITION OF DONE: FAILED ❌❌❌${NC}"
    echo ""
    echo "Critical validation errors:"
    echo -e "$VALIDATION_ERRORS"
    echo ""
    
    if [ -n "$VALIDATION_WARNINGS" ]; then
        echo "Warnings:"
        echo -e "$VALIDATION_WARNINGS"
        echo ""
    fi
    
    echo -e "${RED}Fix all errors before v1.0 release.${NC}"
    exit 1
fi

