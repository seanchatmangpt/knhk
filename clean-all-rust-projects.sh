#!/bin/bash

# Script to run cargo clean on all Rust projects found on the system
# This searches for all Cargo.toml files and runs cargo clean in their directories
#
# Usage: ./clean-all-rust-projects.sh [ROOT_DIR]
#   ROOT_DIR: Root directory to search (default: ~)
#
# Example:
#   ./clean-all-rust-projects.sh              # Search home directory (default)
#   ./clean-all-rust-projects.sh /            # Search entire system
#   ./clean-all-rust-projects.sh ~/projects   # Search specific directory

set -uo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Root directory to search (default: ~)
ROOT_DIR="${1:-~}"
# Expand ~ to full path
ROOT_DIR="${ROOT_DIR/#\~/$HOME}"

# Directories to exclude (system directories, package managers, etc.)
EXCLUDE_DIRS=(
    "/System"
    "/Library"
    "/private"
    "/usr/lib"
    "/usr/local/lib"
    "/opt/homebrew/Cellar"
    "/opt/homebrew/var"
    "/.Trash"
    "/.cache"
    "/tmp"
    "/var"
    "/dev"
    "/proc"
    "/sys"
    "/boot"
    "/lost+found"
)

echo -e "${BLUE}Searching for all Cargo.toml files starting from: ${ROOT_DIR}${NC}"
echo ""

# Build find command with exclusions
FIND_ARGS=("$ROOT_DIR")
for dir in "${EXCLUDE_DIRS[@]}"; do
    # Only add exclusion if it's an absolute path and exists
    if [[ "$dir" == /* ]] && [ -d "$dir" ]; then
        FIND_ARGS+=(-path "$dir" -prune -o)
    fi
done
FIND_ARGS+=(-name Cargo.toml -type f -print)

# Execute find and get unique directories, handling errors gracefully
# Also filter out target directories and .git directories
CARGO_DIRS=$(find "${FIND_ARGS[@]}" 2>/dev/null | while read -r file; do
    dir=$(dirname "$file" 2>/dev/null || true)
    if [ -n "$dir" ]; then
        # Skip if in target, .git, or .cargo directories
        if [[ "$dir" != *"/target"* ]] && [[ "$dir" != *"/.git"* ]] && [[ "$dir" != *"/.cargo"* ]]; then
            echo "$dir"
        fi
    fi
done | sort -u)

if [ -z "$CARGO_DIRS" ]; then
    echo -e "${YELLOW}No Cargo.toml files found.${NC}"
    exit 0
fi

# Count total projects
TOTAL=$(echo "$CARGO_DIRS" | wc -l | tr -d ' ')
echo -e "${BLUE}Found ${TOTAL} Rust project(s)${NC}"
echo ""

# Track statistics
SUCCESS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0
TOTAL_FILES_REMOVED=0
TOTAL_SIZE_REMOVED=0

# Process each directory
while IFS= read -r dir; do
    if [ ! -d "$dir" ]; then
        continue
    fi
    
    # Skip if directory doesn't have Cargo.toml (shouldn't happen, but be safe)
    if [ ! -f "$dir/Cargo.toml" ]; then
        continue
    fi
    
    echo -e "${BLUE}Cleaning: ${dir}${NC}"
    
    # Try to clean, capture output
    OUTPUT=""
    if OUTPUT=$(cd "$dir" && cargo clean 2>&1); then
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
        
        # Extract file count and size from cargo clean output (using sed for portability)
        if echo "$OUTPUT" | grep -q "Removed"; then
            # Extract number of files removed (portable sed instead of grep -oP)
            FILES=$(echo "$OUTPUT" | sed -n 's/.*Removed \([0-9]*\) files.*/\1/p' | head -1)
            FILES="${FILES:-0}"
            
            if [ "$FILES" != "0" ] && [ "$FILES" != "" ]; then
                echo -e "  ${GREEN}✓${NC} $OUTPUT"
                TOTAL_FILES_REMOVED=$((TOTAL_FILES_REMOVED + FILES))
            else
                echo -e "  ${GREEN}✓${NC} Already clean"
            fi
        else
            echo -e "  ${GREEN}✓${NC} Cleaned successfully"
        fi
    else
        EXIT_CODE=$?
        FAIL_COUNT=$((FAIL_COUNT + 1))
        
        # Check if it's a harmless error (file already deleted, etc.)
        if echo "$OUTPUT" | grep -qE "(No such file or directory|Directory not empty|already deleted|Permission denied)"; then
            echo -e "  ${YELLOW}⚠${NC} Warning: $(echo "$OUTPUT" | head -1)"
            SKIP_COUNT=$((SKIP_COUNT + 1))
        else
            echo -e "  ${RED}✗${NC} Error: $(echo "$OUTPUT" | head -1)"
        fi
    fi
    
    echo ""
    
done <<< "$CARGO_DIRS"

# Print summary
echo "=========================================="
echo -e "${BLUE}Summary:${NC}"
echo -e "  ${GREEN}Successfully cleaned:${NC} $SUCCESS_COUNT"
echo -e "  ${YELLOW}Warnings/Skipped:${NC} $SKIP_COUNT"
echo -e "  ${RED}Failed:${NC} $((FAIL_COUNT - SKIP_COUNT))"
echo -e "  ${BLUE}Total projects:${NC} $TOTAL"
if [ $TOTAL_FILES_REMOVED -gt 0 ]; then
    echo -e "  ${BLUE}Total files removed:${NC} $TOTAL_FILES_REMOVED"
fi
echo "=========================================="

# Exit with error if there were real failures (not just warnings)
if [ $((FAIL_COUNT - SKIP_COUNT)) -gt 0 ]; then
    exit 1
fi

exit 0

