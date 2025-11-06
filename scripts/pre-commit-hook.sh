#!/bin/bash
# KNHK Pre-commit Hook
# This script runs before every git commit to ensure code quality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=========================================="
echo "KNHK Pre-commit Validation"
echo "==========================================${NC}"
echo ""

# Check if this is a partial commit (only some changes staged)
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(rs|c|h)$' || true)

if [ -z "$STAGED_FILES" ]; then
    echo "No Rust or C files staged for commit. Skipping validation."
    exit 0
fi

echo "Validating staged files:"
echo "$STAGED_FILES"
echo ""

# Run quick validation (faster than full test suite)
VALIDATION_FAILED=false

# 1. Cargo clippy on workspace
echo -e "${YELLOW}1. Running Cargo Clippy...${NC}"
if cargo clippy --workspace -- -D warnings 2>&1 | head -n 50; then
    echo -e "${GREEN}✅ Clippy passed${NC}"
else
    echo -e "${RED}❌ Clippy found issues${NC}"
    VALIDATION_FAILED=true
fi
echo ""

# 2. Cargo build check
echo -e "${YELLOW}2. Running Cargo Build Check...${NC}"
if cargo check --workspace 2>&1 | head -n 50; then
    echo -e "${GREEN}✅ Build check passed${NC}"
else
    echo -e "${RED}❌ Build check failed${NC}"
    VALIDATION_FAILED=true
fi
echo ""

# 3. Check for unsafe patterns
echo -e "${YELLOW}3. Checking for unsafe patterns...${NC}"
UNSAFE_FOUND=false

for file in $STAGED_FILES; do
    if [[ $file == *.rs ]] && [[ ! $file =~ test ]]; then
        if git diff --cached "$file" | grep -E "^\+.*\.unwrap\(\)"; then
            echo -e "${RED}❌ Found .unwrap() in $file${NC}"
            UNSAFE_FOUND=true
        fi
        if git diff --cached "$file" | grep -E "^\+.*\.expect\("; then
            echo -e "${RED}❌ Found .expect() in $file${NC}"
            UNSAFE_FOUND=true
        fi
        if git diff --cached "$file" | grep -E "^\+.*println!"; then
            echo -e "${YELLOW}⚠️  Found println! in $file (should use tracing)${NC}"
        fi
    fi
done

if ! $UNSAFE_FOUND; then
    echo -e "${GREEN}✅ No unsafe patterns found${NC}"
else
    VALIDATION_FAILED=true
fi
echo ""

# 4. Quick cargo test (only unit tests, skip integration)
echo -e "${YELLOW}4. Running Quick Unit Tests...${NC}"
if cargo test --workspace --lib 2>&1 | tail -n 20; then
    echo -e "${GREEN}✅ Unit tests passed${NC}"
else
    echo -e "${RED}❌ Unit tests failed${NC}"
    VALIDATION_FAILED=true
fi
echo ""

# Summary
echo "=========================================="
if $VALIDATION_FAILED; then
    echo -e "${RED}❌ PRE-COMMIT VALIDATION FAILED${NC}"
    echo ""
    echo "Fix the issues above before committing."
    echo ""
    echo "To skip this hook (NOT RECOMMENDED):"
    echo "  git commit --no-verify"
    echo ""
    echo "To run full validation:"
    echo "  ./scripts/validate-production-ready.sh"
    exit 1
else
    echo -e "${GREEN}✅ PRE-COMMIT VALIDATION PASSED${NC}"
    echo ""
    echo "Note: This is a quick validation. Run full test suite before merging:"
    echo "  ./scripts/run-all-tests.sh"
    echo "  ./scripts/validate-production-ready.sh"
    exit 0
fi
