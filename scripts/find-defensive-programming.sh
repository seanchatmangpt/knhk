#!/bin/bash
# Find defensive programming patterns in execution paths
# Defensive programming is prohibited - validation should happen at ingress only

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

echo "=== Finding Defensive Programming Patterns ==="
echo ""
echo "Searching for validation checks in execution paths..."
echo "(Hot path, executor, state management should assume pre-validated inputs)"
echo ""

# Patterns to search for
PATTERNS=(
    # Bounds checks
    "\.len\(\)\s*>\s*"
    "\.len\(\)\s*>=\s*"
    "\.len\(\)\s*<\s*"
    "\.len\(\)\s*<=\s*"
    "\.is_empty\(\)"
    "\.is_none\(\)"
    # Null/None checks
    "if\s+.*\s*==\s*None"
    "if\s+.*\s*!=\s*None"
    "if\s+.*\s*==\s*null"
    "if\s+.*\s*!=\s*null"
    # Array bounds
    "i\s*>=\s*.*\.len"
    "i\s*<\s*.*\.len"
    "index\s*>\s*.*\.len"
    "index\s*>=\s*.*\.len"
    # String checks
    "\.is_empty\(\)"
    "\.is_whitespace\(\)"
    # Option checks
    "\.is_some\(\)"
    "if\s+let\s+Some\("
    "match\s+.*\s*None"
)

# Directories to search (execution paths)
EXECUTION_PATHS=(
    "rust/knhk-workflow-engine/src/performance"
    "rust/knhk-workflow-engine/src/executor"
    "rust/knhk-workflow-engine/src/state"
    "rust/knhk-hot/src"
    "c/src"
)

# Directories to exclude (ingress points where validation is OK)
INGRESS_PATHS=(
    "rust/knhk-workflow-engine/src/security/guards.rs"
    "rust/knhk-workflow-engine/src/services/admission.rs"
)

FOUND=0

echo "Searching execution paths for defensive checks..."
echo ""

for dir in "${EXECUTION_PATHS[@]}"; do
    if [ ! -d "$dir" ]; then
        continue
    fi
    
    echo "--- Checking $dir ---"
    
    # Find Rust files
    while IFS= read -r file; do
        if [ -z "$file" ]; then
            continue
        fi
        
        # Skip ingress paths
        SKIP=false
        for ingress in "${INGRESS_PATHS[@]}"; do
            if [[ "$file" == *"$ingress"* ]]; then
                SKIP=true
                break
            fi
        done
        
        if [ "$SKIP" = true ]; then
            continue
        fi
        
        # Check each pattern
        for pattern in "${PATTERNS[@]}"; do
            matches=$(grep -n -E "$pattern" "$file" 2>/dev/null || true)
            if [ -n "$matches" ]; then
                # Filter out comments and doc comments
                while IFS= read -r line; do
                    if [[ ! "$line" =~ ^[[:space:]]*// ]] && [[ ! "$line" =~ ^[[:space:]]*/// ]] && [[ ! "$line" =~ ^[[:space:]]*\* ]]; then
                        echo "  $file: $line"
                        FOUND=$((FOUND + 1))
                    fi
                done <<< "$matches"
            fi
        done
    done < <(find "$dir" -type f \( -name "*.rs" -o -name "*.c" -o -name "*.h" \) 2>/dev/null || true)
done

echo ""
echo "=== Summary ==="
echo "Found $FOUND potential defensive programming instances"
echo ""

if [ $FOUND -eq 0 ]; then
    echo "✅ No defensive programming found in execution paths"
    exit 0
else
    echo "⚠️  Found defensive programming patterns in execution paths"
    echo "   These should be moved to ingress validation (guards, admission gates)"
    exit 1
fi

