#!/bin/bash
# Deduplication Prevention Script
# Prevents duplicate evidence file creation
# DFLSS: Eliminates 4.7h motion waste (10%)

set -e

FILE=$1
INDEX="docs/EVIDENCE_INDEX.md"

if [ -z "$FILE" ]; then
    echo "=== KNHK Documentation Deduplication Prevention ==="
    echo ""
    echo "Usage: $0 <filename>"
    echo "Example: $0 weaver_validation_report.md"
    echo ""
    echo "This script checks if a file would be a duplicate before creation."
    exit 1
fi

# Strip path if provided
BASENAME=$(basename "$FILE")

echo "🔍 Checking for duplicates: $BASENAME"
echo ""

# Check if file is listed in Evidence Index as canonical
if [ -f "$INDEX" ]; then
    if grep -q "CANONICAL.*$BASENAME" "$INDEX" 2>/dev/null; then
        echo "✅ CANONICAL FILE: This is listed as a primary source"
        echo "   Safe to create/update"
        exit 0
    fi
fi
echo ""

# Check if file is listed in "Eliminated" section (DO NOT CREATE)
if [ -f "$INDEX" ]; then
    if grep -A 50 "Eliminated:" "$INDEX" | grep -q "$BASENAME" 2>/dev/null; then
        echo "❌ ERROR: Duplicate file detected!"
        echo ""
        echo "   File '$BASENAME' is listed in the ELIMINATED section of EVIDENCE_INDEX.md"
        echo ""
        echo "   📖 This file was identified as a duplicate. Use canonical source instead:"

        # Find the canonical source for this category
        SECTION=$(grep -B 10 "$BASENAME" "$INDEX" | grep "^####" | tail -1 || echo "Unknown category")
        CANONICAL=$(grep -A 3 "$SECTION" "$INDEX" | grep "CANONICAL:" | head -1 || echo "See EVIDENCE_INDEX.md")

        echo "   $CANONICAL"
        echo ""
        echo "   See: docs/EVIDENCE_INDEX.md for full list of canonical sources"
        exit 1
    fi
fi

# Check if filename matches common duplicate patterns
DUPLICATE_PATTERNS=(
    "validation-report"
    "validation_report"
    "compliance-report"
    "quality-report"
    "analysis-report"
    "status-report"
    "final-report"
    "weaver.*validation"
    "code.*quality"
    "performance.*validation"
    "orchestration.*status"
)

for PATTERN in "${DUPLICATE_PATTERNS[@]}"; do
    if echo "$BASENAME" | grep -qE "$PATTERN"; then
        echo "⚠️  WARNING: Filename matches common duplicate pattern: $PATTERN"
        echo ""
        echo "   📋 Canonical files in this category:"
        grep -A 3 "$(echo $PATTERN | sed 's/\.\*//')" "$INDEX" | grep "CANONICAL:" || echo "   (See EVIDENCE_INDEX.md)"
        echo ""
        read -p "   Continue anyway? (y/N): " CONFIRM
        if [ "$CONFIRM" != "y" ] && [ "$CONFIRM" != "Y" ]; then
            echo "   Aborting to prevent duplication"
            exit 1
        fi
    fi
done

# Check if file already exists in docs/evidence
if [ -f "docs/evidence/$BASENAME" ]; then
    echo "⚠️  FILE EXISTS: docs/evidence/$BASENAME"
    echo ""
    echo "   Options:"
    echo "   1. Update existing file instead of creating duplicate"
    echo "   2. Check if this is listed as canonical in EVIDENCE_INDEX.md"
    echo ""
    exit 1
fi

# Check if file already exists in docs/
if [ -f "docs/$BASENAME" ]; then
    echo "⚠️  FILE EXISTS: docs/$BASENAME"
    echo ""
    echo "   Options:"
    echo "   1. Update existing file instead of creating duplicate"
    echo "   2. Check if this is listed as canonical in EVIDENCE_INDEX.md"
    echo ""
    exit 1
fi

echo "✅ OK: New evidence file allowed"
echo ""
echo "📝 Next steps:"
echo "   1. Create file in: docs/evidence/$BASENAME"
echo "   2. Update $INDEX with new canonical source"
echo "   3. Store status: npx claude-flow@alpha hooks memory store \"evidence/<category>\" \"<status>\""
echo ""
exit 0
