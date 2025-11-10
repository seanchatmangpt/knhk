#!/usr/bin/env bash
# KNHK Build Time Measurement Script
# Measures build times for each crate in the workspace
# Generates build-time documentation

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Output files
OUTPUT_DIR="$PROJECT_ROOT/docs/build-times"
JSON_OUTPUT="$OUTPUT_DIR/build-times.json"
MARKDOWN_OUTPUT="$OUTPUT_DIR/BUILD_TIMES.md"
CSV_OUTPUT="$OUTPUT_DIR/build-times.csv"

mkdir -p "$OUTPUT_DIR"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}⏱️  KNHK Build Time Measurement${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# Get system info
SYSTEM_INFO=$(uname -s)
CPU_CORES=$(sysctl -n hw.ncpu 2>/dev/null || nproc 2>/dev/null || echo "unknown")
RUST_VERSION=$(rustc --version 2>/dev/null || echo "unknown")
CARGO_VERSION=$(cargo --version 2>/dev/null || echo "unknown")

echo -e "${BLUE}System Information:${NC}"
echo "  OS: $SYSTEM_INFO"
echo "  CPU Cores: $CPU_CORES"
echo "  Rust: $RUST_VERSION"
echo "  Cargo: $CARGO_VERSION"
echo

# Ensure C library is built first (required for knhk-hot)
echo -e "${BLUE}Building C library first...${NC}"
cd "$PROJECT_ROOT/c"
if make lib >/dev/null 2>&1; then
  echo -e "${GREEN}✅ C library ready${NC}"
else
  echo -e "${YELLOW}⚠️  C library build failed, some crates may fail${NC}"
fi
cd "$PROJECT_ROOT"

# Get list of crates from workspace
cd "$PROJECT_ROOT/rust"
if command -v jq >/dev/null 2>&1; then
    CRATES=$(cargo metadata --format-version 1 2>/dev/null | jq -r '.workspace_members[]' | sed 's/ .*//' | xargs -I {} basename {} 2>/dev/null || echo "")
else
    # Fallback: read from Cargo.toml
    CRATES=$(grep -E '^\s*"' "$PROJECT_ROOT/rust/Cargo.toml" 2>/dev/null | sed 's/.*"\(.*\)".*/\1/' | grep -v '^$' || echo "")
fi

# If still empty, use hardcoded list (excluding knhk-unrdf)
if [ -z "$CRATES" ]; then
    CRATES="knhk-hot knhk-otel knhk-connectors knhk-lockchain knhk-etl knhk-warm knhk-validation knhk-config knhk-patterns knhk-workflow-engine knhk-sidecar knhk-cli knhk-integration-tests chicago-tdd-tools knhk-dflss knhk-latex-compiler knhk-latex"
fi

# Filter out knhk-unrdf if it exists in the list
CRATES=$(echo "$CRATES" | tr ' ' '\n' | grep -v "^knhk-unrdf$" | tr '\n' ' ' | sed 's/ $//')

# Arrays to store results
declare -a CRATE_NAMES
declare -a DEBUG_TIMES
declare -a RELEASE_TIMES
declare -a TEST_TIMES
declare -a CHECK_TIMES
declare -a LOC_COUNTS

TOTAL_DEBUG=0
TOTAL_RELEASE=0
TOTAL_TEST=0
TOTAL_CHECK=0

echo -e "${BLUE}Measuring build times for each crate...${NC}"
echo

# Function to measure time for a command
measure_time() {
    local cmd="$1"
    if command -v gdate >/dev/null 2>&1; then
        # macOS with GNU date
        local start=$(gdate +%s.%N)
        eval "$cmd" >/dev/null 2>&1
        local end=$(gdate +%s.%N)
    else
        # Standard date (Linux)
        local start=$(date +%s.%N)
        eval "$cmd" >/dev/null 2>&1
        local end=$(date +%s.%N)
    fi
    
    if command -v bc >/dev/null 2>&1; then
        echo "$end - $start" | bc
    else
        # Fallback: use awk for basic calculation
        awk "BEGIN {printf \"%.2f\", $end - $start}"
    fi
}

# Function to count lines of code (rough estimate)
count_loc() {
    local crate_path="$1"
    if [ -d "$crate_path/src" ]; then
        find "$crate_path/src" -name "*.rs" -type f 2>/dev/null | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}' || echo "0"
    else
        echo "0"
    fi
}

# Measure each crate
for crate in $CRATES; do
    crate_path="$PROJECT_ROOT/rust/$crate"
    
    if [ ! -d "$crate_path" ] || [ ! -f "$crate_path/Cargo.toml" ]; then
        continue
    fi
    
    echo -e "${BLUE}Measuring ${crate}...${NC}"
    
    # Count LOC
    loc=$(count_loc "$crate_path")
    
    # Clean build for accurate measurement
    cd "$crate_path"
    cargo clean >/dev/null 2>&1 || true
    
    # Measure debug build
    echo -n "  Debug build... "
    debug_time=$(measure_time "cargo build --quiet")
    debug_time_int=$(echo "$debug_time" | cut -d. -f1)
    echo -e "${GREEN}${debug_time_int}s${NC}"
    
    # Measure release build
    echo -n "  Release build... "
    release_time=$(measure_time "cargo build --release --quiet")
    release_time_int=$(echo "$release_time" | cut -d. -f1)
    echo -e "${GREEN}${release_time_int}s${NC}"
    
    # Measure test time
    echo -n "  Tests... "
    test_time=$(measure_time "cargo test --lib --quiet 2>&1")
    test_time_int=$(echo "$test_time" | cut -d. -f1)
    echo -e "${GREEN}${test_time_int}s${NC}"
    
    # Measure check time (faster than build)
    echo -n "  Check... "
    check_time=$(measure_time "cargo check --quiet")
    check_time_int=$(echo "$check_time" | cut -d. -f1)
    echo -e "${GREEN}${check_time_int}s${NC}"
    
    # Store results
    CRATE_NAMES+=("$crate")
    DEBUG_TIMES+=("$debug_time")
    RELEASE_TIMES+=("$release_time")
    TEST_TIMES+=("$test_time")
    CHECK_TIMES+=("$check_time")
    LOC_COUNTS+=("$loc")
    
    if command -v bc >/dev/null 2>&1; then
        TOTAL_DEBUG=$(echo "$TOTAL_DEBUG + $debug_time" | bc)
        TOTAL_RELEASE=$(echo "$TOTAL_RELEASE + $release_time" | bc)
        TOTAL_TEST=$(echo "$TOTAL_TEST + $test_time" | bc)
        TOTAL_CHECK=$(echo "$TOTAL_CHECK + $check_time" | bc)
    else
        TOTAL_DEBUG=$(awk "BEGIN {printf \"%.2f\", $TOTAL_DEBUG + $debug_time}")
        TOTAL_RELEASE=$(awk "BEGIN {printf \"%.2f\", $TOTAL_RELEASE + $release_time}")
        TOTAL_TEST=$(awk "BEGIN {printf \"%.2f\", $TOTAL_TEST + $test_time}")
        TOTAL_CHECK=$(awk "BEGIN {printf \"%.2f\", $TOTAL_CHECK + $check_time}")
    fi
    
    echo
done

cd "$PROJECT_ROOT"

# Generate JSON output
echo "{" > "$JSON_OUTPUT"
echo "  \"timestamp\": \"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\"," >> "$JSON_OUTPUT"
echo "  \"system\": {" >> "$JSON_OUTPUT"
echo "    \"os\": \"$SYSTEM_INFO\"," >> "$JSON_OUTPUT"
echo "    \"cpu_cores\": \"$CPU_CORES\"," >> "$JSON_OUTPUT"
echo "    \"rust_version\": \"$RUST_VERSION\"," >> "$JSON_OUTPUT"
echo "    \"cargo_version\": \"$CARGO_VERSION\"" >> "$JSON_OUTPUT"
echo "  }," >> "$JSON_OUTPUT"
echo "  \"totals\": {" >> "$JSON_OUTPUT"
echo "    \"debug_build\": $TOTAL_DEBUG," >> "$JSON_OUTPUT"
echo "    \"release_build\": $TOTAL_RELEASE," >> "$JSON_OUTPUT"
echo "    \"test\": $TOTAL_TEST," >> "$JSON_OUTPUT"
echo "    \"check\": $TOTAL_CHECK" >> "$JSON_OUTPUT"
echo "  }," >> "$JSON_OUTPUT"
echo "  \"crates\": [" >> "$JSON_OUTPUT"

for i in "${!CRATE_NAMES[@]}"; do
    crate="${CRATE_NAMES[$i]}"
    debug="${DEBUG_TIMES[$i]}"
    release="${RELEASE_TIMES[$i]}"
    test="${TEST_TIMES[$i]}"
    check="${CHECK_TIMES[$i]}"
    loc="${LOC_COUNTS[$i]}"
    
    echo "    {" >> "$JSON_OUTPUT"
    echo "      \"name\": \"$crate\"," >> "$JSON_OUTPUT"
    echo "      \"loc\": $loc," >> "$JSON_OUTPUT"
    echo "      \"debug_build\": $debug," >> "$JSON_OUTPUT"
    echo "      \"release_build\": $release," >> "$JSON_OUTPUT"
    echo "      \"test\": $test," >> "$JSON_OUTPUT"
    echo "      \"check\": $check" >> "$JSON_OUTPUT"
    if [ $i -lt $((${#CRATE_NAMES[@]} - 1)) ]; then
        echo "    }," >> "$JSON_OUTPUT"
    else
        echo "    }" >> "$JSON_OUTPUT"
    fi
done

echo "  ]" >> "$JSON_OUTPUT"
echo "}" >> "$JSON_OUTPUT"

# Generate CSV output
echo "crate,loc,debug_build,release_build,test,check" > "$CSV_OUTPUT"
for i in "${!CRATE_NAMES[@]}"; do
    crate="${CRATE_NAMES[$i]}"
    loc="${LOC_COUNTS[$i]}"
    debug="${DEBUG_TIMES[$i]}"
    release="${RELEASE_TIMES[$i]}"
    test="${TEST_TIMES[$i]}"
    check="${CHECK_TIMES[$i]}"
    echo "$crate,$loc,$debug,$release,$test,$check" >> "$CSV_OUTPUT"
done

# Generate Markdown documentation
cat > "$MARKDOWN_OUTPUT" <<EOF
# KNHK Build Time Documentation

**Last Updated**: $(date -u +"%Y-%m-%d %H:%M:%S UTC")  
**System**: $SYSTEM_INFO ($CPU_CORES cores)  
**Rust**: $RUST_VERSION  
**Cargo**: $CARGO_VERSION

---

## Summary

| Metric | Time | Notes |
|--------|------|-------|
| **Total Debug Build** | $(printf "%.1f" "$TOTAL_DEBUG")s ($(if command -v bc >/dev/null 2>&1; then echo "scale=1; $TOTAL_DEBUG/60" | bc; else awk "BEGIN {printf \"%.1f\", $TOTAL_DEBUG/60}"; fi)m) | Clean build, all crates |
| **Total Release Build** | $(printf "%.1f" "$TOTAL_RELEASE")s ($(if command -v bc >/dev/null 2>&1; then echo "scale=1; $TOTAL_RELEASE/60" | bc; else awk "BEGIN {printf \"%.1f\", $TOTAL_RELEASE/60}"; fi)m) | Clean build, all crates |
| **Total Test Time** | $(printf "%.1f" "$TOTAL_TEST")s ($(if command -v bc >/dev/null 2>&1; then echo "scale=1; $TOTAL_TEST/60" | bc; else awk "BEGIN {printf \"%.1f\", $TOTAL_TEST/60}"; fi)m) | Unit tests only |
| **Total Check Time** | $(printf "%.1f" "$TOTAL_CHECK")s ($(if command -v bc >/dev/null 2>&1; then echo "scale=1; $TOTAL_CHECK/60" | bc; else awk "BEGIN {printf \"%.1f\", $TOTAL_CHECK/60}"; fi)m) | Type checking only |

**Note**: These are clean build times. Incremental builds are typically 5-10x faster.

---

## Per-Crate Build Times

### Debug Build Times

| Crate | LOC | Debug Build | Release Build | Test | Check | Efficiency (LOC/s) |
|-------|-----|-------------|---------------|------|-------|---------------------|
EOF

# Sort by debug build time (descending)
declare -a sorted_indices
for i in "${!CRATE_NAMES[@]}"; do
    sorted_indices+=("$i")
done

# Simple bubble sort by debug time
for i in $(seq 0 $((${#CRATE_NAMES[@]} - 1))); do
    for j in $(seq $((i + 1)) $((${#CRATE_NAMES[@]} - 1))); do
        if command -v bc >/dev/null 2>&1; then
            should_swap=$(echo "${DEBUG_TIMES[$i]} < ${DEBUG_TIMES[$j]}" | bc)
        else
            should_swap=$(awk "BEGIN {print (${DEBUG_TIMES[$i]} < ${DEBUG_TIMES[$j]})}")
        fi
        if [ "$should_swap" = "1" ]; then
            # Swap
            temp="${sorted_indices[$i]}"
            sorted_indices[$i]="${sorted_indices[$j]}"
            sorted_indices[$j]="$temp"
        fi
    done
done

for idx in "${sorted_indices[@]}"; do
    crate="${CRATE_NAMES[$idx]}"
    loc="${LOC_COUNTS[$idx]}"
    debug="${DEBUG_TIMES[$idx]}"
    release="${RELEASE_TIMES[$idx]}"
    test="${TEST_TIMES[$idx]}"
    check="${CHECK_TIMES[$idx]}"
    
    # Calculate efficiency (LOC per second)
    if command -v bc >/dev/null 2>&1 && (( $(echo "$debug > 0" | bc -l) )); then
        efficiency=$(echo "scale=2; $loc / $debug" | bc)
    elif (( $(awk "BEGIN {print ($debug > 0)}") )); then
        efficiency=$(awk "BEGIN {printf \"%.2f\", $loc / $debug}")
    else
        efficiency="N/A"
    fi
    
    debug_int=$(echo "$debug" | cut -d. -f1)
    release_int=$(echo "$release" | cut -d. -f1)
    test_int=$(echo "$test" | cut -d. -f1)
    check_int=$(echo "$check" | cut -d. -f1)
    
    echo "| \`$crate\` | $loc | ${debug_int}s | ${release_int}s | ${test_int}s | ${check_int}s | $efficiency |" >> "$MARKDOWN_OUTPUT"
done

cat >> "$MARKDOWN_OUTPUT" <<EOF

---

## Build Time Categories

### Fast Builds (< 5s debug)
- Quick feedback for development
- Examples: \`knhk-config\`, \`knhk-otel\`, \`knhk-hot\`

### Medium Builds (5-30s debug)
- Moderate complexity
- Examples: \`knhk-etl\`, \`knhk-warm\`, \`knhk-unrdf\`

### Slow Builds (> 30s debug)
- Complex dependencies or large codebase
- Examples: \`knhk-cli\`, \`knhk-workflow-engine\`

---

## Incremental Build Times

Incremental builds (after code changes) are typically **5-10x faster** than clean builds:

| Scenario | Typical Time |
|----------|--------------|
| Single crate change | 1-5s |
| Dependency change | 5-15s |
| Workspace-wide change | 10-30s |

---

## Optimization Tips

1. **Use \`cargo check\`** instead of \`cargo build\` during development (2-3x faster)
2. **Build specific crates** instead of entire workspace: \`cargo build -p knhk-etl\`
3. **Use incremental compilation** (enabled by default)
4. **Consider sccache** for shared compilation cache across developers
5. **Profile slow builds**: \`cargo build --timings\`

---

## Historical Data

Historical build time data is stored in:
- \`build-times.json\` - Machine-readable JSON
- \`build-times.csv\` - CSV for analysis
- Previous measurements: See git history

---

## Running This Script

To update build times:

\`\`\`bash
./scripts/measure-build-times.sh
\`\`\`

This will:
1. Clean build each crate
2. Measure debug, release, test, and check times
3. Generate updated documentation
4. Save results in JSON and CSV formats

**Note**: This takes ~10-15 minutes for a full workspace clean build.

EOF

echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Build time measurement complete!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo "Results saved to:"
echo "  📄 $MARKDOWN_OUTPUT"
echo "  📊 $JSON_OUTPUT"
echo "  📈 $CSV_OUTPUT"
echo

