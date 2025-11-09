#!/usr/bin/env bash
# KNHK Chicago TDD Test Runner
# Executes Chicago-style TDD tests concurrently (Rust)

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

# Temporary directory for test results
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🧪 KNHK Chicago TDD Tests (Rust - Concurrent)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# Ensure C library is built first (required for knhk-hot)
echo -e "${BLUE}Ensuring C library is built...${NC}"
cd "$PROJECT_ROOT/c"
if make lib >/dev/null 2>&1; then
  echo -e "${GREEN}✅ C library ready${NC}"
else
  echo -e "${RED}❌ C library build failed${NC}"
  exit 1
fi
echo

# Run Chicago TDD tests in knhk-etl
cd "$PROJECT_ROOT/rust/knhk-etl"
echo -e "${BLUE}Running Chicago TDD tests in knhk-etl (concurrently)...${NC}"

# Find all Chicago TDD test files
CHICAGO_TESTS=$(find tests -name "chicago_tdd_*.rs" 2>/dev/null || true)

if [ -z "$CHICAGO_TESTS" ]; then
  echo -e "${RED}❌ No Chicago TDD tests found in rust/knhk-etl/tests/${NC}"
  exit 1
fi

echo "Found Chicago TDD tests:"
echo "$CHICAGO_TESTS" | sed 's/^/  • /'
echo

# Test a single Chicago test file (runs in background)
test_chicago_file() {
  local test_file="$1"
  local test_name=$(basename "$test_file" .rs)
  local result_file="$TMPDIR/${test_name}.result"
  local output_file="$TMPDIR/${test_name}.out"

  (
    echo -e "${BLUE}┌─ Running ${test_name}...${NC}" > "$output_file"

    if cargo test --test "$test_name" --quiet 2>&1 >> "$output_file"; then
      echo -e "${GREEN}└─ ✅ PASSED${NC}" >> "$output_file"
      echo "PASS" > "$result_file"
    else
      echo -e "${RED}└─ ❌ FAILED${NC}" >> "$output_file"
      echo "FAIL" > "$result_file"
    fi
    echo >> "$output_file"
  ) &
}

# Run all Chicago tests concurrently
FAILED=0
for test_file in $CHICAGO_TESTS; do
  test_chicago_file "$test_file"
done

# Wait for all background jobs to complete
wait

# Collect results and display output
for test_file in $CHICAGO_TESTS; do
  test_name=$(basename "$test_file" .rs)
  result_file="$TMPDIR/${test_name}.result"
  output_file="$TMPDIR/${test_name}.out"

  if [ -f "$output_file" ]; then
    cat "$output_file"
  fi

  if [ -f "$result_file" ]; then
    result=$(cat "$result_file")
    if [ "$result" = "FAIL" ]; then
      FAILED=$((FAILED + 1))
    fi
  fi
done

# Summary
if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}✅ All Chicago TDD tests passed!${NC}"
  exit 0
else
  echo -e "${RED}❌ ${FAILED} Chicago TDD test(s) failed${NC}"
  exit 1
fi
