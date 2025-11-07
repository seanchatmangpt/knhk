#!/bin/bash
# Poka-yoke: Prevent commits with known issues
# This hook runs Gate 0 validation before allowing any commit

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔒 Pre-commit: Running Gate 0 validation..."

# Run Gate 0 before allowing commit
if ! "$PROJECT_ROOT/scripts/gate-0-validation.sh"; then
  echo ""
  echo "❌ Commit blocked by Gate 0"
  echo "Fix issues above before committing"
  echo ""
  echo "To skip this check (NOT RECOMMENDED):"
  echo "  git commit --no-verify"
  exit 1
fi

echo "✅ Pre-commit validation passed"
exit 0
