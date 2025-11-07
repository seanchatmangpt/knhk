#!/bin/bash
# monitor_unwrap_remediation.sh
# TDD London School: Continuous regression monitoring during unwrap() fixes
#
# Usage: ./monitor_unwrap_remediation.sh [watch]
#   - Default: Single validation pass
#   - watch: Continuous monitoring every 30 seconds

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_FILE="$CRATE_DIR/remediation_log.txt"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +%T)]${NC} $1" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}[$(date +%T)] ERROR:${NC} $1" | tee -a "$LOG_FILE"
}

warn() {
    echo -e "${YELLOW}[$(date +%T)] WARN:${NC} $1" | tee -a "$LOG_FILE"
}

# Initialize log
echo "=== Unwrap() Remediation Monitor ===" > "$LOG_FILE"
echo "Started: $(date)" >> "$LOG_FILE"
echo "" >> "$LOG_FILE"

validate() {
    log "Starting validation pass..."

    # 1. Count unwrap() calls
    log "Counting unwrap() usage..."
    UNWRAP_COUNT=$(grep -r "\.unwrap()" "$CRATE_DIR/src" | wc -l | tr -d ' ')
    log "Found $UNWRAP_COUNT unwrap() calls"

    # 2. Count expect() calls
    log "Counting expect() usage..."
    EXPECT_COUNT=$(grep -r "\.expect(" "$CRATE_DIR/src" | wc -l | tr -d ' ')
    log "Found $EXPECT_COUNT expect() calls"

    # 3. Run clippy
    log "Running clippy..."
    cd "$CRATE_DIR"
    if cargo clippy -- -D warnings 2>&1 | tee -a "$LOG_FILE"; then
        log "${GREEN}✓ Clippy passed with zero warnings${NC}"
        CLIPPY_STATUS="PASS"
    else
        error "✗ Clippy found warnings/errors"
        CLIPPY_STATUS="FAIL"
    fi

    # 4. Run tests
    log "Running tests..."
    if cargo test 2>&1 | tee -a "$LOG_FILE" | tail -20; then
        log "${GREEN}✓ Tests passed${NC}"
        TEST_STATUS="PASS"
    else
        error "✗ Tests failed"
        TEST_STATUS="FAIL"
    fi

    # 5. Check Chicago TDD tests
    log "Checking Chicago TDD tests..."
    CHICAGO_TESTS=$(find "$CRATE_DIR/tests" -name "chicago_tdd_*.rs" | wc -l | tr -d ' ')
    log "Found $CHICAGO_TESTS Chicago TDD test files"

    # 6. Generate summary
    echo "" | tee -a "$LOG_FILE"
    echo "=== Validation Summary ===" | tee -a "$LOG_FILE"
    echo "Timestamp: $(date)" | tee -a "$LOG_FILE"
    echo "unwrap() count: $UNWRAP_COUNT" | tee -a "$LOG_FILE"
    echo "expect() count: $EXPECT_COUNT" | tee -a "$LOG_FILE"
    echo "Clippy status: $CLIPPY_STATUS" | tee -a "$LOG_FILE"
    echo "Test status: $TEST_STATUS" | tee -a "$LOG_FILE"
    echo "Chicago tests: $CHICAGO_TESTS" | tee -a "$LOG_FILE"
    echo "" | tee -a "$LOG_FILE"

    # 7. Store metrics in memory
    log "Storing metrics in Claude Flow memory..."
    npx claude-flow@alpha hooks notify --message "Remediation validation: unwrap=$UNWRAP_COUNT, clippy=$CLIPPY_STATUS, tests=$TEST_STATUS"

    # 8. Check regression criteria
    if [ "$CLIPPY_STATUS" = "FAIL" ]; then
        error "REGRESSION DETECTED: Clippy failures introduced"
        return 1
    fi

    if [ "$TEST_STATUS" = "FAIL" ]; then
        error "REGRESSION DETECTED: Test failures introduced"
        return 1
    fi

    log "${GREEN}✓ No regressions detected${NC}"
    return 0
}

# Main execution
if [ "$1" = "watch" ]; then
    log "Starting continuous monitoring (Ctrl+C to stop)..."
    while true; do
        validate
        log "Waiting 30 seconds before next check..."
        sleep 30
    done
else
    validate
fi
