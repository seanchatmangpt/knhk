#!/usr/bin/env bash
# KNHK Rust Lint Runner
# Executes cargo clippy on all Rust crates concurrently

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Temporary directory for lint results
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🔍 KNHK Rust Lint Runner (Concurrent)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# Project root
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

# All Rust crates
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
  "rust/knhk-patterns"
  "rust/knhk-workflow-engine"
  "rust/knhk-json-bench"
  "rust/chicago-tdd-tools"
  "rust/knhk-dflss"
)

# Lint a single crate (runs in background)
lint_crate() {
  local crate_path="$1"
  local crate_name=$(basename "$crate_path")
  local result_file="$TMPDIR/${crate_name}.result"
  local output_file="$TMPDIR/${crate_name}.out"

  (
    echo -e "${BLUE}┌─ Linting ${crate_name}...${NC}" > "$output_file"

    if [ ! -d "$crate_path" ]; then
      echo -e "${YELLOW}└─ ⚠️  Crate not found, skipping${NC}" >> "$output_file"
      echo "SKIP" > "$result_file"
      return
    fi

    if [ ! -f "$crate_path/Cargo.toml" ]; then
      echo -e "${YELLOW}└─ ⚠️  No Cargo.toml, skipping${NC}" >> "$output_file"
      echo "SKIP" > "$result_file"
      return
    fi

    # Run clippy
    if (cd "$crate_path" && cargo clippy --all-targets --all-features -- -D warnings 2>&1 >> "$output_file"); then
      echo -e "${GREEN}└─ ✅ PASSED${NC}" >> "$output_file"
      echo "PASS" > "$result_file"
    else
      echo -e "${RED}└─ ❌ FAILED${NC}" >> "$output_file"
      echo "FAIL" > "$result_file"
    fi
    echo >> "$output_file"
  ) &
}

# Run lints for all crates concurrently
echo -e "${BLUE}Starting concurrent lint execution...${NC}"
TOTAL_CRATES=${#CRATES[@]}
PASSED_CRATES=0
FAILED_CRATES=()

for crate in "${CRATES[@]}"; do
  lint_crate "$crate"
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
echo -e "${BLUE}📊 Lint Summary${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "Total crates linted: ${TOTAL_CRATES}"
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
  echo -e "${GREEN}✅ All lints passed!${NC}"
  echo
  exit 0
fi

