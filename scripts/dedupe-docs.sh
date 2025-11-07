#!/bin/bash
# Deduplication Detection Script
# Finds duplicate documentation content in KNHK project

set -e

echo "=== KNHK Documentation Deduplication Analyzer ==="
echo ""

# Find duplicate files by hash
echo "1. CHECKING FOR IDENTICAL FILES (by hash)..."
echo "================================================"
if command -v md5sum &> /dev/null; then
  find docs -name "*.md" -type f -exec md5sum {} \; | sort | awk '{print $1}' | uniq -d
elif command -v md5 &> /dev/null; then
  find docs -name "*.md" -type f -exec md5 {} \; | sort | awk '{print $4}' | uniq -d
fi
echo ""

# Find files with duplicate section headings
echo "2. CHECKING FOR DUPLICATE SECTION HEADINGS..."
echo "=============================================="
temp_file=$(mktemp)
find docs -name "*.md" -type f | while read file; do
  grep "^## " "$file" 2>/dev/null | while read heading; do
    echo "$heading|$file"
  done
done > "$temp_file"

# Count duplicate headings
awk -F'|' '{print $1}' "$temp_file" | sort | uniq -c | sort -rn | awk '$1 > 2 {print $1 " occurrences: " $2}'

echo ""
echo "3. ANALYZING DFLSS DOCUMENTATION..."
echo "===================================="
if [ -d "docs/evidence" ]; then
  cd docs/evidence

  echo "Total DFLSS files: $(find . -name "dflss_*.md" | wc -l)"
  echo "Total lines: $(cat dflss_*.md 2>/dev/null | wc -l)"

  echo ""
  echo "Duplicate section analysis:"
  grep -h "^## " dflss_*.md 2>/dev/null | sort | uniq -c | sort -rn | head -10

  cd - > /dev/null
fi

echo ""
echo "4. CHECKING FOR DUPLICATE CONTENT (similar files)..."
echo "====================================================="
# Find files with >70% similar headings
find docs/evidence -name "dflss_*.md" -type f | while read file1; do
  find docs/evidence -name "dflss_*.md" -type f | while read file2; do
    if [ "$file1" != "$file2" ] && [ "$file1" \< "$file2" ]; then
      headings1=$(grep "^## " "$file1" 2>/dev/null | sort)
      headings2=$(grep "^## " "$file2" 2>/dev/null | sort)

      common=$(comm -12 <(echo "$headings1") <(echo "$headings2") | wc -l)
      total=$(echo "$headings1" | wc -l)

      if [ $total -gt 0 ]; then
        similarity=$((common * 100 / total))
        if [ $similarity -gt 70 ]; then
          echo "DUPLICATE CANDIDATE ($similarity% similar):"
          echo "  - $(basename "$file1")"
          echo "  - $(basename "$file2")"
        fi
      fi
    fi
  done
done

echo ""
echo "5. RECOMMENDATIONS..."
echo "====================="
echo "✓ Check files with identical hashes (exact duplicates)"
echo "✓ Consolidate files with >70% similar headings"
echo "✓ For sections appearing >3 times, create single source of truth"
echo "✓ Use EVIDENCE_INDEX.md to link to canonical sources"
echo ""

# Cleanup
rm -f "$temp_file"

echo "Done. Review findings above and consolidate duplicates."
