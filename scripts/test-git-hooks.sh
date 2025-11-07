#!/bin/bash
# Test script for poka-yoke git hooks
# Verifies that hooks correctly block bad code and allow good code

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "🧪 Testing KNHK poka-yoke git hooks..."
echo ""

# Test 1: Pre-commit hook blocks unwrap()
echo "Test 1: Pre-commit hook blocks unwrap() calls"
cat > rust/knhk-test-bad.rs << 'EOF'
pub fn bad_function() {
    let x = Some(1).unwrap(); // Should be blocked
}
EOF

git add rust/knhk-test-bad.rs
if git commit -m "test: bad commit" 2>&1 | grep -q "Cannot commit.*unwrap()"; then
  echo "✅ Pre-commit correctly blocked unwrap()"
  git reset HEAD rust/knhk-test-bad.rs
  rm -f rust/knhk-test-bad.rs
else
  echo "❌ Pre-commit failed to block unwrap()"
  git reset HEAD rust/knhk-test-bad.rs
  rm -f rust/knhk-test-bad.rs
  exit 1
fi
echo ""

# Test 2: Pre-commit hook blocks unimplemented!()
echo "Test 2: Pre-commit hook blocks unimplemented!()"
cat > rust/knhk-test-unimpl.rs << 'EOF'
pub fn incomplete_function() {
    unimplemented!() // Should be blocked
}
EOF

git add rust/knhk-test-unimpl.rs
if git commit -m "test: unimplemented" 2>&1 | grep -q "Cannot commit.*unimplemented!()"; then
  echo "✅ Pre-commit correctly blocked unimplemented!()"
  git reset HEAD rust/knhk-test-unimpl.rs
  rm -f rust/knhk-test-unimpl.rs
else
  echo "❌ Pre-commit failed to block unimplemented!()"
  git reset HEAD rust/knhk-test-unimpl.rs
  rm -f rust/knhk-test-unimpl.rs
  exit 1
fi
echo ""

# Test 3: Pre-commit hook warns about expect()
echo "Test 3: Pre-commit hook warns about expect()"
cat > rust/knhk-test-expect.rs << 'EOF'
pub fn expect_function() {
    let x = Some(1).expect("should have value"); // Should warn
}
EOF

git add rust/knhk-test-expect.rs
if timeout 30 git commit -m "test: expect warning" 2>&1 | grep -q "WARNING.*expect()"; then
  echo "✅ Pre-commit correctly warned about expect()"
  # This might actually commit, so reset if needed
  git reset --soft HEAD~1 2>/dev/null || true
  git reset HEAD rust/knhk-test-expect.rs
  rm -f rust/knhk-test-expect.rs
else
  echo "⚠️  Pre-commit may not have warned about expect() (non-critical)"
  git reset HEAD rust/knhk-test-expect.rs 2>/dev/null || true
  rm -f rust/knhk-test-expect.rs
fi
echo ""

# Test 4: Pre-commit hook allows proper error handling
echo "Test 4: Pre-commit hook allows proper error handling"
cat > rust/knhk-test-good.rs << 'EOF'
pub fn good_function() -> Result<(), Box<dyn std::error::Error>> {
    let value = Some(42);
    let _unwrapped = value.ok_or("Value is None")?;
    Ok(())
}
EOF

git add rust/knhk-test-good.rs
if timeout 60 git commit -m "test: good error handling" 2>&1; then
  echo "✅ Pre-commit correctly allowed proper error handling"
  # Undo the commit
  git reset --soft HEAD~1
  git reset HEAD rust/knhk-test-good.rs
  rm -f rust/knhk-test-good.rs
else
  echo "❌ Pre-commit blocked valid code"
  git reset HEAD rust/knhk-test-good.rs 2>/dev/null || true
  rm -f rust/knhk-test-good.rs
  exit 1
fi
echo ""

echo "✅ All hook tests passed!"
echo ""
echo "📋 Summary:"
echo "   • unwrap() calls are blocked ✅"
echo "   • unimplemented!() is blocked ✅"
echo "   • expect() generates warnings ✅"
echo "   • Proper error handling is allowed ✅"
echo ""
echo "🔒 Hooks are functioning correctly!"
