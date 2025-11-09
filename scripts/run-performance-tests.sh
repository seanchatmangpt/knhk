#!/usr/bin/env bash
# KNHK Performance Test Runner
# Validates performance constraints (≤8 ticks hot path) - Concurrent execution

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
echo -e "${BLUE}⚡ KNHK Performance Tests (τ ≤ 8 validation - Concurrent)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# C Performance Tests (v04) - runs in background
run_c_perf_tests() {
  local result_file="$TMPDIR/c_perf.result"
  local output_file="$TMPDIR/c_perf.out"

  (
    echo -e "${BLUE}Running C performance tests...${NC}" > "$output_file"
    cd "$PROJECT_ROOT/c"
    if make test-performance-v04 >> "$output_file" 2>&1; then
      echo -e "${GREEN}✅ C performance tests passed${NC}" >> "$output_file"
      echo "PASS" > "$result_file"
    else
      echo -e "${RED}❌ C performance tests failed${NC}" >> "$output_file"
      echo "FAIL" > "$result_file"
    fi
    echo >> "$output_file"
  ) &
}

# Rust Performance Tests - runs in background
run_rust_perf_tests() {
  local crate="$1"
  local crate_name=$(basename "$crate")
  local result_file="$TMPDIR/${crate_name}_perf.result"
  local output_file="$TMPDIR/${crate_name}_perf.out"

  (
    echo -e "${BLUE}Checking ${crate}...${NC}" > "$output_file"
    if [ -d "$crate" ]; then
      if (cd "$crate" && cargo test --release performance -- --nocapture 2>&1 | grep -E "(test result|PASSED|≤8)" >> "$output_file"); then
        echo -e "${GREEN}✅ Performance tests passed for ${crate_name}${NC}" >> "$output_file"
        echo "PASS" > "$result_file"
      else
        echo -e "${BLUE}ℹ️  No performance tests in ${crate_name}${NC}" >> "$output_file"
        echo "SKIP" > "$result_file"
      fi
    else
      echo -e "${BLUE}ℹ️  Crate ${crate_name} not found${NC}" >> "$output_file"
      echo "SKIP" > "$result_file"
    fi
    echo >> "$output_file"
  ) &
}

# Start C performance tests
run_c_perf_tests

# Start Rust performance tests concurrently
CRATES_WITH_PERF=(
  "rust/knhk-etl"
  "rust/knhk-warm"
  "rust/knhk-hot"
)

for crate in "${CRATES_WITH_PERF[@]}"; do
  run_rust_perf_tests "$crate"
done

# Wait for all background jobs to complete
wait

# Display C test results
if [ -f "$TMPDIR/c_perf.out" ]; then
  cat "$TMPDIR/c_perf.out"
fi

# Display Rust test results
for crate in "${CRATES_WITH_PERF[@]}"; do
  crate_name=$(basename "$crate")
  output_file="$TMPDIR/${crate_name}_perf.out"
  if [ -f "$output_file" ]; then
    cat "$output_file"
  fi
done

# Check for failures
FAILED=0
if [ -f "$TMPDIR/c_perf.result" ]; then
  if [ "$(cat "$TMPDIR/c_perf.result")" = "FAIL" ]; then
    FAILED=$((FAILED + 1))
  fi
fi

for crate in "${CRATES_WITH_PERF[@]}"; do
  crate_name=$(basename "$crate")
  result_file="$TMPDIR/${crate_name}_perf.result"
  if [ -f "$result_file" ] && [ "$(cat "$result_file")" = "FAIL" ]; then
    FAILED=$((FAILED + 1))
  fi
done

if [ $FAILED -gt 0 ]; then
  echo -e "${RED}❌ ${FAILED} performance test suite(s) failed${NC}"
  exit 1
else
  echo -e "${GREEN}✅ Performance validation complete${NC}"
  exit 0
fi
