#!/bin/bash
# Gate 0: Pre-Flight Validation (3 minutes)
# Catches P0 blockers before any agent work starts
# Part of DFLSS waste elimination strategy
# Enhanced with Poka-Yoke error-proofing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🚦 Gate 0: Pre-Flight Validation + Poka-Yoke"
echo "============================================="
echo "Location: $PROJECT_ROOT"
echo ""

START_TIME=$(date +%s)

# Poka-Yoke #1: No unwrap() in production code
echo "→ [Poka-Yoke 1/4] Checking for unwrap() in production..."
unwraps=$(grep -r "\.unwrap()" rust/*/src --include="*.rs" 2>/dev/null | grep -v test | grep -v "cli" | grep -v "examples" || true)
if [ -n "$unwraps" ]; then
  echo "❌ BLOCKER: unwrap() found in production code"
  echo "$unwraps"
  echo "Fix: Replace with proper error handling (? operator or if-let)"
  exit 1
fi
echo "  ✅ No unwrap() in production"

# Poka-Yoke #2: No unimplemented!()
echo "→ [Poka-Yoke 2/4] Checking for unimplemented!()..."
unimpl=$(grep -r "unimplemented!" rust/*/src --include="*.rs" 2>/dev/null || true)
if [ -n "$unimpl" ]; then
  echo "❌ BLOCKER: unimplemented!() found"
  echo "$unimpl"
  echo "Fix: Complete implementation before committing"
  exit 1
fi
echo "  ✅ No unimplemented!() placeholders"

# Poka-Yoke #3: No println! in production
echo "→ [Poka-Yoke 3/4] Checking for println! in production..."
printlns=$(grep -r "println!" rust/*/src --include="*.rs" 2>/dev/null | grep -v "///" | grep -v "// " | grep -v "/test" | grep -v "/tests/" | grep -v "cli" | grep -v "example" | grep -v "main.rs" | grep -v "advisor_example.rs" | grep -v "stress.rs" || true)
if [ -n "$printlns" ]; then
  echo "❌ BLOCKER: println! found in production code"
  echo "$printlns"
  echo "Fix: Use tracing::info! or tracing::debug! instead"
  exit 1
fi
echo "  ✅ No println! in production"

# Poka-Yoke #4: Check for fake Ok(()) near TODOs
echo "→ [Poka-Yoke 4/4] Checking for suspicious Ok(()) returns..."
fake_oks=$(grep -r "Ok(())" rust/*/src --include="*.rs" 2>/dev/null | grep -A2 -B2 "todo!\|unimplemented!\|FIXME\|TODO" || true)
if [ -n "$fake_oks" ]; then
  echo "⚠️  WARNING: Suspicious Ok(()) returns found near TODOs"
  echo "$fake_oks"
  echo "Review these to ensure they're not masking incomplete implementations"
fi
echo "  ✅ Poka-Yoke checks passed"
echo ""

# 1. Compilation check (1 min)
echo "→ [1/3] Checking compilation..."
COMPILE_START=$(date +%s)
COMPILE_FAILED=0
cd rust
for crate in knhk-etl knhk-cli knhk-otel; do
  if [ -d "$crate" ]; then
    if ! cargo check --manifest-path "$crate/Cargo.toml" --quiet 2>&1; then
      echo "❌ BLOCKER: Compilation failed in $crate"
      cargo check --manifest-path "$crate/Cargo.toml"
      COMPILE_FAILED=1
      break
    fi
  fi
done
cd "$PROJECT_ROOT"
if [ $COMPILE_FAILED -eq 1 ]; then
  exit 1
fi
COMPILE_END=$(date +%s)
COMPILE_TIME=$((COMPILE_END - COMPILE_START))
echo "  ✅ Compilation OK (${COMPILE_TIME}s)"

# 2. Clippy check (1 min)
echo "→ [2/3] Checking code quality..."
CLIPPY_START=$(date +%s)
CLIPPY_FAILED=0
cd rust
for crate in knhk-etl knhk-cli knhk-otel; do
  if [ -d "$crate" ]; then
    if ! cargo clippy --manifest-path "$crate/Cargo.toml" --quiet -- -D warnings 2>&1; then
      echo "❌ BLOCKER: Clippy warnings in $crate"
      cargo clippy --manifest-path "$crate/Cargo.toml" -- -D warnings
      CLIPPY_FAILED=1
      break
    fi
  fi
done
cd "$PROJECT_ROOT"
if [ $CLIPPY_FAILED -eq 1 ]; then
  exit 1
fi
CLIPPY_END=$(date +%s)
CLIPPY_TIME=$((CLIPPY_END - CLIPPY_START))
echo "  ✅ Code quality OK (${CLIPPY_TIME}s)"

# 3. Quick smoke tests (1 min)
echo "→ [3/3] Running quick smoke tests..."
TEST_START=$(date +%s)
TEST_FAILED=0
cd rust
for crate in knhk-etl knhk-cli knhk-otel; do
  if [ -d "$crate" ]; then
    if ! cargo test --manifest-path "$crate/Cargo.toml" --lib --quiet 2>&1; then
      echo "❌ BLOCKER: Tests failing in $crate"
      cargo test --manifest-path "$crate/Cargo.toml" --lib
      TEST_FAILED=1
      break
    fi
  fi
done
cd "$PROJECT_ROOT"
if [ $TEST_FAILED -eq 1 ]; then
  exit 1
fi
TEST_END=$(date +%s)
TEST_TIME=$((TEST_END - TEST_START))
echo "  ✅ Smoke tests OK (${TEST_TIME}s)"

END_TIME=$(date +%s)
TOTAL_TIME=$((END_TIME - START_TIME))

echo ""
echo "✅ Gate 0 PASSED: Ready for agent work"
echo "   Total time: ${TOTAL_TIME}s (target: <180s)"
echo "   Breakdown: compile=${COMPILE_TIME}s, clippy=${CLIPPY_TIME}s, test=${TEST_TIME}s"

if [ $TOTAL_TIME -gt 180 ]; then
  echo "⚠️  Warning: Gate 0 exceeded 3-minute target"
fi

exit 0
