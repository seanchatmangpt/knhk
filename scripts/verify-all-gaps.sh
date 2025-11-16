#!/bin/bash
# Comprehensive verification script to fill all gaps
# This script verifies everything that was missed

set -e

cd "$(dirname "$0")/.." || exit 1
PROJECT_ROOT=$(pwd)

echo "=== Filling All Gaps - Comprehensive Verification ==="
echo "Starting at: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
echo ""

# Gap 1: Verify compilation
echo "=== Gap 1: Compilation Verification ==="
cd rust
if cargo check --workspace 2>&1 | tee /tmp/cargo_check.log | grep -q "error"; then
    echo "❌ FAILED: Compilation errors found"
    grep "error" /tmp/cargo_check.log | head -20
    FAILED=1
else
    echo "✅ PASSED: All packages compile successfully"
    FAILED=0
fi
cd "$PROJECT_ROOT"
echo ""

# Gap 2: Verify clippy
echo "=== Gap 2: Clippy Verification ==="
cd rust
if cargo clippy --workspace -- -D warnings 2>&1 | tee /tmp/cargo_clippy.log | grep -qE "error|warning"; then
    echo "❌ FAILED: Clippy warnings/errors found"
    grep -E "error|warning" /tmp/cargo_clippy.log | head -20
    FAILED=1
else
    echo "✅ PASSED: No clippy warnings"
    FAILED=0
fi
cd "$PROJECT_ROOT"
echo ""

# Gap 3: Verify test compilation
echo "=== Gap 3: Test Compilation Verification ==="
cd rust
if cargo test --workspace --no-run 2>&1 | tee /tmp/cargo_test.log | grep -q "error"; then
    echo "❌ FAILED: Test compilation errors found"
    grep "error" /tmp/cargo_test.log | head -20
    FAILED=1
else
    echo "✅ PASSED: All tests compile successfully"
    FAILED=0
fi
cd "$PROJECT_ROOT"
echo ""

# Gap 4: Verify Chicago TDD test files exist
echo "=== Gap 4: Chicago TDD Test Files Verification ==="
CHICAGO_TDD_FILES=$(find rust -name "*chicago_tdd*.rs" -type f | sort)
CHICAGO_TDD_COUNT=$(echo "$CHICAGO_TDD_FILES" | wc -l | tr -d ' ')

if [ "$CHICAGO_TDD_COUNT" -gt 0 ]; then
    echo "✅ PASSED: Found $CHICAGO_TDD_COUNT Chicago TDD test files:"
    echo "$CHICAGO_TDD_FILES" | while read file; do
        if [ -f "$file" ]; then
            echo "  ✓ $file"
        else
            echo "  ❌ MISSING: $file"
            FAILED=1
        fi
    done
else
    echo "❌ FAILED: No Chicago TDD test files found"
    FAILED=1
fi
echo ""

# Gap 5: Verify Chicago TDD tests compile
echo "=== Gap 5: Chicago TDD Test Compilation ==="
for dir in rust/knhk-etl rust/knhk-validation rust/knhk-sidecar; do
    if [ -d "$dir" ]; then
        echo "Checking $dir..."
        cd "$dir"
        if cargo test --no-run 2>&1 | grep -q "error"; then
            echo "❌ FAILED: Test compilation errors in $dir"
            cargo test --no-run 2>&1 | grep "error" | head -10
            FAILED=1
        else
            echo "✅ PASSED: $dir tests compile"
        fi
        cd "$PROJECT_ROOT"
    fi
done
echo ""

# Gap 6: Check for unwrap()/expect() in production code
echo "=== Gap 6: Unwrap()/Expect() in Production Code ==="
UNWRAP_FILES=$(find rust -name "*.rs" -path "*/src/*" -not -path "*/target/*" -exec grep -l "\.unwrap()\|\.expect(" {} \; 2>/dev/null | sort)
UNWRAP_COUNT=$(echo "$UNWRAP_FILES" | grep -v "^$" | wc -l | tr -d ' ')

if [ "$UNWRAP_COUNT" -gt 0 ]; then
    echo "⚠️  WARNING: Found unwrap()/expect() in $UNWRAP_COUNT production files:"
    echo "$UNWRAP_FILES" | head -10
    echo "   (Review each file to ensure they're in acceptable contexts)"
else
    echo "✅ PASSED: No unwrap()/expect() in production code"
fi
echo ""

# Gap 7: Check for async trait methods
echo "=== Gap 7: Async Trait Methods Check ==="
ASYNC_TRAIT_FILES=$(find rust -name "*.rs" -path "*/src/*" -not -path "*/target/*" -exec grep -l "async fn.*trait\|trait.*async fn" {} \; 2>/dev/null | sort)
ASYNC_TRAIT_COUNT=$(echo "$ASYNC_TRAIT_FILES" | grep -v "^$" | wc -l | tr -d ' ')

if [ "$ASYNC_TRAIT_COUNT" -gt 0 ]; then
    echo "❌ FAILED: Found async trait methods in $ASYNC_TRAIT_COUNT files (not dyn compatible):"
    echo "$ASYNC_TRAIT_FILES"
    FAILED=1
else
    echo "✅ PASSED: No async trait methods found"
fi
echo ""

# Gap 8: Check for unimplemented!()
echo "=== Gap 8: Unimplemented!() Check ==="
UNIMPLEMENTED_FILES=$(grep -r "unimplemented!" rust/*/src/*.rs 2>/dev/null | grep -v "//.*unimplemented" | cut -d: -f1 | sort -u)
UNIMPLEMENTED_COUNT=$(echo "$UNIMPLEMENTED_FILES" | grep -v "^$" | wc -l | tr -d ' ')

if [ "$UNIMPLEMENTED_COUNT" -gt 0 ]; then
    echo "⚠️  WARNING: Found unimplemented!() in $UNIMPLEMENTED_COUNT files:"
    echo "$UNIMPLEMENTED_FILES" | head -10
    echo "   (Review to ensure they're acceptable)"
else
    echo "✅ PASSED: No unimplemented!() in production code"
fi
echo ""

# Gap 9: Check for panic!()
echo "=== Gap 9: Panic!() Check ==="
PANIC_FILES=$(find rust -name "*.rs" -path "*/src/*" -not -path "*/target/*" -exec grep -l "panic!" {} \; 2>/dev/null | sort)
PANIC_COUNT=$(echo "$PANIC_FILES" | grep -v "^$" | wc -l | tr -d ' ')

if [ "$PANIC_COUNT" -gt 0 ]; then
    echo "⚠️  WARNING: Found panic!() in $PANIC_COUNT production files:"
    echo "$PANIC_FILES" | head -10
    echo "   (Review to ensure they're in acceptable contexts)"
else
    echo "✅ PASSED: No panic!() in production code"
fi
echo ""

# Gap 10: Verify fixes are correct
echo "=== Gap 10: Verify Fixes Are Correct ==="
echo "Checking knhk-hot/src/ring_ffi.rs for uppercase variables..."
if grep -q "\blet S =\|\blet P =\|\blet O =" rust/knhk-hot/src/ring_ffi.rs 2>/dev/null; then
    echo "❌ FAILED: Still found uppercase S, P, O variables in ring_ffi.rs"
    grep -n "\blet S =\|\blet P =\|\blet O =" rust/knhk-hot/src/ring_ffi.rs | head -5
    FAILED=1
else
    echo "✅ PASSED: No uppercase S, P, O variables found"
fi

echo "Checking knhk-lockchain/src/lib.rs for MerkleError export..."
if grep -q "pub use merkle::.*MerkleError" rust/knhk-lockchain/src/lib.rs 2>/dev/null; then
    echo "✅ PASSED: MerkleError properly exported"
else
    echo "❌ FAILED: MerkleError export not found"
    FAILED=1
fi

echo "Checking BeatScheduler for Debug trait..."
if grep -q "#\[derive(Debug)\]" rust/knhk-etl/src/beat_scheduler.rs 2>/dev/null; then
    echo "✅ PASSED: Debug trait found on BeatScheduler"
else
    echo "❌ FAILED: Debug trait not found on BeatScheduler"
    FAILED=1
fi
echo ""

# Summary
echo "=== Verification Summary ==="
if [ "$FAILED" -eq 0 ]; then
    echo "✅ ALL GAPS FILLED - All verifications passed!"
    exit 0
else
    echo "❌ SOME GAPS REMAIN - Review failures above"
    exit 1
fi

