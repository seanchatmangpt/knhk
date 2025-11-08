#!/bin/bash
# Quick partial analysis of benchmark results (even incomplete JSON)

BENCH_FILE="/Users/sac/knhk/rust/docs/evidence/compilation_benchmark_20251107_174151.json"

echo "===================================================================================="
echo "PARTIAL BENCHMARK ANALYSIS (Real-time)"
echo "===================================================================================="
echo ""

# Extract completed package data (even from incomplete JSON)
echo "Completed Packages:"
echo "------------------------------------------------------------------------------------"
printf "%-25s %10s %8s %15s %15s %15s\n" "Package" "LOC" "Files" "Clean Build" "Incremental" "Test Build"
echo "------------------------------------------------------------------------------------"

# Use grep and awk to extract completed entries
grep -A 10 '"package"' "$BENCH_FILE" 2>/dev/null | \
awk '
  /"package":/ { package=$2; gsub(/[",]/, "", package) }
  /"lines_of_code":/ { loc=$2; gsub(/,/, "", loc) }
  /"file_count":/ { files=$2; gsub(/,/, "", files) }
  /"clean_build_time_sec":/ { clean=$2; gsub(/,/, "", clean) }
  /"incremental_build_time_sec":/ { incr=$2; gsub(/,/, "", incr) }
  /"test_build_time_sec":/ {
    test=$2
    gsub(/,/, "", test)
    if (test != "" && test != "null") {
      printf "%-25s %10s %8s %15.2fs %15.2fs %15.2fs\n", package, loc, files, clean, incr, test
    }
  }
'

echo "===================================================================================="
echo ""
echo "Current Status:"
ps aux | grep "[b]enchmark_compilation.sh" | head -1 | awk '{print "  Running since:", $9, "(PID:"$2")"}'
echo "  File size: $(ls -lh "$BENCH_FILE" 2>/dev/null | awk '{print $5}')"
echo ""
echo "Refresh this analysis with: $0"
echo "===================================================================================="
