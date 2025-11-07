#!/usr/bin/env bash
# KNHK Build Order Script
# Builds C library first, then Rust crates in dependency order

set -euo pipefail

BLUE='\033[0;34m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🔨 KNHK Build Order${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# Step 1: Build C library (required for knhk-hot)
echo -e "${BLUE}Step 1: Building C library...${NC}"
cd "$PROJECT_ROOT/c"
if make lib; then
  echo -e "${GREEN}✅ C library built${NC}"
else
  echo -e "${RED}❌ C library build failed${NC}"
  exit 1
fi
echo

# Step 2: Build Rust crates in dependency order
cd "$PROJECT_ROOT"

echo -e "${BLUE}Step 2: Building Rust crates...${NC}"

# Crates with no internal dependencies (can build in parallel)
TIER_1=(
  "rust/knhk-config"
  "rust/knhk-otel"
  "rust/knhk-aot"
  "rust/knhk-unrdf"
)

# Crates that depend on C library
TIER_2=(
  "rust/knhk-hot"
)

# Crates that depend on tier 1
TIER_3=(
  "rust/knhk-warm"
  "rust/knhk-lockchain"
  "rust/knhk-validation"
)

# Higher-level crates
TIER_4=(
  "rust/knhk-connectors"
  "rust/knhk-etl"
)

# Application crates
TIER_5=(
  "rust/knhk-sidecar"
  "rust/knhk-cli"
  "rust/knhk-integration-tests"
)

build_crate() {
  local crate_path="$1"
  local crate_name=$(basename "$crate_path")

  if [ ! -d "$crate_path" ]; then
    echo -e "  ⚠️  $crate_name: not found, skipping"
    return 0
  fi

  echo -e "  Building $crate_name..."
  if (cd "$crate_path" && cargo build --release --quiet 2>&1 | grep -v "warning:"); then
    echo -e "  ${GREEN}✅ $crate_name${NC}"
  else
    echo -e "  ${RED}❌ $crate_name failed${NC}"
    return 1
  fi
}

echo "Tier 1: Base crates"
for crate in "${TIER_1[@]}"; do
  build_crate "$crate" || exit 1
done
echo

echo "Tier 2: C-dependent crates"
for crate in "${TIER_2[@]}"; do
  build_crate "$crate" || exit 1
done
echo

echo "Tier 3: Mid-level crates"
for crate in "${TIER_3[@]}"; do
  build_crate "$crate" || exit 1
done
echo

echo "Tier 4: High-level crates"
for crate in "${TIER_4[@]}"; do
  build_crate "$crate" || exit 1
done
echo

echo "Tier 5: Application crates"
for crate in "${TIER_5[@]}"; do
  build_crate "$crate" || exit 1
done
echo

echo -e "${GREEN}✅ Build complete${NC}"
