#!/usr/bin/env bash
# Fast parallel benchmark using cargo's built-in workspace features

set -euo pipefail

WORKSPACE_ROOT="/Users/sac/knhk/rust"
cd "$WORKSPACE_ROOT"

echo "=== Quick Workspace Benchmark ==="
echo ""

# 1. Count total LOC
echo "📊 Counting lines of code..."
total_loc=$(find . -path ./target -prune -o -name "*.rs" -type f -exec cat {} + | wc -l | tr -d ' ')
echo "Total LOC: $total_loc"
echo ""

# 2. Workspace build time
echo "🔨 Building entire workspace (debug)..."
time_start=$(date +%s)
cargo build --workspace 2>&1 | tail -5
time_end=$(date +%s)
workspace_build_debug=$((time_end - time_start))
echo "Workspace build (debug): ${workspace_build_debug}s"
echo ""

# 3. Workspace test time
echo "🧪 Testing entire workspace..."
time_start=$(date +%s)
cargo test --workspace --no-fail-fast 2>&1 | tail -10
time_end=$(date +%s)
workspace_test=$((time_end - time_start))
echo "Workspace test: ${workspace_test}s"
echo ""

# 4. Clippy on workspace
echo "📎 Running clippy on workspace..."
time_start=$(date +%s)
cargo clippy --workspace -- -D warnings 2>&1 | tail -5
time_end=$(date +%s)
workspace_clippy=$((time_end - time_start))
echo "Workspace clippy: ${workspace_clippy}s"
echo ""

# 5. List crates by dependency count
echo "📦 Crate dependency analysis..."
cargo tree --workspace --depth 1 --prefix none | grep -E "^[a-z]" | sort | uniq -c | sort -rn | head -10
echo ""

# 6. Build timings report
echo "⏱️  Generating detailed build timings..."
cargo build --workspace --timings 2>&1 | grep -E "(Finished|Compiling)" | tail -10
echo ""

# Summary
echo "=== Summary ==="
echo "Total LOC: $total_loc"
echo "Workspace build (debug): ${workspace_build_debug}s ($(echo "$workspace_build_debug / 60" | bc)m)"
echo "Workspace test: ${workspace_test}s ($(echo "$workspace_test / 60" | bc)m)"
echo "Workspace clippy: ${workspace_clippy}s"
echo ""
echo "✅ Check cargo-timing.html for detailed build timing visualization"
