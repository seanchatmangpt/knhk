#!/usr/bin/env bash
# Quick benchmark summary from existing results

CSV="/Users/sac/knhk/rust/crate_metrics.csv"

if [ ! -f "$CSV" ]; then
  echo "❌ No benchmark results found. Run ./scripts/benchmark_crates.sh first"
  exit 1
fi

echo "🚀 KNHK Workspace Performance Summary"
echo "======================================"
echo ""

# Parse CSV and compute totals (skip header)
tail -n +2 "$CSV" | awk -F',' '
BEGIN {
  total_loc=0; total_build_debug=0; total_build_release=0;
  total_test=0; total_clippy=0;
}
{
  total_loc+=$2;
  total_build_debug+=$3;
  total_build_release+=$4;
  total_test+=$5;
  total_clippy+=$6;
  count++;
}
END {
  printf "Total Crates:        %d\n", count;
  printf "Total LOC:           %s\n", total_loc;
  printf "Build (debug):       %.1fs (%.1fm)\n", total_build_debug, total_build_debug/60;
  printf "Build (release):     %.1fs (%.1fm)\n", total_build_release, total_build_release/60;
  printf "Test:                %.1fs (%.1fm)\n", total_test, total_test/60;
  printf "Clippy:              %.1fs\n", total_clippy;
  printf "\n";
  printf "Avg per crate:\n";
  printf "  Build (debug):     %.1fs\n", total_build_debug/count;
  printf "  Build (release):   %.1fs\n", total_build_release/count;
  printf "  Test:              %.1fs\n", total_test/count;
  printf "  Clippy:            %.1fs\n", total_clippy/count;
}'

echo ""
echo "📊 Top 3 Slowest:"
echo ""
echo "Debug Builds:"
tail -n +2 "$CSV" | sort -t',' -k3 -rn | head -3 | awk -F',' '{printf "  %s: %.1fs\n", $1, $3}'
echo ""
echo "Release Builds:"
tail -n +2 "$CSV" | sort -t',' -k4 -rn | head -3 | awk -F',' '{printf "  %s: %.1fs\n", $1, $4}'
echo ""
echo "Tests:"
tail -n +2 "$CSV" | sort -t',' -k5 -rn | head -3 | awk -F',' '{printf "  %s: %.1fs\n", $1, $5}'
echo ""
echo "✅ Full report: docs/PERFORMANCE_BENCHMARK.md"
