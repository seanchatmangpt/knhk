#!/bin/bash
# Fix v1.0 Release Blockers

set -e

echo "=== v1.0 Blocker Remediation Script ==="
echo "Starting at: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
echo ""

cd "$(dirname "$0")/.." || exit 1
PROJECT_ROOT=$(pwd)

# Phase 1: Fix Rust Compilation Issues
echo "=== Phase 1: Fix Rust Compilation Issues ==="
echo ""

# 1a. Fix naming issues
echo "1. Applying cargo fix for naming issues..."
cd rust/knhk-etl || exit 1
if cargo fix --lib -p knhk-etl --allow-dirty --allow-staged 2>&1 | tail -20; then
    echo "✅ Cargo fix applied successfully"
else
    echo "⚠️  Cargo fix had some issues (check manually)"
fi
cd "$PROJECT_ROOT" || exit 1
echo ""

# 1b. Verify clippy
echo "2. Verifying clippy after fix..."
cd rust/knhk-etl || exit 1
CLIPPY_ERRORS=$(cargo clippy --workspace -- -D warnings 2>&1 | grep -c "^error:" || true)
if [ "$CLIPPY_ERRORS" -eq 0 ]; then
    echo "✅ Clippy: PASSED (zero errors)"
else
    echo "⚠️  Clippy: $CLIPPY_ERRORS errors remaining"
    echo "   Manual intervention required"
fi
cd "$PROJECT_ROOT" || exit 1
echo ""

# 1c. List remaining test compilation errors
echo "3. Checking test compilation..."
cd rust/knhk-etl || exit 1
echo "   Attempting to compile tests..."
TEST_ERRORS=$(cargo test --workspace --no-run 2>&1 | grep -c "^error:" || true)
if [ "$TEST_ERRORS" -eq 0 ]; then
    echo "✅ Test compilation: PASSED"
else
    echo "⚠️  Test compilation: $TEST_ERRORS errors remaining"
    echo "   Manual fixes required:"
    echo "   - Add #[derive(Debug)] to BeatScheduler"
    echo "   - Implement stop_streaming() or update tests"
    echo "   - Fix trait bound errors"
fi
cd "$PROJECT_ROOT" || exit 1
echo ""

# Phase 2: Fix C Build System
echo "=== Phase 2: Fix C Build System ==="
echo ""

echo "4. Checking C Makefile..."
if [ -f "c/Makefile" ]; then
    cd c || exit 1

    # Check if build target exists
    if grep -q "^build:" Makefile; then
        echo "✅ Build target exists in Makefile"
    else
        echo "⚠️  Build target missing - needs manual addition"
        echo "   Add to Makefile:"
        echo "   build: libknhk.a"
    fi

    # Check for missing test files
    echo ""
    echo "   Checking for missing test files..."
    MISSING_TESTS=$(make -q test-config-v05 2>&1 | grep "No rule to make target" || true)
    if [ -n "$MISSING_TESTS" ]; then
        echo "⚠️  Missing test files detected:"
        echo "$MISSING_TESTS" | sed 's/^/   /'
    else
        echo "✅ All test dependencies present"
    fi

    cd "$PROJECT_ROOT" || exit 1
else
    echo "⚠️  C Makefile not found at c/Makefile"
fi
echo ""

# Summary
echo "=== Remediation Summary ==="
echo ""
echo "✅ Completed automated fixes:"
echo "   - cargo fix applied to naming issues"
echo ""
echo "⚠️  Manual intervention required for:"
echo "   1. Remaining clippy errors (if any)"
echo "   2. Test compilation errors:"
echo "      - Add #[derive(Debug)] to BeatScheduler"
echo "      - Implement missing test methods"
echo "   3. C Makefile build target (if missing)"
echo "   4. Missing C test source files (if any)"
echo ""
echo "📋 Next Steps:"
echo "   1. Review and fix remaining issues manually"
echo "   2. Run: ./scripts/v1_final_validation.sh"
echo "   3. Verify all checks pass"
echo "   4. Update V1_RELEASE_VALIDATION_CHECKLIST.md"
echo ""
echo "Completed at: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
