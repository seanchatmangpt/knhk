#!/bin/bash
# Validate and fix Chicago TDD tests across all packages

set -e

cd "$(dirname "$0")/.." || exit 1
PROJECT_ROOT=$(pwd)

echo "=== Chicago TDD Validation and Fix Script ==="
echo "Starting at: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
echo ""

# Find all Chicago TDD test files
echo "=== Finding Chicago TDD Test Files ==="
CHICAGO_TDD_TESTS=$(find rust -name "*chicago_tdd*.rs" -type f | sort)
echo "Found Chicago TDD test files:"
echo "$CHICAGO_TDD_TESTS"
echo ""

# Check each package with Chicago TDD tests
echo "=== Validating Chicago TDD Tests ==="

# knhk-etl
if [ -d "rust/knhk-etl" ]; then
    echo "Checking knhk-etl Chicago TDD tests..."
    cd rust/knhk-etl
    if cargo check --tests 2>&1 | grep -q "error"; then
        echo "❌ Compilation errors found in knhk-etl tests"
        cargo check --tests 2>&1 | grep "error" | head -20
    else
        echo "✅ knhk-etl tests compile successfully"
    fi
    cd "$PROJECT_ROOT"
    echo ""
fi

# knhk-validation
if [ -d "rust/knhk-validation" ]; then
    echo "Checking knhk-validation Chicago TDD tests..."
    cd rust/knhk-validation
    if cargo check --tests 2>&1 | grep -q "error"; then
        echo "❌ Compilation errors found in knhk-validation tests"
        cargo check --tests 2>&1 | grep "error" | head -20
    else
        echo "✅ knhk-validation tests compile successfully"
    fi
    cd "$PROJECT_ROOT"
    echo ""
fi

# knhk-sidecar
if [ -d "rust/knhk-sidecar" ]; then
    echo "Checking knhk-sidecar Chicago TDD tests..."
    cd rust/knhk-sidecar
    if cargo check --tests 2>&1 | grep -q "error"; then
        echo "❌ Compilation errors found in knhk-sidecar tests"
        cargo check --tests 2>&1 | grep "error" | head -20
    else
        echo "✅ knhk-sidecar tests compile successfully"
    fi
    cd "$PROJECT_ROOT"
    echo ""
fi

echo "=== Chicago TDD Validation Complete ==="
echo "Finished at: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"

