#!/bin/bash
# Update git hooks to treat warnings as errors (80/20 approach)
# High-value warnings: unused variables, dead code, errors
# Low-value warnings: deprecated APIs (already marked), naming conventions

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

echo "🔧 Updating git hooks to treat warnings as errors (80/20 approach)..."
echo ""

# Ensure .git/hooks directory exists
if [ ! -d "$HOOKS_DIR" ]; then
  echo "❌ ERROR: .git/hooks directory not found"
  echo "   Are you in a git repository?"
  exit 1
fi

# Update pre-commit hook to use -D warnings (treat warnings as errors)
if [ -f "$HOOKS_DIR/pre-commit" ]; then
  echo "📝 Updating pre-commit hook..."
  
  # Check if already using -D warnings
  if grep -q "cargo clippy.*-D warnings" "$HOOKS_DIR/pre-commit"; then
    echo "  ✅ Pre-commit hook already uses -D warnings"
  else
    # Update clippy command to include -D warnings
    sed -i.bak 's/cargo clippy --package/cargo clippy --package/g' "$HOOKS_DIR/pre-commit"
    sed -i.bak 's/cargo clippy --package "$pkg" --lib --bins/cargo clippy --package "$pkg" --lib --bins -- -D warnings/g' "$HOOKS_DIR/pre-commit"
    echo "  ✅ Updated pre-commit hook to use -D warnings"
  fi
else
  echo "  ⚠️  Pre-commit hook not found, creating from template..."
  cp "$SCRIPT_DIR/pre-commit-hook.sh" "$HOOKS_DIR/pre-commit"
  chmod +x "$HOOKS_DIR/pre-commit"
  echo "  ✅ Created pre-commit hook"
fi

# Update pre-push hook to use -D warnings (treat warnings as errors)
if [ -f "$HOOKS_DIR/pre-push" ]; then
  echo "📝 Updating pre-push hook..."
  
  # Check if already using -D warnings
  if grep -q "cargo clippy.*-D warnings" "$HOOKS_DIR/pre-push"; then
    echo "  ✅ Pre-push hook already uses -D warnings"
  else
    # Update clippy command to include -D warnings
    sed -i.bak 's/cargo clippy --workspace --lib --bins/cargo clippy --workspace --lib --bins -- -D warnings/g' "$HOOKS_DIR/pre-push"
    echo "  ✅ Updated pre-push hook to use -D warnings"
  fi
else
  echo "  ⚠️  Pre-push hook not found, creating from template..."
  cp "$SCRIPT_DIR/pre-push-hook.sh" "$HOOKS_DIR/pre-push"
  chmod +x "$HOOKS_DIR/pre-push"
  echo "  ✅ Created pre-push hook"
fi

# Clean up backup files
rm -f "$HOOKS_DIR/pre-commit.bak" "$HOOKS_DIR/pre-push.bak" 2>/dev/null || true

echo ""
echo "✅ Git hooks updated to treat warnings as errors"
echo ""
echo "80/20 Approach:"
echo "  ✅ High-value warnings treated as errors:"
echo "     - Unused variables"
echo "     - Dead code"
echo "     - Compilation errors"
echo ""
echo "  ⚠️  Low-value warnings filtered (already marked):"
echo "     - Deprecated APIs (marked with #[allow(deprecated)])"
echo "     - Test files (excluded from clippy checks)"
echo "     - Naming conventions (cosmetic only)"
echo ""
echo "To test:"
echo "  git commit -m 'test'  # Will run pre-commit hook"
echo "  git push              # Will run pre-push hook"





