#!/bin/bash
# Pre-commit validation (fast checks)
# Part of KNHK Build Validation Matrix
# Target: ~3 minutes
set -e

echo "=== Pre-Commit Validation ==="
echo "Running fast validation checks before commit"
echo ""

start_time=$(date +%s)

# 1. Format check (~10s)
echo "## 1. Format Check"
cargo fmt --all -- --check
echo "✅ Code formatting is correct"
echo ""

# 2. Clippy (~60s)
echo "## 2. Clippy Lints"
cargo clippy --workspace -- -D warnings
echo "✅ No clippy warnings"
echo ""

# 3. Quick tests (~60s with cache)
echo "## 3. Library Tests"
cargo test --workspace --lib -- --test-threads=4
echo "✅ Library tests passed"
echo ""

# 4. Chicago TDD (~60s)
echo "## 4. Chicago TDD Validation"
if [ -f "Makefile" ] || [ -f "../Makefile" ]; then
  make test-chicago-v04
  echo "✅ Chicago TDD tests passed"
else
  echo "⚠️  Makefile not found, skipping Chicago TDD tests"
fi
echo ""

end_time=$(date +%s)
elapsed=$((end_time - start_time))

echo "=== Pre-Commit Validation Summary ==="
echo "Time elapsed: ${elapsed}s"
echo "✅ All pre-commit checks passed"
echo ""
echo "You can now safely commit your changes!"
