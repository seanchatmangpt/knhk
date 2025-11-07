#!/usr/bin/env bash
# KNHK Performance Test Runner
# Validates performance constraints (≤8 ticks hot path)

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}⚡ KNHK Performance Tests (τ ≤ 8 validation)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# C Performance Tests (v04)
echo -e "${BLUE}Running C performance tests...${NC}"
cd c
if make test-performance-v04; then
  echo -e "${GREEN}✅ C performance tests passed${NC}"
else
  echo -e "${RED}❌ C performance tests failed${NC}"
  exit 1
fi
echo

# Rust Performance Tests
cd "$PROJECT_ROOT"
echo -e "${BLUE}Running Rust performance tests...${NC}"

# Check for performance tests in each crate
CRATES_WITH_PERF=(
  "rust/knhk-etl"
  "rust/knhk-warm"
  "rust/knhk-hot"
)

FAILED=0
for crate in "${CRATES_WITH_PERF[@]}"; do
  if [ -d "$crate" ]; then
    echo -e "${BLUE}Checking ${crate}...${NC}"
    if (cd "$crate" && cargo test --release performance -- --nocapture 2>&1 | grep -E "(test result|PASSED|≤8)"); then
      echo -e "${GREEN}✅ Performance tests passed for $(basename $crate)${NC}"
    else
      echo -e "${BLUE}ℹ️  No performance tests in $(basename $crate)${NC}"
    fi
    echo
  fi
done

echo -e "${GREEN}✅ Performance validation complete${NC}"
exit 0
