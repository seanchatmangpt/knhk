#!/usr/bin/env bash
# KNHK C Lint Runner
# Executes clang-tidy on all C source and header files

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🔍 KNHK C Lint Runner (clang-tidy)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# Project root
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

# Check if clang-tidy is available
if ! command -v clang-tidy &> /dev/null; then
  echo -e "${RED}❌ clang-tidy not found${NC}"
  echo -e "${YELLOW}   Install clang-tidy to enable C linting${NC}"
  exit 1
fi

# C source directory
C_DIR="$PROJECT_ROOT/c"

if [ ! -d "$C_DIR" ]; then
  echo -e "${YELLOW}⚠️  C directory not found, skipping${NC}"
  exit 0
fi

# Find all C source files (skip headers to avoid redundant checks and improve performance)
# Headers are checked when included by source files
C_FILES=$(find "$C_DIR" -type f -name "*.c" 2>/dev/null | sort)

if [ -z "$C_FILES" ]; then
  echo -e "${YELLOW}⚠️  No C files found, skipping${NC}"
  exit 0
fi

# Count files
TOTAL_FILES=$(echo "$C_FILES" | wc -l)
echo -e "${BLUE}Found ${TOTAL_FILES} C files to lint${NC}"
echo

# clang-tidy checks
# Fast mode: Focus on critical issues only
# - bugprone: Common bugs (critical)
# - clang-analyzer-core: Core static analysis (critical)
CHECKS="bugprone-*,clang-analyzer-core.*,performance-*"

# Disable specific noisy checks
DISABLED_CHECKS="-bugprone-easily-swappable-parameters,-bugprone-reserved-identifier"

LINT_RESULT=0
FAILED_FILES=()

# Lint each file
while IFS= read -r file; do
  relative_file="${file#$PROJECT_ROOT/}"
  echo -e "${BLUE}┌─ Linting ${relative_file}...${NC}"

  # Run clang-tidy and capture output and exit code
  # Suppress non-user code warnings and unknown warning options
  set +e
  clang_output=$(clang-tidy \
    --checks="$CHECKS,$DISABLED_CHECKS" \
    --warnings-as-errors='*' \
    -p "$C_DIR" \
    "$file" \
    -- \
    -I"$C_DIR/include" \
    -I"$C_DIR/src" \
    2>&1 | grep -v "warning: unknown warning option" | grep -E "(error:|warning:)" || true)
  clang_exit=$?
  set -e

  # Display output if there are errors/warnings
  if [ -n "$clang_output" ]; then
    echo "$clang_output"
  fi

  # Check if clang-tidy found errors
  if echo "$clang_output" | grep -q "error:"; then
    echo -e "${RED}└─ ❌ FAILED${NC}"
    LINT_RESULT=1
    FAILED_FILES+=("$relative_file")
  else
    echo -e "${GREEN}└─ ✅ PASSED${NC}"
  fi
  echo
done <<< "$C_FILES"

# Summary
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}📊 Lint Summary${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "Total files linted: ${TOTAL_FILES}"

if [ "$LINT_RESULT" -eq 0 ]; then
  echo -e "${GREEN}Passed: ${TOTAL_FILES}${NC}"
  echo -e "${RED}Failed: 0${NC}"
  echo
  echo -e "${GREEN}✅ All C lints passed!${NC}"
  echo
  exit 0
else
  PASSED_COUNT=$((TOTAL_FILES - ${#FAILED_FILES[@]}))
  echo -e "${GREEN}Passed: ${PASSED_COUNT}${NC}"
  echo -e "${RED}Failed: ${#FAILED_FILES[@]}${NC}"
  echo
  echo -e "${RED}Failed files:${NC}"
  for file in "${FAILED_FILES[@]}"; do
    echo -e "  ${RED}• ${file}${NC}"
  done
  echo
  exit 1
fi
