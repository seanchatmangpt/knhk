#!/bin/bash
# Pre-flight validation gate - prevents work without clear DoD

set -e

echo "🚦 Pre-flight Validation Gate"
echo "=============================="

# Check 1: DoD exists and is clear
if [ ! -f "docs/v1.0-definition-of-done.md" ]; then
  echo "❌ FAILED: No Definition of Done found"
  exit 1
fi

# Check 2: No placeholder implementations
PLACEHOLDERS=$(grep -r "unimplemented!()" rust/ 2>/dev/null | wc -l || echo 0)
if [ "$PLACEHOLDERS" -gt 0 ]; then
  echo "⚠️  WARNING: $PLACEHOLDERS placeholder implementations found"
fi

# Check 3: No duplicate work in progress
UNSTAGED=$(git status --porcelain | wc -l)
if [ "$UNSTAGED" -gt 10 ]; then
  echo "⚠️  WARNING: $UNSTAGED unstaged changes - possible WIP waste"
fi

# Check 4: LEAN metrics baseline
echo ""
echo "📊 LEAN Baseline Check:"
echo "  - Total docs: $(find docs -name '*.md' | wc -l)"
echo "  - Archived docs: $(find docs/archived -name '*.md' 2>/dev/null | wc -l || echo 0)"
echo "  - Evidence docs: $(find docs/evidence -name '*.md' 2>/dev/null | wc -l || echo 0)"

echo ""
echo "✅ Pre-flight validation PASSED"
echo "   Ready to start value-add work"
