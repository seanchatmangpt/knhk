#!/usr/bin/env bash
# KNHK Rust Test Runner
# Executes all Rust crate tests concurrently (no workspace)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TOTAL_CRATES=0
PASSED_CRATES=0
FAILED_CRATES=()

# Temporary directory for test results
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🧪 KNHK Rust Test Suite Runner (Concurrent)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# Project root
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

# Ensure C library is built first (required for knhk-hot)
echo -e "${BLUE}Ensuring C library is built...${NC}"
cd "$PROJECT_ROOT/c"
if make lib >/dev/null 2>&1; then
  echo -e "${GREEN}✅ C library ready${NC}"
  echo
else
  echo -e "${YELLOW}⚠️  C library build failed, some crates may fail${NC}"
  echo
fi
cd "$PROJECT_ROOT"

# Fast mode: use minimal features for slow crates (knhk-cli, knhk-workflow-engine)
# Set FAST_MODE=0 to use full features
FAST_MODE=${FAST_MODE:-1}

# All Rust crates (ordered by dependency)
CRATES=(
  "rust/knhk-config"
  "rust/knhk-otel"
  "rust/knhk-aot"
  "rust/knhk-hot"
  "rust/knhk-warm"
  "rust/knhk-unrdf"
  "rust/knhk-lockchain"
  "rust/knhk-validation"
  "rust/knhk-connectors"
  "rust/knhk-etl"
  "rust/knhk-sidecar"
  "rust/knhk-cli"
  "rust/knhk-workflow-engine"
  "rust/knhk-integration-tests"
)

if [ "$FAST_MODE" = "1" ]; then
  echo -e "${YELLOW}⚡ Fast mode: Using minimal features for CLI and workflow-engine${NC}"
  echo -e "${YELLOW}   Set FAST_MODE=0 to use full features${NC}"
  echo
fi

# Test a single crate (runs in background)
test_crate() {
  local crate_path="$1"
  local crate_name=$(basename "$crate_path")
  local result_file="$TMPDIR/${crate_name}.result"
  local output_file="$TMPDIR/${crate_name}.out"

  (
    echo -e "${BLUE}┌─ Testing ${crate_name}...${NC}" > "$output_file"

    if [ ! -d "$crate_path" ]; then
      echo -e "${YELLOW}└─ ⚠️  Crate not found, skipping${NC}" >> "$output_file"
      echo "SKIP" > "$result_file"
    elif [ ! -f "$crate_path/Cargo.toml" ]; then
      echo -e "${YELLOW}└─ ⚠️  No Cargo.toml, skipping${NC}" >> "$output_file"
      echo "SKIP" > "$result_file"
    else
      # Run tests (lib tests only for speed, incremental compilation)
      # Use minimal features for slow crates in fast mode
      local features=""
      if [ "$FAST_MODE" = "1" ]; then
        case "$crate_name" in
          knhk-cli)
            features="--features minimal"
            ;;
          knhk-workflow-engine)
            features="--no-default-features"
            ;;
        esac
      fi
      
      if (cd "$crate_path" && CARGO_INCREMENTAL=1 cargo test --lib --test-threads=1 $features --quiet 2>&1 >> "$output_file"); then
        echo -e "${GREEN}└─ ✅ PASSED${NC}" >> "$output_file"
        echo "PASS" > "$result_file"
      else
        echo -e "${RED}└─ ❌ FAILED${NC}" >> "$output_file"
        echo "FAIL" > "$result_file"
      fi
    fi
    echo >> "$output_file"
  ) &
}

# Run tests for all crates concurrently
echo -e "${BLUE}Starting concurrent test execution...${NC}"
TOTAL_CRATES=${#CRATES[@]}
for crate in "${CRATES[@]}"; do
  test_crate "$crate"
done

# Wait for all background jobs to complete
wait

# Collect results and display output
for crate in "${CRATES[@]}"; do
  crate_name=$(basename "$crate")
  result_file="$TMPDIR/${crate_name}.result"
  output_file="$TMPDIR/${crate_name}.out"

  if [ -f "$output_file" ]; then
    cat "$output_file"
  fi

  if [ -f "$result_file" ]; then
    result=$(cat "$result_file")
    case "$result" in
      "PASS")
        PASSED_CRATES=$((PASSED_CRATES + 1))
        ;;
      "FAIL")
        FAILED_CRATES+=("$crate_name")
        ;;
      "SKIP")
        # Already counted in TOTAL_CRATES, but not in passed/failed
        ;;
    esac
  fi
done

# Summary
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}📊 Test Summary${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "Total crates tested: ${TOTAL_CRATES}"
echo -e "${GREEN}Passed: ${PASSED_CRATES}${NC}"
echo -e "${RED}Failed: ${#FAILED_CRATES[@]}${NC}"

if [ ${#FAILED_CRATES[@]} -gt 0 ]; then
  echo
  echo -e "${RED}Failed crates:${NC}"
  for crate in "${FAILED_CRATES[@]}"; do
    echo -e "  ${RED}• ${crate}${NC}"
  done
  echo
  exit 1
else
  echo
  echo -e "${GREEN}✅ All tests passed!${NC}"
  echo
  exit 0
fi
