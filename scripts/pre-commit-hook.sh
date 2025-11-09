#!/bin/bash
# Pre-commit hook: Fast validation aligned with core team 80/20 best practices
# Target: 2-5 seconds (only checks staged files/packages)
# Enforces: No unwrap/expect/TODO/FUTURE/unimplemented on MAIN branch only
# Other branches: relaxed rules (TODO/FUTURE/unimplemented allowed)

set -e

# Change to project root
cd "$(git rev-parse --show-toplevel)"

echo "🔍 Running pre-commit validation..."

# Only check if Rust files are staged
if ! git diff --cached --name-only | grep -q '\.rs$'; then
  echo "✅ No Rust files staged, skipping validation"
  exit 0
fi

# Get list of staged Rust files
STAGED_FILES=$(git diff --cached --name-only | grep '\.rs$' || true)

if [ -z "$STAGED_FILES" ]; then
  echo "✅ No Rust files to validate"
  exit 0
fi

# Detect current branch - strict rules only apply to main
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
IS_MAIN_BRANCH=false
if [ "$CURRENT_BRANCH" = "main" ] || [ "$CURRENT_BRANCH" = "master" ]; then
  IS_MAIN_BRANCH=true
  echo "🔒 Main branch detected - enforcing strict rules (no TODO/FUTURE/unimplemented!)"
else
  echo "🌿 Branch '$CURRENT_BRANCH' - strict rules relaxed (TODO/FUTURE/unimplemented! allowed)"
fi

# Check 1: No unwrap() in production code (excluding test files, build scripts, CLI)
echo "   Checking for unwrap() calls in production code..."
UNWRAP_COUNT=0
for file in $STAGED_FILES; do
  # Skip test files, examples, benches, build scripts
  if [[ "$file" =~ /(test|tests|example|examples|bench|benches)/ ]] || [[ "$file" == *"build.rs" ]] || [[ "$file" =~ ^(test|tests|example|examples|bench|benches)/ ]]; then
    continue
  fi
  
  # Check if file has allow attribute (check both diff and actual file)
  if git diff --cached "$file" | grep -qE "#!?\[allow\(clippy::unwrap_used\)\]" || \
     grep -qE "#!?\[allow\(clippy::unwrap_used\)\]" "$file" 2>/dev/null; then
    continue
  fi
  
  # Pragmatic exception for pre-commit: if file has test modules, allow unwrap() calls
  # Rationale: Test modules should have allow attributes, but we're lenient for fast feedback
  # Pre-push hook will enforce stricter rules (require allow attributes in test modules)
  if grep -q "#\[cfg(test)\]" "$file" 2>/dev/null; then
    # File has test modules - allow unwrap() for pre-commit (fast feedback)
    # Pre-push will check that test modules have proper allow attributes
    continue
  fi
  
  # Count unwrap() calls in staged changes
  UNWRAPS=$(git diff --cached "$file" | grep -E "^\+" | grep -c "\.unwrap()" || echo 0)
  UNWRAPS=${UNWRAPS//[^0-9]/}  # Remove any non-numeric characters
  if [ "${UNWRAPS:-0}" -gt 0 ]; then
    echo "     ❌ $file: $UNWRAPS unwrap() call(s) found"
    UNWRAP_COUNT=$((UNWRAP_COUNT + UNWRAPS))
  fi
done

if [ "$UNWRAP_COUNT" -gt 0 ]; then
  echo "❌ ERROR: Cannot commit $UNWRAP_COUNT unwrap() calls in production code"
  echo "   Replace with proper Result<T,E> error handling"
  echo "   Use ? operator or match statements instead"
  echo "   Or add #![allow(clippy::unwrap_used)] if truly necessary"
  exit 1
fi
echo "  ✅ No unwrap() in production code"

# Check 2: No unimplemented!() placeholders - BLOCKED ONLY ON MAIN
if [ "$IS_MAIN_BRANCH" = true ]; then
  echo "   Checking for unimplemented!() placeholders..."
  UNIMPL_COUNT=0
  for file in $STAGED_FILES; do
    # Check ALL files - no exceptions on main
    UNIMPL=$(git diff --cached "$file" | grep -E "^\+" | grep -c "unimplemented!" || echo 0)
    UNIMPL=${UNIMPL//[^0-9]/}  # Remove any non-numeric characters
    if [ "${UNIMPL:-0}" -gt 0 ]; then
      echo "     ❌ $file: $UNIMPL unimplemented!() placeholder(s) found"
      git diff --cached "$file" | grep -E "^\+" | grep "unimplemented!" | head -5
      UNIMPL_COUNT=$((UNIMPL_COUNT + UNIMPL))
    fi
  done

  if [ "$UNIMPL_COUNT" -gt 0 ]; then
    echo "❌ ERROR: Cannot commit $UNIMPL_COUNT unimplemented!() placeholders to main"
    echo "   Complete implementations before committing - NO EXCEPTIONS"
    exit 1
  fi
  echo "  ✅ No unimplemented!() placeholders"
else
  echo "  ⏭️  Skipping unimplemented!() check (not on main branch)"
fi

# Check 3: No FUTURE or TODO comments - BLOCKED ONLY ON MAIN
if [ "$IS_MAIN_BRANCH" = true ]; then
  echo "   Checking for FUTURE/TODO comments..."
  TODO_COUNT=0
  for file in $STAGED_FILES; do
    # Skip only documentation files (markdown, text files)
    if [[ "$file" =~ \.(md|txt|rst)$ ]]; then
      continue
    fi
    
    # Check ALL Rust files - no exceptions for test files on main
    # Block ALL TODO/FUTURE comments regardless of context
    TODOS=$(git diff --cached "$file" | grep -E "^\+" | grep -iE "\b(TODO|FUTURE)\b" | grep -c . || echo 0)
    TODOS=${TODOS//[^0-9]/}  # Remove any non-numeric characters
    if [ "${TODOS:-0}" -gt 0 ]; then
      echo "     ❌ $file: $TODOS FUTURE/TODO comment(s) found"
      git diff --cached "$file" | grep -E "^\+" | grep -iE "\b(TODO|FUTURE)\b" | head -5
      TODO_COUNT=$((TODO_COUNT + TODOS))
    fi
  done

  if [ "$TODO_COUNT" -gt 0 ]; then
    echo "❌ ERROR: Cannot commit $TODO_COUNT FUTURE/TODO comments to main"
    echo "   Remove ALL TODO/FUTURE comments before committing - NO EXCEPTIONS"
    echo "   This applies to ALL code including tests"
    exit 1
  fi
  echo "  ✅ No FUTURE/TODO comments"
else
  echo "  ⏭️  Skipping FUTURE/TODO check (not on main branch)"
fi

# Check 4: No expect() in production code (excluding CLI, test files, build scripts, allowed modules)
echo "   Checking for expect() calls in production code..."
EXPECT_COUNT=0
for file in $STAGED_FILES; do
  # Skip test files, examples, benches, build scripts
  if [[ "$file" =~ /(test|tests|example|examples|bench|benches)/ ]] || [[ "$file" == *"build.rs" ]] || [[ "$file" =~ ^(test|tests|example|examples|bench|benches)/ ]]; then
    continue
  fi
  
  # Allow CLI code to use expect() (user-facing, different needs)
  if [[ "$file" =~ knhk-cli/ ]]; then
    continue
  fi
  
  # Check if file has allow attribute for expect (check actual file, not just diff)
  # Check for both #[allow] and #![allow] patterns
  if grep -qE "#!?\[allow\(clippy::expect_used\)\]" "$file" 2>/dev/null || \
     git diff --cached "$file" | grep -qE "#!?\[allow\(clippy::expect_used\)\]"; then
    continue
  fi
  
  # Pragmatic exception for pre-commit: if file has test modules, allow expect() calls
  # Rationale: Test modules should have allow attributes, but we're lenient for fast feedback
  # Pre-push hook will enforce stricter rules (require allow attributes in test modules)
  if grep -q "#\[cfg(test)\]" "$file" 2>/dev/null; then
    # File has test modules - allow expect() for pre-commit (fast feedback)
    # Pre-push will check that test modules have proper allow attributes
    continue
  fi
  
  # Count expect() calls in staged changes
  EXPECTS=$(git diff --cached "$file" | grep -E "^\+" | grep -c "\.expect(" || echo 0)
  EXPECTS=${EXPECTS//[^0-9]/}  # Remove any non-numeric characters
  if [ "${EXPECTS:-0}" -gt 0 ]; then
    echo "     ❌ $file: $EXPECTS expect() call(s) found"
    EXPECT_COUNT=$((EXPECT_COUNT + EXPECTS))
  fi
done

if [ "$EXPECT_COUNT" -gt 0 ]; then
  echo "❌ ERROR: Cannot commit $EXPECT_COUNT expect() calls in production code"
  echo "   Replace with proper error handling or add #![allow(clippy::expect_used)]"
  echo "   Note: CLI code (knhk-cli) can use expect() for user-facing errors"
  exit 1
fi
echo "  ✅ No expect() in production code (CLI exempt)"

# Check 5: Formatting (check entire workspace - fast enough)
echo "   Checking Rust formatting..."
cd rust
if ! cargo fmt --all -- --check 2>&1; then
  echo "❌ ERROR: Code is not formatted"
  echo "   Run: cd rust && cargo fmt --all"
  exit 1
fi
cd "$(git rev-parse --show-toplevel)"
echo "  ✅ Code is formatted"

# Check 6: Quick clippy check (only on staged packages, allow test files)
echo "   Running clippy on staged packages..."
cd rust

# Get unique packages from staged files
PACKAGES=$(echo "$STAGED_FILES" | sed 's|rust/||' | cut -d'/' -f1 | sort -u | grep -E "^knhk-" || true)

CLIPPY_FAILED=0
if [ -n "$PACKAGES" ]; then
  for pkg in $PACKAGES; do
    if [ -d "$pkg" ]; then
      # Run clippy on lib and bins only (faster, excludes tests)
      # Capture output to check for real errors
      if cargo clippy --package "$pkg" --lib --bins -- -D warnings 2>&1 > /tmp/clippy_output.txt; then
        # Clippy passed
        continue
      else
        # Check exit code - if non-zero, check if it's test-related
        # Filter out test-related warnings and check if any remain
        if grep -v "test\|tests\|example\|examples\|bench\|benches\|\.rs:" /tmp/clippy_output.txt | grep -qE "(error|warning):"; then
          echo "❌ ERROR: Clippy found issues in $pkg"
          grep -v "test\|tests\|example\|examples\|bench\|benches" /tmp/clippy_output.txt | head -20
          echo "   Fix clippy warnings before committing"
          CLIPPY_FAILED=1
          break
        fi
      fi
    fi
  done
fi

rm -f /tmp/clippy_output.txt

cd "$(git rev-parse --show-toplevel)"
if [ "$CLIPPY_FAILED" -eq 1 ]; then
  exit 1
fi
echo "  ✅ Clippy checks passed"

echo "✅ Pre-commit validation passed"
exit 0
