#!/usr/bin/env bash
# Install test optimization dependencies
# Installs cargo-nextest, fswatch/inotify-tools, and sccache

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}📦 Installing Test Optimization Dependencies${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# Install cargo-nextest
if command -v cargo-nextest > /dev/null 2>&1; then
  echo -e "${GREEN}✅ cargo-nextest already installed${NC}"
else
  echo -e "${BLUE}Installing cargo-nextest...${NC}"
  cargo install cargo-nextest --locked
  echo -e "${GREEN}✅ cargo-nextest installed${NC}"
fi

# Install file watcher
if [[ "$OSTYPE" == "darwin"* ]]; then
  # macOS
  if command -v fswatch > /dev/null 2>&1; then
    echo -e "${GREEN}✅ fswatch already installed${NC}"
  else
    echo -e "${BLUE}Installing fswatch (macOS file watcher)...${NC}"
    if command -v brew > /dev/null 2>&1; then
      brew install fswatch
      echo -e "${GREEN}✅ fswatch installed${NC}"
    else
      echo -e "${YELLOW}⚠️  Homebrew not found. Install fswatch manually:${NC}"
      echo -e "${YELLOW}   brew install fswatch${NC}"
    fi
  fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
  # Linux
  if command -v inotifywait > /dev/null 2>&1; then
    echo -e "${GREEN}✅ inotify-tools already installed${NC}"
  else
    echo -e "${BLUE}Installing inotify-tools (Linux file watcher)...${NC}"
    if command -v apt-get > /dev/null 2>&1; then
      sudo apt-get update && sudo apt-get install -y inotify-tools
      echo -e "${GREEN}✅ inotify-tools installed${NC}"
    elif command -v yum > /dev/null 2>&1; then
      sudo yum install -y inotify-tools
      echo -e "${GREEN}✅ inotify-tools installed${NC}"
    else
      echo -e "${YELLOW}⚠️  Package manager not found. Install inotify-tools manually${NC}"
    fi
  fi
fi

# Install sccache (optional, for binary caching)
if command -v sccache > /dev/null 2>&1; then
  echo -e "${GREEN}✅ sccache already installed${NC}"
else
  echo -e "${BLUE}Installing sccache (binary cache)...${NC}"
  cargo install sccache
  echo -e "${GREEN}✅ sccache installed${NC}"
  echo -e "${YELLOW}   Configure with: export RUSTC_WRAPPER=sccache${NC}"
fi

echo
echo -e "${GREEN}✅ All dependencies installed${NC}"
echo
echo -e "${BLUE}Next steps:${NC}"
echo -e "${BLUE}  1. Start test cache daemon: ./scripts/test-cache-daemon.sh start${NC}"
echo -e "${BLUE}  2. Run optimized tests: ./scripts/test-runner-optimized.sh${NC}"

