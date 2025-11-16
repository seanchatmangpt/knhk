#!/usr/bin/env bash
set -euo pipefail

# KNHK WASM Build Pipeline
# Compiles Rust to WASM with size optimization

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
WASM_CRATE="$PROJECT_ROOT/rust/knhk-wasm"
DIST_DIR="$PROJECT_ROOT/wasm-dist"

echo "🚀 Building KNHK WASM module..."

# Check for wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# Build for different targets
echo "📦 Building for web target..."
cd "$WASM_CRATE"
wasm-pack build --target web --out-dir "$DIST_DIR/web" --release

echo "📦 Building for Node.js target..."
wasm-pack build --target nodejs --out-dir "$DIST_DIR/nodejs" --release

echo "📦 Building for bundler target..."
wasm-pack build --target bundler --out-dir "$DIST_DIR/bundler" --release

# Optimize WASM binaries
if command -v wasm-opt &> /dev/null; then
    echo "⚡ Optimizing WASM binaries..."

    for target in web nodejs bundler; do
        WASM_FILE="$DIST_DIR/$target/knhk_wasm_bg.wasm"
        if [ -f "$WASM_FILE" ]; then
            echo "  Optimizing $target..."
            wasm-opt -Oz --enable-simd -o "${WASM_FILE}.opt" "$WASM_FILE"
            mv "${WASM_FILE}.opt" "$WASM_FILE"
        fi
    done
else
    echo "⚠️  wasm-opt not found. Skipping optimization."
    echo "   Install with: cargo install wasm-opt"
fi

# Get file sizes
echo ""
echo "📊 Build Results:"
echo "================="
for target in web nodejs bundler; do
    WASM_FILE="$DIST_DIR/$target/knhk_wasm_bg.wasm"
    if [ -f "$WASM_FILE" ]; then
        SIZE=$(du -h "$WASM_FILE" | cut -f1)
        echo "  $target: $SIZE"
    fi
done

# Create tarball for distribution
echo ""
echo "📦 Creating distribution package..."
cd "$DIST_DIR"
tar -czf knhk-wasm.tar.gz web/ nodejs/ bundler/
echo "  Created: $DIST_DIR/knhk-wasm.tar.gz"

echo ""
echo "✅ WASM build complete!"
echo "   Output: $DIST_DIR"
