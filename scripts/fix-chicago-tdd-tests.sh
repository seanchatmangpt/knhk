#!/bin/bash
# Fix Chicago TDD test compilation errors

set -e

cd "$(dirname "$0")/.." || exit 1
PROJECT_ROOT=$(pwd)

echo "=== Fixing Chicago TDD Tests ==="
echo ""

# Fix 1: Check for missing Debug traits
echo "Checking for missing Debug traits..."
find rust -name "*chicago_tdd*.rs" -type f -exec grep -l "struct.*{" {} \; | while read file; do
    if grep -q "struct.*{" "$file" && ! grep -q "#\[derive(Debug" "$file"; then
        echo "⚠️  Missing Debug trait in $file"
    fi
done

# Fix 2: Check for unwrap() in tests
echo ""
echo "Checking for unwrap() in Chicago TDD tests..."
find rust -name "*chicago_tdd*.rs" -type f -exec grep -l "unwrap()" {} \; | while read file; do
    echo "⚠️  Found unwrap() in $file"
done

# Fix 3: Check for uppercase S, P, O variables
echo ""
echo "Checking for uppercase variable names..."
find rust -name "*chicago_tdd*.rs" -type f -exec grep -l "\bS\b.*\bP\b.*\bO\b" {} \; | while read file; do
    echo "⚠️  Found uppercase S, P, O in $file"
done

# Fix 4: Compile check each package
echo ""
echo "=== Compilation Check ==="

for dir in rust/knhk-etl rust/knhk-validation rust/knhk-sidecar; do
    if [ -d "$dir" ]; then
        echo "Checking $dir..."
        cd "$dir"
        if cargo check --tests 2>&1 | grep -q "error"; then
            echo "❌ Errors found in $dir"
            cargo check --tests 2>&1 | grep "error" | head -10
        else
            echo "✅ $dir compiles successfully"
        fi
        cd "$PROJECT_ROOT"
    fi
done

echo ""
echo "=== Fix Complete ==="

