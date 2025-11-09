#!/usr/bin/env bash
# KNHK Integration Test Runner
# Executes integration tests concurrently (C + Rust)

set -euo pipefail

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
echo -e "${BLUE}🔗 KNHK Integration Tests (Concurrent)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# C Integration Tests (v2) - runs in background
run_c_integration_tests() {
  local result_file="$TMPDIR/c_integration.result"
  local output_file="$TMPDIR/c_integration.out"

  (
    echo -e "${BLUE}Running C integration tests (v2)...${NC}" > "$output_file"
    cd "$PROJECT_ROOT/c"
    if make test-integration-v2 >> "$output_file" 2>&1; then
      echo -e "${GREEN}✅ C integration tests passed${NC}" >> "$output_file"
      echo "PASS" > "$result_file"
    else
      echo -e "${RED}❌ C integration tests failed${NC}" >> "$output_file"
      echo "FAIL" > "$result_file"
    fi
    echo >> "$output_file"
  ) &
}

# Rust Integration Tests - runs in background
run_rust_integration_tests() {
  local result_file="$TMPDIR/rust_integration.result"
  local output_file="$TMPDIR/rust_integration.out"

  (
    echo -e "${BLUE}Running Rust integration tests...${NC}" > "$output_file"
    if [ -d "$PROJECT_ROOT/rust/knhk-integration-tests" ]; then
      cd "$PROJECT_ROOT/rust/knhk-integration-tests"
      if cargo test --quiet 2>&1 >> "$output_file"; then
        echo -e "${GREEN}✅ Rust integration tests passed${NC}" >> "$output_file"
        echo "PASS" > "$result_file"
      else
        echo -e "${RED}❌ Rust integration tests failed${NC}" >> "$output_file"
        echo "FAIL" > "$result_file"
      fi
    else
      echo -e "${BLUE}ℹ️  No dedicated Rust integration test crate${NC}" >> "$output_file"
      echo "SKIP" > "$result_file"
    fi
    echo >> "$output_file"
  ) &
}

# Start both test suites concurrently
run_c_integration_tests
run_rust_integration_tests

# Wait for all background jobs to complete
wait

# Display results
if [ -f "$TMPDIR/c_integration.out" ]; then
  cat "$TMPDIR/c_integration.out"
fi

if [ -f "$TMPDIR/rust_integration.out" ]; then
  cat "$TMPDIR/rust_integration.out"
fi

# Check for failures
FAILED=0
if [ -f "$TMPDIR/c_integration.result" ]; then
  if [ "$(cat "$TMPDIR/c_integration.result")" = "FAIL" ]; then
    FAILED=$((FAILED + 1))
  fi
fi

if [ -f "$TMPDIR/rust_integration.result" ]; then
  if [ "$(cat "$TMPDIR/rust_integration.result")" = "FAIL" ]; then
    FAILED=$((FAILED + 1))
  fi
fi

echo
if [ $FAILED -gt 0 ]; then
  echo -e "${RED}❌ ${FAILED} integration test suite(s) failed${NC}"
  exit 1
else
  echo -e "${GREEN}✅ Integration validation complete${NC}"
  exit 0
fi
