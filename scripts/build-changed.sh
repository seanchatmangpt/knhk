#!/usr/bin/env bash
# Build only changed Rust crates
# Uses git diff to detect changes and only builds affected crates

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🔍 Building Changed Crates Only${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# Detect changed crates
CHANGED_CRATES=$(git diff --name-only HEAD 2>/dev/null | \
  grep -E '^rust/(knhk-[^/]+|chicago-tdd-tools)/' | \
  cut -d'/' -f2 | \
  sort -u || true)

if [ -z "$CHANGED_CRATES" ]; then
  echo -e "${YELLOW}⚠️  No changed Rust crates detected${NC}"
  echo "   Checking if any crates need rebuild..."
  
  # Check if any crates are missing from target/
  cd "$PROJECT_ROOT/rust"
  for crate in */Cargo.toml; do
    crate_name=$(basename "$(dirname "$crate")")
    if [ ! -f "target/debug/deps/lib${crate_name//-/_}.rlib" ] 2>/dev/null; then
      CHANGED_CRATES="${CHANGED_CRATES}${crate_name} "
    fi
  done
  
  if [ -z "$CHANGED_CRATES" ]; then
    echo -e "${GREEN}✅ All crates are up to date${NC}"
    exit 0
  fi
fi

echo -e "${BLUE}Changed crates:${NC}"
for crate in $CHANGED_CRATES; do
  echo -e "  • ${crate}"
done
echo

# Build changed crates with minimal features for fast builds
cd "$PROJECT_ROOT/rust"
BUILT=0
FAILED=0

for crate in $CHANGED_CRATES; do
  echo -e "${BLUE}Building ${crate}...${NC}"
  
  # Use minimal features for slow crates
  local features=""
  case "$crate" in
    knhk-cli)
      features="--features minimal"
      ;;
    knhk-workflow-engine)
      features="--no-default-features"
      ;;
  esac
  
  if CARGO_INCREMENTAL=1 cargo build -p "$crate" $features 2>&1; then
    echo -e "${GREEN}✅ ${crate} built successfully${NC}"
    BUILT=$((BUILT + 1))
  else
    echo -e "${RED}❌ ${crate} build failed${NC}"
    FAILED=$((FAILED + 1))
  fi
  echo
done

# Summary
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}📊 Build Summary${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "Built: ${GREEN}${BUILT}${NC}"
echo -e "Failed: ${RED}${FAILED}${NC}"

if [ $FAILED -gt 0 ]; then
  exit 1
else
  exit 0
fi

