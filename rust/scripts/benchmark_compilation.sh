#!/bin/bash
set -euo pipefail

# KNHK Compilation Performance Benchmark
# Measures build times, LOC, and dependencies for all workspace packages

WORKSPACE_ROOT="/Users/sac/knhk/rust"
cd "$WORKSPACE_ROOT"

# ANSI colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# All workspace packages
PACKAGES=(
    "knhk-hot"
    "knhk-otel"
    "knhk-connectors"
    "knhk-lockchain"
    "knhk-unrdf"
    "knhk-etl"
    "knhk-warm"
    "knhk-aot"
    "knhk-validation"
    "knhk-config"
    "knhk-patterns"
    "knhk-cli"
    "knhk-integration-tests"
)

OUTPUT_FILE="$WORKSPACE_ROOT/docs/evidence/compilation_benchmark_$(date +%Y%m%d_%H%M%S).json"
mkdir -p "$(dirname "$OUTPUT_FILE")"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  KNHK Compilation Performance Benchmark${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# JSON output array
echo "[" > "$OUTPUT_FILE"

FIRST=true

for pkg in "${PACKAGES[@]}"; do
    echo -e "${GREEN}▶ Benchmarking: ${YELLOW}$pkg${NC}"

    # Add comma separator for JSON array (except first element)
    if [ "$FIRST" = true ]; then
        FIRST=false
    else
        echo "," >> "$OUTPUT_FILE"
    fi

    echo "  {" >> "$OUTPUT_FILE"
    echo "    \"package\": \"$pkg\"," >> "$OUTPUT_FILE"

    # 1. Count lines of code
    echo -n "  [1/5] Counting lines of code... "
    if [ -d "$pkg/src" ]; then
        LOC=$(find "$pkg/src" -name "*.rs" -type f -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}' || echo "0")
        FILE_COUNT=$(find "$pkg/src" -name "*.rs" -type f | wc -l | tr -d ' ')
    else
        LOC=0
        FILE_COUNT=0
    fi
    echo -e "${GREEN}✓${NC} $LOC lines in $FILE_COUNT files"
    echo "    \"lines_of_code\": $LOC," >> "$OUTPUT_FILE"
    echo "    \"file_count\": $FILE_COUNT," >> "$OUTPUT_FILE"

    # 2. Count dependencies (direct)
    echo -n "  [2/5] Analyzing dependencies... "
    DIRECT_DEPS=$(cargo metadata --manifest-path "$pkg/Cargo.toml" --no-deps --format-version 1 2>/dev/null | \
        jq -r ".packages[] | select(.name == \"$pkg\") | .dependencies | length" || echo "0")

    # Count transitive dependencies
    TRANSITIVE_DEPS=$(cargo tree -p "$pkg" --edges normal 2>/dev/null | grep -c "├\|└" || echo "0")

    echo -e "${GREEN}✓${NC} $DIRECT_DEPS direct, $TRANSITIVE_DEPS transitive"
    echo "    \"direct_dependencies\": $DIRECT_DEPS," >> "$OUTPUT_FILE"
    echo "    \"transitive_dependencies\": $TRANSITIVE_DEPS," >> "$OUTPUT_FILE"

    # 3. Clean build time
    echo -n "  [3/5] Clean build (release)... "
    cargo clean -p "$pkg" 2>/dev/null || true

    START_TIME=$(date +%s.%N)
    if cargo build -p "$pkg" --release --quiet 2>&1 | grep -q "error"; then
        CLEAN_BUILD_TIME="null"
        echo -e "${RED}✗ Failed${NC}"
    else
        END_TIME=$(date +%s.%N)
        CLEAN_BUILD_TIME=$(echo "$END_TIME - $START_TIME" | bc)
        echo -e "${GREEN}✓${NC} ${CLEAN_BUILD_TIME}s"
    fi
    echo "    \"clean_build_time_sec\": $CLEAN_BUILD_TIME," >> "$OUTPUT_FILE"

    # 4. Incremental rebuild time
    echo -n "  [4/5] Incremental rebuild... "
    if [ -d "$pkg/src" ]; then
        # Touch a random .rs file
        RANDOM_FILE=$(find "$pkg/src" -name "*.rs" -type f | head -1)
        if [ -n "$RANDOM_FILE" ]; then
            touch "$RANDOM_FILE"

            START_TIME=$(date +%s.%N)
            if cargo build -p "$pkg" --release --quiet 2>&1 | grep -q "error"; then
                INCREMENTAL_BUILD_TIME="null"
                echo -e "${RED}✗ Failed${NC}"
            else
                END_TIME=$(date +%s.%N)
                INCREMENTAL_BUILD_TIME=$(echo "$END_TIME - $START_TIME" | bc)
                echo -e "${GREEN}✓${NC} ${INCREMENTAL_BUILD_TIME}s"
            fi
        else
            INCREMENTAL_BUILD_TIME="null"
            echo -e "${YELLOW}⊘ No source files${NC}"
        fi
    else
        INCREMENTAL_BUILD_TIME="null"
        echo -e "${YELLOW}⊘ No src directory${NC}"
    fi
    echo "    \"incremental_build_time_sec\": $INCREMENTAL_BUILD_TIME," >> "$OUTPUT_FILE"

    # 5. Test compilation time (no run)
    echo -n "  [5/5] Test compilation... "
    START_TIME=$(date +%s.%N)
    if cargo test -p "$pkg" --no-run --quiet 2>&1 | grep -q "error"; then
        TEST_BUILD_TIME="null"
        echo -e "${RED}✗ Failed${NC}"
    else
        END_TIME=$(date +%s.%N)
        TEST_BUILD_TIME=$(echo "$END_TIME - $START_TIME" | bc)
        echo -e "${GREEN}✓${NC} ${TEST_BUILD_TIME}s"
    fi
    echo "    \"test_build_time_sec\": $TEST_BUILD_TIME" >> "$OUTPUT_FILE"

    echo "  }" >> "$OUTPUT_FILE"
    echo ""
done

echo "]" >> "$OUTPUT_FILE"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✓ Benchmark complete!${NC}"
echo -e "  Results saved to: ${YELLOW}$OUTPUT_FILE${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
