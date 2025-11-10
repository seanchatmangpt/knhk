#!/usr/bin/env bash
# KNHK Test Cache Daemon
# Autonomic file watcher that keeps test binaries pre-compiled and ready
# Monitors code changes and automatically rebuilds test binaries in background

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
PID_FILE="${CACHE_DIR}/daemon.pid"
LOG_FILE="${CACHE_DIR}/daemon.log"
LOCK_FILE="${CACHE_DIR}/build.lock"
HASH_FILE="${CACHE_DIR}/code.hash"

# Ensure cache directory exists
mkdir -p "$CACHE_DIR"

# Check if daemon is already running
check_daemon() {
  if [ -f "$PID_FILE" ]; then
    local pid=$(cat "$PID_FILE")
    if ps -p "$pid" > /dev/null 2>&1; then
      return 0  # Running
    else
      rm -f "$PID_FILE"
      return 1  # Not running
    fi
  fi
  return 1  # Not running
}

# Generate hash of all Rust source files
generate_code_hash() {
  local hash
  hash=$(find "$PROJECT_ROOT/rust" -name "*.rs" -type f -exec sha256sum {} \; 2>/dev/null | \
         sort | sha256sum | cut -d' ' -f1)
  echo "$hash"
}

# Check if code has changed
code_changed() {
  local current_hash=$(generate_code_hash)
  local cached_hash=""
  
  if [ -f "$HASH_FILE" ]; then
    cached_hash=$(cat "$HASH_FILE")
  fi
  
  if [ "$current_hash" != "$cached_hash" ]; then
    echo "$current_hash" > "$HASH_FILE"
    return 0  # Changed
  fi
  
  return 1  # Unchanged
}

# Pre-compile test binaries
precompile_tests() {
  # Acquire lock to prevent concurrent builds
  if [ -f "$LOCK_FILE" ]; then
    local lock_pid=$(cat "$LOCK_FILE")
    if ps -p "$lock_pid" > /dev/null 2>&1; then
      echo "$(date '+%Y-%m-%d %H:%M:%S') - Build already in progress (PID: $lock_pid)" >> "$LOG_FILE"
      return 0
    fi
  fi
  
  echo $$ > "$LOCK_FILE"
  echo "$(date '+%Y-%m-%d %H:%M:%S') - Starting test binary pre-compilation..." >> "$LOG_FILE"
  
  # Pre-compile test binaries
  cd "$PROJECT_ROOT/rust"
  if CARGO_INCREMENTAL=1 cargo build --tests --workspace > "${CACHE_DIR}/build.log" 2>&1; then
    echo "$(date '+%Y-%m-%d %H:%M:%S') - ✅ Test binaries pre-compiled successfully" >> "$LOG_FILE"
    
    # Also compile with nextest if available
    if command -v cargo-nextest > /dev/null 2>&1; then
      cargo nextest run --workspace --no-run --profile fast >> "${CACHE_DIR}/build.log" 2>&1 || true
    fi
    
    rm -f "$LOCK_FILE"
    return 0
  else
    echo "$(date '+%Y-%m-%d %H:%M:%S') - ❌ Test binary compilation failed" >> "$LOG_FILE"
    rm -f "$LOCK_FILE"
    return 1
  fi
}

# File watcher function
watch_files() {
  local watcher_cmd=""
  
  # Detect available file watcher
  if command -v fswatch > /dev/null 2>&1; then
    # macOS fswatch
    watcher_cmd="fswatch -o -r -l 0.5 --exclude='.*\.(git|target|test-cache)' --include='.*\.rs$'"
  elif command -v inotifywait > /dev/null 2>&1; then
    # Linux inotifywait
    watcher_cmd="inotifywait -m -r -e modify,create,delete --format '%w%f' --exclude '\.(git|target|test-cache)'"
  else
    echo -e "${RED}❌ No file watcher found. Install fswatch (macOS) or inotify-tools (Linux)${NC}"
    exit 1
  fi
  
  echo -e "${BLUE}🔍 Watching for code changes...${NC}"
  echo "$(date '+%Y-%m-%d %H:%M:%S') - File watcher started" >> "$LOG_FILE"
  
  # Watch for changes
  eval "$watcher_cmd" "$PROJECT_ROOT/rust" | while read -r changed_file; do
    # Debounce: wait 1 second for more changes
    sleep 1
    
    if code_changed; then
      echo -e "${YELLOW}📝 Code changed, pre-compiling test binaries...${NC}"
      precompile_tests &
    fi
  done
}

# Start daemon
start_daemon() {
  if check_daemon; then
    echo -e "${YELLOW}⚠️  Daemon already running (PID: $(cat "$PID_FILE"))${NC}"
    return 1
  fi
  
  echo -e "${BLUE}🚀 Starting test cache daemon...${NC}"
  
  # Initial pre-compilation
  if code_changed; then
    echo -e "${BLUE}📦 Pre-compiling test binaries...${NC}"
    precompile_tests
  else
    echo -e "${GREEN}✅ Test binaries already up-to-date${NC}"
  fi
  
  # Start file watcher in background
  watch_files &
  local watcher_pid=$!
  echo $watcher_pid > "$PID_FILE"
  
  echo -e "${GREEN}✅ Test cache daemon started (PID: $watcher_pid)${NC}"
  echo -e "${BLUE}   Log: $LOG_FILE${NC}"
  echo -e "${BLUE}   PID: $PID_FILE${NC}"
}

# Stop daemon
stop_daemon() {
  if ! check_daemon; then
    echo -e "${YELLOW}⚠️  Daemon not running${NC}"
    return 1
  fi
  
  local pid=$(cat "$PID_FILE")
  echo -e "${BLUE}🛑 Stopping test cache daemon (PID: $pid)...${NC}"
  
  # Kill daemon and all children
  pkill -P "$pid" 2>/dev/null || true
  kill "$pid" 2>/dev/null || true
  
  rm -f "$PID_FILE"
  echo -e "${GREEN}✅ Daemon stopped${NC}"
}

# Status check
status_daemon() {
  if check_daemon; then
    local pid=$(cat "$PID_FILE")
    echo -e "${GREEN}✅ Daemon running (PID: $pid)${NC}"
    
    if [ -f "$LOCK_FILE" ]; then
      local lock_pid=$(cat "$LOCK_FILE")
      if ps -p "$lock_pid" > /dev/null 2>&1; then
        echo -e "${YELLOW}   Building test binaries (PID: $lock_pid)${NC}"
      fi
    else
      echo -e "${GREEN}   Test binaries ready${NC}"
    fi
    
    # Show last log entries
    if [ -f "$LOG_FILE" ]; then
      echo -e "${BLUE}   Last log entries:${NC}"
      tail -5 "$LOG_FILE" | sed 's/^/   /'
    fi
  else
    echo -e "${RED}❌ Daemon not running${NC}"
  fi
}

# Force rebuild
rebuild_cache() {
  echo -e "${BLUE}🔨 Forcing test binary rebuild...${NC}"
  rm -f "$HASH_FILE"
  code_changed  # This will update hash
  precompile_tests
  echo -e "${GREEN}✅ Rebuild complete${NC}"
}

# Main command handler
case "${1:-start}" in
  start)
    start_daemon
    ;;
  stop)
    stop_daemon
    ;;
  restart)
    stop_daemon
    sleep 1
    start_daemon
    ;;
  status)
    status_daemon
    ;;
  rebuild)
    rebuild_cache
    ;;
  *)
    echo "Usage: $0 {start|stop|restart|status|rebuild}"
    exit 1
    ;;
esac

