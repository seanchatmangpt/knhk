#!/bin/bash

# Git Hooks Setup Script for Chicago TDD Safety Enforcement
# Prevents unsafe patterns from being committed to production code

set -e

HOOKS_DIR=".git/hooks"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$(dirname "${SCRIPT_DIR}")" && pwd)"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🔒 Setting up Git hooks for safety enforcement...${NC}"

# Ensure hooks directory exists
mkdir -p "${HOOKS_DIR}"

# ============================================================================
# PRE-COMMIT HOOK: Check for unsafe patterns
# ============================================================================

cat > "${HOOKS_DIR}/pre-commit" << 'HOOK_SCRIPT'
#!/bin/bash

# Pre-commit hook: Check for unsafe patterns in Rust code
# Prevents unwrap(), expect(), panic!() in production code

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

ERRORS=0
WARNINGS=0

echo -e "${YELLOW}🔍 Running pre-commit safety checks...${NC}"

# Check for unwrap() in production code (not tests)
echo "  Checking for unsafe unwrap() calls..."
if git diff --cached --name-only | grep -E '\.rs$' | xargs -I {} git show :"{}" | \
   grep -n "\.unwrap()" > /tmp/unwrap_check.txt 2>/dev/null; then
    if ! grep -q "// SAFETY:" /tmp/unwrap_check.txt && \
       ! grep -q "#\[cfg(test)\]" /tmp/unwrap_check.txt; then
        echo -e "    ${YELLOW}⚠️  Found potential unwrap() usage:${NC}"
        cat /tmp/unwrap_check.txt | head -5
        ((WARNINGS++))
    fi
fi

# Check for expect() in production code
echo "  Checking for unsafe expect() calls..."
if git diff --cached --name-only | grep -E '\.rs$' | xargs -I {} git show :"{}" | \
   grep -n "\.expect(" > /tmp/expect_check.txt 2>/dev/null; then
    if ! grep -q "// SAFETY:" /tmp/expect_check.txt && \
       ! grep -q "#\[cfg(test)\]" /tmp/expect_check.txt; then
        echo -e "    ${YELLOW}⚠️  Found potential expect() usage:${NC}"
        cat /tmp/expect_check.txt | head -5
        ((WARNINGS++))
    fi
fi

# Check for panic!() in production code
echo "  Checking for explicit panic!() calls..."
if git diff --cached --name-only | grep -E '\.rs$' | xargs -I {} git show :"{}" | \
   grep -n "panic!(" > /tmp/panic_check.txt 2>/dev/null; then
    if ! grep -q "// SAFETY:" /tmp/panic_check.txt && \
       ! grep -q "#\[cfg(test)\]" /tmp/panic_check.txt; then
        echo -e "    ${RED}❌ Found explicit panic!() calls:${NC}"
        cat /tmp/panic_check.txt | head -5
        ((ERRORS++))
    fi
fi

# Run rustfmt check
echo "  Checking code formatting..."
if ! cargo fmt --all -- --check > /dev/null 2>&1; then
    echo -e "    ${YELLOW}⚠️  Code needs formatting (run: cargo fmt --all)${NC}"
    ((WARNINGS++))
fi

# Run clippy check (warnings only, not errors)
echo "  Checking with Clippy..."
if ! cargo clippy --lib -- -D warnings > /dev/null 2>&1; then
    echo -e "    ${YELLOW}⚠️  Clippy found warnings${NC}"
    ((WARNINGS++))
fi

# Summary
echo ""
if [ $ERRORS -gt 0 ]; then
    echo -e "${RED}❌ Pre-commit check FAILED ($ERRORS errors, $WARNINGS warnings)${NC}"
    echo -e "${RED}   Cannot commit with critical safety issues${NC}"
    exit 1
elif [ $WARNINGS -gt 0 ]; then
    echo -e "${YELLOW}⚠️  Pre-commit check passed with $WARNINGS warnings${NC}"
    echo -e "${YELLOW}   Consider fixing these before committing${NC}"
    read -p "Continue with commit? (y/n) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
else
    echo -e "${GREEN}✅ Pre-commit check passed${NC}"
fi

exit 0
HOOK_SCRIPT

chmod +x "${HOOKS_DIR}/pre-commit"

# ============================================================================
# COMMIT-MSG HOOK: Enforce commit message format
# ============================================================================

cat > "${HOOKS_DIR}/commit-msg" << 'HOOK_SCRIPT'
#!/bin/bash

# Commit-msg hook: Enforce conventional commit format
# Format: type(scope): description
# Types: feat, fix, docs, test, refactor, perf, style, chore, revert

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

COMMIT_MSG_FILE=$1
COMMIT_MSG=$(cat "$COMMIT_MSG_FILE")

# Skip validation for merge commits
if git rev-parse MERGE_HEAD > /dev/null 2>&1; then
    exit 0
fi

# Check conventional commit format
if ! echo "$COMMIT_MSG" | grep -qE '^(feat|fix|docs|test|refactor|perf|style|chore|revert)(\(.+\))?:'; then
    echo -e "${YELLOW}⚠️  Commit message doesn't follow conventional format${NC}"
    echo "Expected format: type(scope): description"
    echo "Valid types: feat, fix, docs, test, refactor, perf, style, chore, revert"
    echo ""
    echo "Example: feat(knhk-cli): add Chicago TDD integration"
    # Don't enforce strictly (just warn)
fi

exit 0
HOOK_SCRIPT

chmod +x "${HOOKS_DIR}/commit-msg"

# ============================================================================
# POST-MERGE HOOK: Suggest running tests
# ============================================================================

cat > "${HOOKS_DIR}/post-merge" << 'HOOK_SCRIPT'
#!/bin/bash

# Post-merge hook: Remind to run tests after merging

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo ""
echo -e "${YELLOW}📝 Merged changes - remember to:${NC}"
echo "  1. Run: ${GREEN}cargo make test${NC} (unit tests)"
echo "  2. Run: ${GREEN}cargo make build${NC} (ensure build succeeds)"
echo "  3. Run: ${GREEN}cargo make pre-commit${NC} (full validation)"
echo ""

exit 0
HOOK_SCRIPT

chmod +x "${HOOKS_DIR}/post-merge"

# ============================================================================
# PUSH HOOK: Suggest running full test suite
# ============================================================================

cat > "${HOOKS_DIR}/pre-push" << 'HOOK_SCRIPT'
#!/bin/bash

# Pre-push hook: Run tests before pushing

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${YELLOW}🚀 Running tests before push...${NC}"

# Run pre-commit validation
if ! cargo make pre-commit > /dev/null 2>&1; then
    echo -e "${RED}❌ Pre-commit validation failed${NC}"
    echo "Fix issues before pushing"
    exit 1
fi

echo -e "${GREEN}✅ All tests passed${NC}"
exit 0
HOOK_SCRIPT

chmod +x "${HOOKS_DIR}/pre-push"

# ============================================================================
# Success Message
# ============================================================================

echo ""
echo -e "${GREEN}✅ Git hooks installed successfully${NC}"
echo ""
echo "Installed hooks:"
echo "  • pre-commit:   Check for unsafe patterns, formatting, and clippy"
echo "  • commit-msg:   Suggest conventional commit format"
echo "  • post-merge:   Remind to run tests"
echo "  • pre-push:     Run validation before pushing"
echo ""
echo "To bypass hooks temporarily:"
echo "  ${YELLOW}git commit --no-verify${NC} (NOT RECOMMENDED)"
echo ""
echo "To remove hooks:"
echo "  ${YELLOW}rm -rf .git/hooks${NC}"
echo ""
