#!/usr/bin/env bash
# KNHK Optimized Test Runner
# Uses pre-compiled test binaries and result caching for maximum speed

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Configuration
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CACHE_DIR="${PROJECT_ROOT}/.test-cache"
RESULT_CACHE_DIR="${CACHE_DIR}/results"
HASH_FILE="${CACHE_DIR}/code.hash"
TIMEOUT=5

# Ensure cache directories exist
mkdir -p "$CACHE_DIR" "$RESULT_CACHE_DIR"

# Generate hash of all Rust source files
generate_code_hash() {
  find "$PROJECT_ROOT/rust" -name "*.rs" -type f -exec sha256sum {} \; 2>/dev/null | \
    sort | sha256sum | cut -d' ' -f1
}

# Check if test results are cached
check_result_cache() {
  local current_hash=$(generate_code_hash)
  local cached_result="${RESULT_CACHE_DIR}/${current_hash}.json"
  
  if [ -f "$cached_result" ]; then
    local cache_age=$(($(date +%s) - $(stat -f %m "$cached_result" 2>/dev/null || stat -c %Y "$cached_result" 2>/dev/null)))
    
    # Use cache if less than 1 hour old
    if [ "$cache_age" -lt 3600 ]; then
      echo "$cached_result"
      return 0
    fi
  fi
  
  return 1
}

# Save test results to cache
save_result_cache() {
  local result_file="$1"
  local current_hash=$(generate_code_hash)
  local cached_result="${RESULT_CACHE_DIR}/${current_hash}.json"
  
  cp "$result_file" "$cached_result"
  
  # Clean old cache entries (keep last 10)
  ls -t "$RESULT_CACHE_DIR"/*.json 2>/dev/null | tail -n +11 | xargs rm -f 2>/dev/null || true
}

# Check if test binaries are pre-compiled
check_test_binaries() {
  # Check if test binaries exist and are recent
  local test_binary_count=$(find "$PROJECT_ROOT/rust/target" -name "*test*" -type f -newer "$HASH_FILE" 2>/dev/null | wc -l | tr -d ' ')
  
  if [ "$test_binary_count" -gt 0 ]; then
    return 0  # Binaries exist and are recent
  fi
  
  return 1  # Need to compile
}

# Run tests with maximum optimization
run_tests_optimized() {
  local test_type="${1:-all}"
  local start_time=$(date +%s)
  
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${BLUE}⚡ KNHK Optimized Test Runner${NC}"
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo
  
  # Check result cache first
  if cached_result=$(check_result_cache); then
    echo -e "${GREEN}✅ Using cached test results (code unchanged)${NC}"
    cat "$cached_result"
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    echo -e "${GREEN}⏱️  Completed in ${duration}s (cached)${NC}"
    return 0
  fi
  
  # Check if test binaries are pre-compiled
  if ! check_test_binaries; then
    echo -e "${YELLOW}⚠️  Test binaries not pre-compiled, compiling now...${NC}"
    cd "$PROJECT_ROOT/rust"
    CARGO_INCREMENTAL=1 cargo build --tests --workspace > /dev/null 2>&1 || {
      echo -e "${RED}❌ Test binary compilation failed${NC}"
      return 1
    }
  else
    echo -e "${GREEN}✅ Using pre-compiled test binaries${NC}"
  fi
  
  # Run tests with cargo-nextest if available, otherwise cargo test
  cd "$PROJECT_ROOT/rust"
  local result_file="${CACHE_DIR}/test-results.json"
  
  if command -v cargo-nextest > /dev/null 2>&1; then
    echo -e "${BLUE}🚀 Running tests with cargo-nextest (maximum parallelization)...${NC}"
    
    # Use fast profile for speed
    if cargo nextest run --workspace --profile fast --no-build --format json > "$result_file" 2>&1; then
      echo -e "${GREEN}✅ All tests passed${NC}"
      save_result_cache "$result_file"
      local end_time=$(date +%s)
      local duration=$((end_time - start_time))
      echo -e "${GREEN}⏱️  Completed in ${duration}s${NC}"
      return 0
    else
      echo -e "${RED}❌ Some tests failed${NC}"
      cat "$result_file"
      return 1
    fi
  else
    echo -e "${YELLOW}⚠️  cargo-nextest not installed, using cargo test${NC}"
    echo -e "${BLUE}   Install with: cargo install cargo-nextest${NC}"
    
    # Use cargo test with maximum parallelization
    local cpu_count=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
    
    if CARGO_INCREMENTAL=1 cargo test --workspace --lib --no-fail-fast -- --test-threads="$cpu_count" > "$result_file" 2>&1; then
      echo -e "${GREEN}✅ All tests passed${NC}"
      save_result_cache "$result_file"
      local end_time=$(date +%s)
      local duration=$((end_time - start_time))
      echo -e "${GREEN}⏱️  Completed in ${duration}s${NC}"
      return 0
    else
      echo -e "${RED}❌ Some tests failed${NC}"
      cat "$result_file"
      return 1
    fi
  fi
}

# Main execution
case "${1:-all}" in
  all)
    run_tests_optimized "all"
    ;;
  chicago)
    run_tests_optimized "chicago"
    ;;
  performance)
    run_tests_optimized "performance"
    ;;
  *)
    echo "Usage: $0 {all|chicago|performance}"
    exit 1
    ;;
esac

