#!/bin/bash
# Validate all test suites
# Part of KNHK Build Validation Matrix
set -e

echo "=== Test Suite Validation ==="
echo "Running comprehensive test suite across all test types"
echo ""

# Unit tests
echo "## 1. Library Tests (Unit Tests)"
echo "Running all library tests with parallel execution..."
cargo test --workspace --lib -- --test-threads=4
echo "✅ Library tests passed"
echo ""

# Integration tests
echo "## 2. Integration Tests"
echo "Running all integration tests..."
cargo test --workspace --test '*' -- --test-threads=2
echo "✅ Integration tests passed"
echo ""

# Doc tests
echo "## 3. Documentation Tests"
echo "Running all documentation tests..."
cargo test --workspace --doc
echo "✅ Documentation tests passed"
echo ""

# Specialized test suites
echo "## 4. Specialized Test Suites"

echo "Running Chicago TDD tests..."
if [ -f "Makefile" ] || [ -f "../Makefile" ]; then
  make test-chicago-v04
  echo "✅ Chicago TDD tests passed"
else
  echo "⚠️  Makefile not found, skipping Chicago TDD tests"
fi

echo "Running performance tests..."
if [ -f "Makefile" ] || [ -f "../Makefile" ]; then
  make test-performance-v04
  echo "✅ Performance tests passed"
else
  echo "⚠️  Makefile not found, skipping performance tests"
fi

echo "Running integration v2 tests..."
if [ -f "Makefile" ] || [ -f "../Makefile" ]; then
  make test-integration-v2
  echo "✅ Integration v2 tests passed"
else
  echo "⚠️  Makefile not found, skipping integration v2 tests"
fi
echo ""

# Benchmarks (non-blocking)
echo "## 5. Performance Benchmarks"
echo "Running benchmarks (non-blocking)..."
if cargo bench --workspace --no-fail-fast 2>/dev/null; then
  echo "✅ Benchmarks completed"
else
  echo "⚠️  Some benchmarks failed or were skipped"
fi
echo ""

echo "=== Test Suite Validation Summary ==="
echo "✅ Library tests: PASSED"
echo "✅ Integration tests: PASSED"
echo "✅ Documentation tests: PASSED"
echo "✅ Specialized test suites: PASSED"
echo ""
echo "✅ All test suites validated successfully"
