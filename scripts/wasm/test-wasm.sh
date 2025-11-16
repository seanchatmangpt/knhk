#!/usr/bin/env bash
set -euo pipefail

# KNHK WASM Test Runner
# Runs WASM tests in headless browsers

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
WASM_CRATE="$PROJECT_ROOT/rust/knhk-wasm"

echo "🧪 Running WASM tests..."

cd "$WASM_CRATE"

# Check for required tools
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Run tests in different browsers
echo "🦊 Testing in Firefox..."
wasm-pack test --headless --firefox

echo "🌐 Testing in Chrome..."
wasm-pack test --headless --chrome

echo ""
echo "✅ All WASM tests passed!"
