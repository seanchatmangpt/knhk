#!/usr/bin/env bash
# Setup build caching for KNHK workspace
# Installs and configures sccache for faster incremental builds

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🔧 KNHK Build Cache Setup${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CARGO_CONFIG="$PROJECT_ROOT/rust/.cargo/config.toml"

# Check if sccache is installed
if command -v sccache &> /dev/null; then
    echo -e "${GREEN}✅ sccache is already installed${NC}"
    SCCACHE_VERSION=$(sccache --version | head -1)
    echo "   Version: $SCCACHE_VERSION"
else
    echo -e "${YELLOW}📦 Installing sccache...${NC}"
    if cargo install sccache 2>&1; then
        echo -e "${GREEN}✅ sccache installed successfully${NC}"
    else
        echo -e "${YELLOW}⚠️  Failed to install sccache, continuing without it${NC}"
        echo "   You can install manually: cargo install sccache"
    fi
fi

# Configure sccache in .cargo/config.toml
if command -v sccache &> /dev/null; then
    echo
    echo -e "${BLUE}Configuring sccache in .cargo/config.toml...${NC}"
    
    # Check if rustc-wrapper is already set
    if grep -q "rustc-wrapper.*sccache" "$CARGO_CONFIG" 2>/dev/null; then
        echo -e "${GREEN}✅ sccache already configured${NC}"
    else
        # Uncomment the rustc-wrapper line
        if grep -q "# rustc-wrapper = \"sccache\"" "$CARGO_CONFIG" 2>/dev/null; then
            sed -i.bak 's/# rustc-wrapper = "sccache"/rustc-wrapper = "sccache"/' "$CARGO_CONFIG"
            rm -f "$CARGO_CONFIG.bak"
            echo -e "${GREEN}✅ sccache configured in .cargo/config.toml${NC}"
        else
            echo -e "${YELLOW}⚠️  Could not find sccache configuration line to uncomment${NC}"
            echo "   Please manually add to $CARGO_CONFIG:"
            echo "   [build]"
            echo "   rustc-wrapper = \"sccache\""
        fi
    fi
fi

# Check for fast linker (zld on macOS, mold on Linux)
echo
echo -e "${BLUE}Checking for fast linker...${NC}"

if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS - check for zld
    if command -v zld &> /dev/null || [ -f "/opt/homebrew/bin/zld" ]; then
        echo -e "${GREEN}✅ zld found${NC}"
        echo "   Fast linker is available"
    else
        echo -e "${YELLOW}💡 Install zld for faster linking:${NC}"
        echo "   cargo install -f zld"
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux - check for mold
    if command -v mold &> /dev/null; then
        echo -e "${GREEN}✅ mold found${NC}"
        echo "   Fast linker is available"
    else
        echo -e "${YELLOW}💡 Install mold for faster linking:${NC}"
        echo "   sudo apt install mold  # Debian/Ubuntu"
        echo "   or: cargo install -f mold"
    fi
fi

echo
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Build cache setup complete!${NC}"
echo
echo "Next steps:"
echo "  1. Run: make check  (should be fast with incremental builds)"
echo "  2. Run: make test-rust  (lib tests only, fast)"
echo "  3. Monitor: sccache --show-stats  (if sccache is installed)"
echo
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"


