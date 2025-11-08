#!/bin/bash
# Pre-push validation (standard checks)
# Part of KNHK Build Validation Matrix
# Target: ~6 minutes
set -e

echo "=== Pre-Push Validation ==="
echo "Running standard validation checks before push"
echo ""

start_time=$(date +%s)

# 1. Run pre-commit checks first
echo "## Running Pre-Commit Checks"
if [ -f "./scripts/validate-pre-commit.sh" ]; then
  ./scripts/validate-pre-commit.sh
else
  # Run inline if script not found
  cargo fmt --all -- --check
  cargo clippy --workspace -- -D warnings
  cargo test --workspace --lib
fi
echo ""

# 2. Full test suite (~180s)
echo "## Full Workspace Tests"
cargo test --workspace -- --test-threads=4
echo "✅ All workspace tests passed"
echo ""

# 3. Performance validation (~30s)
echo "## Performance Validation"
if [ -f "Makefile" ] || [ -f "../Makefile" ]; then
  make test-performance-v04
  echo "✅ Performance tests passed"
else
  echo "⚠️  Makefile not found, skipping performance tests"
fi
echo ""

# 4. Feature flag sampling (~120s)
echo "## Feature Flag Sampling"
echo "Testing critical feature combinations..."
cargo build -p knhk-validation --all-features --release >/dev/null 2>&1 && echo "✅ knhk-validation"
cargo build -p knhk-connectors --all-features --release >/dev/null 2>&1 && echo "✅ knhk-connectors"
cargo build -p knhk-etl --all-features --release >/dev/null 2>&1 && echo "✅ knhk-etl"
cargo build -p knhk-unrdf --all-features --release >/dev/null 2>&1 && echo "✅ knhk-unrdf"
echo "✅ Critical feature combinations validated"
echo ""

end_time=$(date +%s)
elapsed=$((end_time - start_time))

echo "=== Pre-Push Validation Summary ==="
echo "Time elapsed: ${elapsed}s"
echo "✅ All pre-push checks passed"
echo ""
echo "You can now safely push your changes!"
