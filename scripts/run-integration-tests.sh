#!/usr/bin/env bash
# KNHK Integration Test Runner
# Executes integration tests (C + Rust)

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🔗 KNHK Integration Tests${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# C Integration Tests (v2)
echo -e "${BLUE}Running C integration tests (v2)...${NC}"
cd c
if make test-integration-v2; then
  echo -e "${GREEN}✅ C integration tests passed${NC}"
else
  echo -e "${RED}❌ C integration tests failed${NC}"
  exit 1
fi
echo

# Rust Integration Tests
cd "$PROJECT_ROOT"
echo -e "${BLUE}Running Rust integration tests...${NC}"

if [ -d "rust/knhk-integration-tests" ]; then
  cd rust/knhk-integration-tests
  if cargo test --quiet 2>&1; then
    echo -e "${GREEN}✅ Rust integration tests passed${NC}"
  else
    echo -e "${RED}❌ Rust integration tests failed${NC}"
    exit 1
  fi
else
  echo -e "${BLUE}ℹ️  No dedicated Rust integration test crate${NC}"
fi

echo
echo -e "${GREEN}✅ Integration validation complete${NC}"
exit 0
