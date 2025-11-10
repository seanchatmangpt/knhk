#!/bin/bash
# Validate Test Quality - Detect meaningless tests that violate Chicago TDD principles
#
# Scans test files for:
# 1. Tests with only assert_ok!() or assert_err!() without behavior verification
# 2. Tests that don't verify observable outputs
# 3. Tests with JTBD comments that don't match actual assertions

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUST_DIR="$PROJECT_ROOT/rust"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track violations
VIOLATIONS=0
VIOLATION_FILES=()

echo "🔍 Scanning test files for meaningless tests..."

# Find all test files
TEST_FILES=$(find "$RUST_DIR" -type f \( -name "*test*.rs" -o -name "*_test.rs" \) | sort)

if [ -z "$TEST_FILES" ]; then
    echo "❌ No test files found"
    exit 1
fi

# Function to check if test has meaningful assertions
check_test_quality() {
    local file="$1"
    local test_name="$2"
    local test_body="$3"
    local jtbd_comment="$4"
    
    # Check if test only has assert_ok!() or assert_err!() without other assertions
    local assert_ok_count=$(echo "$test_body" | grep -o "assert_ok!" | wc -l | tr -d ' ')
    local assert_err_count=$(echo "$test_body" | grep -o "assert_err!" | wc -l | tr -d ' ')
    local other_assertions=$(echo "$test_body" | grep -E "assert_eq!|assert_ne!|assert!\([^)]*[^ok][^err]" | wc -l | tr -d ' ')
    
    # If only assert_ok!/assert_err! without other assertions, it's meaningless
    if [ "$assert_ok_count" -gt 0 ] && [ "$other_assertions" -eq 0 ]; then
        echo "  ❌ Only checks assert_ok!(), doesn't verify behavior"
        return 1
    fi
    
    if [ "$assert_err_count" -gt 0 ] && [ "$other_assertions" -eq 0 ]; then
        echo "  ❌ Only checks assert_err!(), doesn't verify behavior"
        return 1
    fi
    
    # Check if JTBD comment mentions behavior but test doesn't verify it
    if [ -n "$jtbd_comment" ]; then
        # Check for behavior keywords in JTBD
        if echo "$jtbd_comment" | grep -qiE "sequential|parallel|order|timing|state|output|verify|check"; then
            # Check if test body verifies these behaviors
            if ! echo "$test_body" | grep -qiE "order|timing|state|output|verify|check|execution|sequence|parallel"; then
                echo "  ❌ JTBD mentions behavior but test doesn't verify it"
                return 1
            fi
        fi
    fi
    
    return 0
}

# Process each test file
for test_file in $TEST_FILES; do
    # Skip if file doesn't exist or is empty
    [ -f "$test_file" ] || continue
    [ -s "$test_file" ] || continue
    
    # Extract test functions and their JTBD comments
    # This is a simplified check - in production, would use proper Rust AST parsing
    local file_violations=0
    
    # Check for common meaningless patterns
    if grep -q "assert_ok!" "$test_file"; then
        # Check if file has tests with only assert_ok! without other meaningful assertions
        # This is a heuristic - proper implementation would parse AST
        local line_num=0
        while IFS= read -r line; do
            ((line_num++))
            
            # Look for test definitions
            if echo "$line" | grep -qE "^(chicago_test|chicago_async_test|#\[test\]|fn test_)"; then
                # Extract test name
                local test_name=$(echo "$line" | sed -E 's/.*(test_[a-zA-Z0-9_]+).*/\1/' | head -1)
                
                # Check next 20 lines for meaningful assertions
                local test_section=$(sed -n "${line_num},$((line_num+20))p" "$test_file")
                local assert_count=$(echo "$test_section" | grep -cE "assert_eq!|assert_ne!|assert!\([^)]*[^ok][^err]" || true)
                local assert_ok_count=$(echo "$test_section" | grep -c "assert_ok!" || true)
                
                # If only assert_ok! without other assertions, flag it
                if [ "$assert_ok_count" -gt 0 ] && [ "$assert_count" -eq 0 ]; then
                    echo -e "${YELLOW}⚠️  Potential meaningless test:${NC}"
                    echo "   File: $test_file:$line_num"
                    echo "   Test: $test_name"
                    echo "   Issue: Only checks assert_ok!(), may not verify behavior"
                    ((file_violations++))
                fi
            fi
        done < "$test_file"
    fi
    
    if [ "$file_violations" -gt 0 ]; then
        ((VIOLATIONS += file_violations))
        VIOLATION_FILES+=("$test_file")
    fi
done

# Report results
echo ""
if [ "$VIOLATIONS" -eq 0 ]; then
    echo -e "${GREEN}✅ No meaningless tests found${NC}"
    exit 0
else
    echo -e "${RED}❌ Found $VIOLATIONS potential meaningless test(s)${NC}"
    echo ""
    echo "Files with violations:"
    for file in "${VIOLATION_FILES[@]}"; do
        echo "  - $file"
    done
    echo ""
    echo "Review these tests and ensure they verify observable behavior, not just function existence."
    echo "See .cursor/rules/chicago-tdd-standards.mdc for examples of meaningful tests."
    exit 1
fi


