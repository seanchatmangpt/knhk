#!/bin/bash
# Auto-fix common issues (reduce manual rework)
# DFLSS Poka-Yoke: Automated error prevention

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🔧 Auto-fixing common issues..."
echo "================================"
echo ""

START_TIME=$(date +%s)

# Auto-fix #1: Format code
echo "→ [1/4] Formatting code with rustfmt..."
cargo fmt --all
echo "  ✅ Code formatted"

# Auto-fix #2: Fix clippy suggestions (if safe)
echo "→ [2/4] Applying safe clippy fixes..."
if cargo clippy --fix --allow-dirty --workspace --quiet 2>&1; then
  echo "  ✅ Clippy fixes applied"
else
  echo "  ⚠️  Some clippy fixes require manual intervention"
fi

# Auto-fix #3: Remove trailing whitespace
echo "→ [3/4] Removing trailing whitespace..."
find rust/*/src -name "*.rs" -type f -exec sed -i '' 's/[[:space:]]*$//' {} \; 2>/dev/null || true
echo "  ✅ Trailing whitespace removed"

# Auto-fix #4: Update Cargo.lock (if needed)
echo "→ [4/4] Updating Cargo.lock..."
if [ -f "Cargo.lock" ]; then
  cargo update --workspace --quiet 2>&1 || true
  echo "  ✅ Cargo.lock updated"
else
  echo "  ⏭️  No Cargo.lock found"
fi

END_TIME=$(date +%s)
TOTAL_TIME=$((END_TIME - START_TIME))

echo ""
echo "✅ Auto-fixes complete (${TOTAL_TIME}s)"
echo ""
echo "Next steps:"
echo "  1. Run 'git diff' to review changes"
echo "  2. Run './scripts/gate-0-validation.sh' to verify"
echo "  3. Commit with 'git commit -m \"your message\"'"
echo ""
