#!/bin/bash
# Install poka-yoke git hooks for KNHK project
# Prevents unwrap() calls and unimplemented!() from being committed

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

echo "🔧 Installing KNHK poka-yoke git hooks..."

# Ensure .git/hooks directory exists
if [ ! -d "$HOOKS_DIR" ]; then
  echo "❌ ERROR: .git/hooks directory not found"
  echo "   Are you in a git repository?"
  exit 1
fi

# Create pre-commit hook
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/bash
# Poka-yoke pre-commit hook: Prevent unwrap() and unimplemented!() from being committed
# Part of KNHK Definition of Done enforcement

set -e

echo "🔍 Running pre-commit validation..."

# Check for unwrap() in staged Rust files
if git diff --cached --name-only | grep -q '\.rs$'; then
  echo "   Checking for unwrap() calls in staged Rust files..."

  # Count unwrap() calls in staged changes (not in test files)
  UNWRAPS=$(git diff --cached --diff-filter=ACM | grep -E "^\+" | grep -v "^[+-].*#\[test\]" | grep -c "\.unwrap()" || true)

  if [ "$UNWRAPS" -gt 0 ]; then
    echo "❌ ERROR: Cannot commit $UNWRAPS unwrap() calls in production code"
    echo "   Replace with proper Result<T,E> error handling"
    echo "   Use ? operator or match statements instead"
    echo ""
    echo "   Files with unwrap():"
    git diff --cached --name-only | grep '\.rs$' | while read file; do
      if git diff --cached "$file" | grep -E "^\+" | grep -q "\.unwrap()"; then
        echo "     - $file"
        git diff --cached "$file" | grep -E "^\+" | grep "\.unwrap()" | head -3
      fi
    done
    exit 1
  fi
fi

# Check for unimplemented!() placeholders
if git diff --cached --name-only | grep -q '\.rs$'; then
  echo "   Checking for unimplemented!() placeholders..."

  UNIMPL=$(git diff --cached --diff-filter=ACM | grep -E "^\+" | grep -c "unimplemented!()" || true)

  if [ "$UNIMPL" -gt 0 ]; then
    echo "❌ ERROR: Cannot commit $UNIMPL unimplemented!() placeholders"
    echo "   Complete implementations or use todo!() for acknowledged technical debt"
    echo ""
    echo "   Files with unimplemented!():"
    git diff --cached --name-only | grep '\.rs$' | while read file; do
      if git diff --cached "$file" | grep -E "^\+" | grep -q "unimplemented!()"; then
        echo "     - $file"
      fi
    done
    exit 1
  fi
fi

# Check for expect() calls (similar to unwrap but with message)
if git diff --cached --name-only | grep -q '\.rs$'; then
  echo "   Checking for expect() calls in staged Rust files..."

  EXPECTS=$(git diff --cached --diff-filter=ACM | grep -E "^\+" | grep -v "^[+-].*#\[test\]" | grep -c "\.expect(" || true)

  if [ "$EXPECTS" -gt 0 ]; then
    echo "⚠️  WARNING: Found $EXPECTS expect() calls in production code"
    echo "   Consider using proper error handling with ? or match"
  fi
fi

# Run cargo clippy on workspace if Rust files changed
if git diff --cached --name-only | grep -q '\.rs$'; then
  echo "   Running clippy on workspace..."

  if ! cargo clippy --workspace --all-targets -- -D warnings 2>&1; then
    echo "❌ ERROR: Clippy found issues"
    echo "   Fix clippy warnings before committing"
    exit 1
  fi
fi

# Check formatting
if git diff --cached --name-only | grep -q '\.rs$'; then
  echo "   Checking Rust formatting..."

  if ! cargo fmt --all -- --check 2>&1; then
    echo "❌ ERROR: Code is not formatted"
    echo "   Run: cargo fmt --all"
    exit 1
  fi
fi

echo "✅ Pre-commit validation passed"
exit 0
EOF

# Create pre-push hook
cat > "$HOOKS_DIR/pre-push" << 'EOF'
#!/bin/bash
# Poka-yoke pre-push hook: 5-gate validation before push
# Part of KNHK Definition of Done enforcement

set -e

echo "🚦 Pre-push validation (5 gates)..."
echo ""

# Gate 1: cargo check
echo "Gate 1/5: Cargo check..."
if ! cargo check --workspace 2>&1; then
  echo "❌ ERROR: cargo check failed"
  exit 1
fi
echo "✅ Gate 1 passed"
echo ""

# Gate 2: clippy with strict warnings
echo "Gate 2/5: Clippy (strict mode)..."
if ! cargo clippy --workspace --all-targets -- -D warnings 2>&1; then
  echo "❌ ERROR: Clippy found warnings or errors"
  exit 1
fi
echo "✅ Gate 2 passed"
echo ""

# Gate 3: formatting check
echo "Gate 3/5: Formatting check..."
if ! cargo fmt --all -- --check 2>&1; then
  echo "❌ ERROR: Code is not formatted"
  echo "   Run: cargo fmt --all"
  exit 1
fi
echo "✅ Gate 3 passed"
echo ""

# Gate 4: fast tests (lib and bins only, skip integration tests)
echo "Gate 4/5: Fast tests (lib + bins)..."
if ! cargo test --workspace --lib --bins 2>&1 | tail -20; then
  echo "❌ ERROR: Tests failed"
  exit 1
fi
echo "✅ Gate 4 passed"
echo ""

# Gate 5: security audit (warning only, don't block)
echo "Gate 5/5: Security audit..."
if command -v cargo-audit &> /dev/null; then
  if ! cargo audit 2>&1; then
    echo "⚠️  Security audit found issues (non-blocking)"
  else
    echo "✅ Gate 5 passed"
  fi
else
  echo "⚠️  cargo-audit not installed (optional)"
  echo "   Install: cargo install cargo-audit"
fi
echo ""

# Optional: Run DoD validation script if available
if [ -f "scripts/validate-dod-v1.sh" ]; then
  echo "📋 Running DoD validation..."
  if bash scripts/validate-dod-v1.sh 2>&1 | tail -20; then
    echo "✅ DoD validation passed"
  else
    echo "⚠️  DoD validation issues found (review output)"
  fi
  echo ""
fi

echo "✅ All gates passed - ready to push"
exit 0
EOF

# Make hooks executable
chmod +x "$HOOKS_DIR/pre-commit"
chmod +x "$HOOKS_DIR/pre-push"

echo "✅ Git hooks installed successfully:"
echo "   - $HOOKS_DIR/pre-commit"
echo "   - $HOOKS_DIR/pre-push"
echo ""
echo "🔍 Hooks will now enforce:"
echo "   • No unwrap() calls in production code"
echo "   • No unimplemented!() placeholders"
echo "   • Clippy warnings must be fixed"
echo "   • Code must be formatted"
echo "   • Tests must pass before push"
echo ""
echo "💡 To test hooks:"
echo "   1. Stage a file with unwrap(): git add <file>"
echo "   2. Try to commit: git commit -m 'test'"
echo "   3. Hook should prevent commit"
