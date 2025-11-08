#!/usr/bin/env bash
# Benchmark all KNHK crates: build, test, clippy times + LOC + binary size

set -euo pipefail

WORKSPACE_ROOT="/Users/sac/knhk/rust"
RESULTS_FILE="$WORKSPACE_ROOT/benchmark_results.json"
METRICS_FILE="$WORKSPACE_ROOT/crate_metrics.csv"

# Color output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Initialize results
echo "{}" > "$RESULTS_FILE"
echo "crate,loc,build_debug_sec,build_release_sec,test_sec,clippy_sec,binary_size_kb" > "$METRICS_FILE"

# List of crates to benchmark
CRATES=(
  "knhk-aot"
  "knhk-cli"
  "knhk-config"
  "knhk-connectors"
  "knhk-etl"
  "knhk-hot"
  "knhk-integration-tests"
  "knhk-lockchain"
  "knhk-otel"
  "knhk-patterns"
  "knhk-sidecar"
  "knhk-unrdf"
  "knhk-validation"
  "knhk-warm"
)

# Function to measure time in seconds
time_command() {
  local start=$(date +%s.%N)
  eval "$1" > /dev/null 2>&1 || true
  local end=$(date +%s.%N)
  echo $(echo "$end - $start" | bc)
}

# Function to count LOC
count_loc() {
  local crate=$1
  find "$WORKSPACE_ROOT/$crate/src" -name "*.rs" -exec cat {} + | wc -l | tr -d ' '
}

# Function to get binary size (if applicable)
get_binary_size() {
  local crate=$1
  local binary_path="$WORKSPACE_ROOT/target/release/$crate"

  # Check if crate produces a binary
  if grep -q "^\[\[bin\]\]" "$WORKSPACE_ROOT/$crate/Cargo.toml" 2>/dev/null || \
     grep -q "^name = \"$crate\"" "$WORKSPACE_ROOT/$crate/Cargo.toml" 2>/dev/null; then
    if [ -f "$binary_path" ]; then
      local size_bytes=$(stat -f%z "$binary_path" 2>/dev/null || echo "0")
      echo $(( size_bytes / 1024 ))
    else
      echo "0"
    fi
  else
    echo "N/A"
  fi
}

# Benchmark each crate
total_build_debug=0
total_build_release=0
total_test=0
total_clippy=0
total_loc=0

echo -e "${GREEN}=== KNHK Crate Performance Benchmark ===${NC}\n"

for crate in "${CRATES[@]}"; do
  echo -e "${YELLOW}Benchmarking: $crate${NC}"

  # Clean before each crate
  cargo clean -p "$crate" 2>/dev/null || true

  # 1. Count LOC
  loc=$(count_loc "$crate")
  total_loc=$((total_loc + loc))
  echo "  LOC: $loc"

  # 2. Build time (debug)
  echo "  Building (debug)..."
  build_debug=$(time_command "cargo build --package $crate 2>&1")
  total_build_debug=$(echo "$total_build_debug + $build_debug" | bc)
  printf "  Build (debug): %.2f sec\n" "$build_debug"

  # 3. Build time (release)
  echo "  Building (release)..."
  build_release=$(time_command "cargo build --package $crate --release 2>&1")
  total_build_release=$(echo "$total_build_release + $build_release" | bc)
  printf "  Build (release): %.2f sec\n" "$build_release"

  # 4. Test time
  echo "  Testing..."
  test_time=$(time_command "cargo test --package $crate 2>&1")
  total_test=$(echo "$total_test + $test_time" | bc)
  printf "  Test: %.2f sec\n" "$test_time"

  # 5. Clippy time
  echo "  Clippy..."
  clippy_time=$(time_command "cargo clippy --package $crate -- -D warnings 2>&1")
  total_clippy=$(echo "$total_clippy + $clippy_time" | bc)
  printf "  Clippy: %.2f sec\n" "$clippy_time"

  # 6. Binary size
  binary_size=$(get_binary_size "$crate")
  echo "  Binary size: ${binary_size} KB"

  # Write to CSV
  echo "$crate,$loc,$build_debug,$build_release,$test_time,$clippy_time,$binary_size" >> "$METRICS_FILE"

  echo ""
done

# Summary
echo -e "${GREEN}=== Workspace Summary ===${NC}"
printf "Total LOC: %d\n" "$total_loc"
printf "Total Build (debug): %.2f sec (%.2f min)\n" "$total_build_debug" "$(echo "$total_build_debug / 60" | bc -l)"
printf "Total Build (release): %.2f sec (%.2f min)\n" "$total_build_release" "$(echo "$total_build_release / 60" | bc -l)"
printf "Total Test: %.2f sec (%.2f min)\n" "$total_test" "$(echo "$total_test / 60" | bc -l)"
printf "Total Clippy: %.2f sec (%.2f min)\n" "$total_clippy" "$(echo "$total_clippy / 60" | bc -l)"
echo ""

echo -e "${GREEN}Results saved to:${NC}"
echo "  - $METRICS_FILE"
echo "  - $RESULTS_FILE"
