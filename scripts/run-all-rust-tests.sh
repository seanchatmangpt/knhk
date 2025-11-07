#!/usr/bin/env bash
# KNHK Rust Test Runner
# Executes all Rust crate tests independently (no workspace)

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

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🧪 KNHK Rust Test Suite Runner${NC}"
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
  "rust/knhk-integration-tests"
)

# Test a single crate
test_crate() {
  local crate_path="$1"
  local crate_name=$(basename "$crate_path")

  TOTAL_CRATES=$((TOTAL_CRATES + 1))

  echo -e "${BLUE}┌─ Testing ${crate_name}...${NC}"

  if [ ! -d "$crate_path" ]; then
    echo -e "${YELLOW}└─ ⚠️  Crate not found, skipping${NC}"
    echo
    return
  fi

  if [ ! -f "$crate_path/Cargo.toml" ]; then
    echo -e "${YELLOW}└─ ⚠️  No Cargo.toml, skipping${NC}"
    echo
    return
  fi

  # Run tests
  if (cd "$crate_path" && cargo test --quiet 2>&1); then
    echo -e "${GREEN}└─ ✅ PASSED${NC}"
    PASSED_CRATES=$((PASSED_CRATES + 1))
  else
    echo -e "${RED}└─ ❌ FAILED${NC}"
    FAILED_CRATES+=("$crate_name")
  fi

  echo
}

# Run tests for all crates
for crate in "${CRATES[@]}"; do
  test_crate "$crate"
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
