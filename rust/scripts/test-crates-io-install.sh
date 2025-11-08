#!/bin/bash
set -e

# Use full jq path or fallback to PATH
JQ="${JQ:-$(which jq 2>/dev/null || echo '/opt/homebrew/bin/jq')}"

echo "🔍 Testing crates.io installation simulation..."
echo ""

# Change to rust directory
cd "$(dirname "$0")/.."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test dry-run for all crates in dependency order
echo -e "${YELLOW}Step 1: Validating package metadata${NC}"
for crate in knhk-otel knhk-lockchain knhk-hot knhk-etl knhk-warm knhk-config knhk-connectors knhk-cli; do
    if [ ! -f "$crate/Cargo.toml" ]; then
        echo -e "${YELLOW}  ⚠️  $crate not found (skipping)${NC}"
        continue
    fi

    echo "  Checking $crate metadata..."
    cd $crate

    # Check required fields
    if ! cargo metadata --no-deps --format-version 1 2>/dev/null | $JQ -e '.packages[0].license' > /dev/null; then
        echo -e "${RED}  ✗ Missing license in $crate${NC}"
        exit 1
    fi

    if ! cargo metadata --no-deps --format-version 1 2>/dev/null | $JQ -e '.packages[0].description' > /dev/null; then
        echo -e "${RED}  ✗ Missing description in $crate${NC}"
        exit 1
    fi

    if ! cargo metadata --no-deps --format-version 1 2>/dev/null | $JQ -e '.packages[0].repository' > /dev/null; then
        echo -e "${RED}  ✗ Missing repository in $crate${NC}"
        exit 1
    fi

    cd ..
    echo -e "${GREEN}  ✓ $crate metadata valid${NC}"
done
echo ""

echo -e "${YELLOW}Step 2: Dry-run publish validation${NC}"
for crate in knhk-otel knhk-lockchain knhk-hot knhk-etl knhk-warm knhk-config knhk-connectors knhk-cli; do
    if [ ! -f "$crate/Cargo.toml" ]; then
        continue
    fi

    echo "  Validating $crate..."
    if cargo publish --dry-run --manifest-path $crate/Cargo.toml 2>&1 | tee /tmp/publish-$crate.log; then
        echo -e "${GREEN}  ✓ $crate passes dry-run${NC}"
    else
        echo -e "${RED}  ✗ $crate failed dry-run${NC}"
        echo "Error log:"
        cat /tmp/publish-$crate.log
        exit 1
    fi
done
echo ""

echo -e "${YELLOW}Step 3: Building CLI binary${NC}"
if cargo build --release --bin knhk; then
    echo -e "${GREEN}  ✓ CLI binary built successfully${NC}"
else
    echo -e "${RED}  ✗ CLI binary build failed${NC}"
    exit 1
fi
echo ""

echo -e "${YELLOW}Step 4: Testing local install${NC}"
if cargo install --path knhk-cli --force; then
    echo -e "${GREEN}  ✓ Local install successful${NC}"
else
    echo -e "${RED}  ✗ Local install failed${NC}"
    exit 1
fi
echo ""

echo -e "${YELLOW}Step 5: Verifying installation${NC}"
echo "  Testing: knhk --version"
if knhk --version; then
    echo -e "${GREEN}  ✓ Version command works${NC}"
else
    echo -e "${RED}  ✗ Version command failed${NC}"
    exit 1
fi

echo ""
echo "  Testing: knhk --help"
if knhk --help > /dev/null; then
    echo -e "${GREEN}  ✓ Help command works${NC}"
else
    echo -e "${RED}  ✗ Help command failed${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}✅ All crates.io publication checks passed!${NC}"
echo ""
echo "Ready to publish with:"
echo "  git tag v0.1.0"
echo "  git push origin v0.1.0"
