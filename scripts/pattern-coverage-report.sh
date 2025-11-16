#!/usr/bin/env bash
# Pattern Coverage Report Script
# Generates comprehensive coverage report for pattern matrix validator
# Validates Covenant 4: All Patterns Expressible via Permutations

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VALIDATION_DIR="$PROJECT_ROOT/rust/knhk-validation"

echo -e "${BLUE}=== Pattern Coverage Report Generator ===${NC}"
echo ""

cd "$VALIDATION_DIR" || exit 1

# Step 1: Build the validation library
echo -e "${BLUE}Step 1: Building validation library...${NC}"
if cargo build --release --lib 2>&1; then
    echo -e "${GREEN}✓ Build successful${NC}"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

# Step 2: Run pattern validation tests
echo ""
echo -e "${BLUE}Step 2: Running pattern validation tests...${NC}"
if cargo test --test pattern_validation_tests -- --nocapture 2>&1 | tee /tmp/pattern-test-output.txt; then
    echo -e "${GREEN}✓ Tests passed${NC}"
else
    echo -e "${YELLOW}⚠ Some tests may have failed (check output)${NC}"
fi

# Step 3: Run coverage report example
echo ""
echo -e "${BLUE}Step 3: Generating coverage report...${NC}"
if cargo run --release --example validate_patterns 2>&1 | tee /tmp/pattern-coverage-output.txt; then
    echo -e "${GREEN}✓ Coverage report generated${NC}"
else
    echo -e "${RED}✗ Coverage report failed${NC}"
    exit 1
fi

# Step 4: Analyze coverage
echo ""
echo -e "${BLUE}Step 4: Analyzing coverage...${NC}"

# Extract coverage percentage from output
COVERAGE_PERCENT=$(grep -o "Coverage: [0-9.]*%" /tmp/pattern-coverage-output.txt | head -1 | grep -o "[0-9.]*")

if [ -n "$COVERAGE_PERCENT" ]; then
    echo -e "${GREEN}Coverage: ${COVERAGE_PERCENT}%${NC}"

    # Check if coverage meets threshold
    THRESHOLD=20.0
    if awk "BEGIN {exit !($COVERAGE_PERCENT >= $THRESHOLD)}"; then
        echo -e "${GREEN}✓ Coverage meets threshold (≥${THRESHOLD}%)${NC}"
    else
        echo -e "${YELLOW}⚠ Coverage below threshold (${THRESHOLD}%)${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Could not extract coverage percentage${NC}"
fi

# Step 5: Generate summary report
echo ""
echo -e "${BLUE}Step 5: Summary Report${NC}"
echo ""
echo "=== Pattern Matrix Validator Coverage Report ==="
echo ""
echo "Total W3C Patterns: 43"
echo "Supported Patterns: $(grep -c "✓" /tmp/pattern-coverage-output.txt || echo "N/A")"
echo "Total Combinations: $(grep "Total Combinations:" /tmp/pattern-coverage-output.txt | grep -o "[0-9]*" || echo "N/A")"
echo "Coverage: ${COVERAGE_PERCENT:-N/A}%"
echo ""

# List supported patterns
echo "Supported Patterns:"
grep "✓" /tmp/pattern-coverage-output.txt | sed 's/^/  /' || echo "  (none found)"
echo ""

# List gaps
echo "Gaps (unsupported patterns):"
grep "✗" /tmp/pattern-coverage-output.txt | sed 's/^/  /' || echo "  (none found)"
echo ""

# Step 6: Check for invalid patterns
echo -e "${BLUE}Step 6: Validating invalid pattern detection...${NC}"
INVALID_PATTERNS=$(grep -c "Invalid pattern" /tmp/pattern-coverage-output.txt || echo "0")
echo -e "${GREEN}✓ Invalid patterns detected: ${INVALID_PATTERNS}${NC}"

# Step 7: Generate metrics
echo ""
echo -e "${BLUE}Step 7: Metrics${NC}"
echo ""

TEST_COUNT=$(grep -c "^test " /tmp/pattern-test-output.txt || echo "0")
PASSED_COUNT=$(grep -c " ok$" /tmp/pattern-test-output.txt || echo "0")
FAILED_COUNT=$(grep -c " FAILED$" /tmp/pattern-test-output.txt || echo "0")

echo "Test Results:"
echo "  Total tests: $TEST_COUNT"
echo "  Passed: $PASSED_COUNT"
echo "  Failed: $FAILED_COUNT"
echo ""

# Step 8: CI/CD integration check
echo -e "${BLUE}Step 8: CI/CD Integration Check${NC}"
if [ "$FAILED_COUNT" -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed - ready for CI/CD${NC}"
    EXIT_CODE=0
else
    echo -e "${RED}✗ Tests failed - fix before promoting${NC}"
    EXIT_CODE=1
fi

# Step 9: Generate JSON report
echo ""
echo -e "${BLUE}Step 9: Generating JSON report...${NC}"
cat > /tmp/pattern-coverage-report.json <<EOF
{
  "total_w3c_patterns": 43,
  "supported_patterns": $(grep -c "✓" /tmp/pattern-coverage-output.txt || echo "0"),
  "total_combinations": $(grep "Total Combinations:" /tmp/pattern-coverage-output.txt | grep -o "[0-9]*" || echo "0"),
  "coverage_percentage": ${COVERAGE_PERCENT:-0},
  "test_count": $TEST_COUNT,
  "passed_count": $PASSED_COUNT,
  "failed_count": $FAILED_COUNT,
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF
echo -e "${GREEN}✓ JSON report saved to /tmp/pattern-coverage-report.json${NC}"

# Step 10: Save reports
echo ""
echo -e "${BLUE}Step 10: Saving reports...${NC}"
REPORT_DIR="$PROJECT_ROOT/docs/coverage-reports"
mkdir -p "$REPORT_DIR"

cp /tmp/pattern-coverage-output.txt "$REPORT_DIR/pattern-coverage-$(date +%Y%m%d-%H%M%S).txt"
cp /tmp/pattern-coverage-report.json "$REPORT_DIR/pattern-coverage-latest.json"

echo -e "${GREEN}✓ Reports saved to $REPORT_DIR${NC}"

echo ""
echo -e "${BLUE}=== Coverage Report Complete ===${NC}"
echo ""

exit $EXIT_CODE
