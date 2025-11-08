#!/bin/bash
# Comprehensive validation and fix script for compilation errors

set -e

cd /Users/sac/knhk/rust

echo "=== VALIDATION PHASE ==="

# Check each crate
for crate in knhk-etl knhk-sidecar knhk-warm knhk-aot knhk-lockchain knhk-validation knhk-unrdf; do
    if [ -d "$crate" ]; then
        echo ""
        echo "Checking $crate..."
        cd "$crate"
        cargo check --lib 2>&1 | head -30 || true
        cd ..
    fi
done

echo ""
echo "=== FIX PHASE ==="

# Fix 1: knhk-aot no_std placement
if [ -f "knhk-aot/src/template_analyzer.rs" ]; then
    if grep -q "^#!\[no_std\]" knhk-aot/src/template_analyzer.rs; then
        echo "Fixing knhk-aot: Removing #![no_std] from template_analyzer.rs"
        sed -i.bak '1{/^#!\[no_std\]/d}' knhk-aot/src/template_analyzer.rs
        
        # Add to lib.rs if not present
        if ! grep -q "^#!\[no_std\]" knhk-aot/src/lib.rs; then
            echo "Fixing knhk-aot: Adding #![no_std] to lib.rs"
            sed -i.bak '1i#![no_std]' knhk-aot/src/lib.rs
        fi
    fi
fi

# Fix 2: knhk-sidecar dependencies
if [ -f "knhk-sidecar/Cargo.toml" ]; then
    if ! grep -q "knhk-etl" knhk-sidecar/Cargo.toml; then
        echo "Fixing knhk-sidecar: Adding knhk-etl dependency"
        # Add after knhk-otel dependency
        sed -i.bak '/knhk-otel = {/a\
knhk-etl = { path = "../knhk-etl", features = ["std"] }
' knhk-sidecar/Cargo.toml
    fi
fi

echo ""
echo "=== Re-validating after fixes ==="
cargo check --workspace 2>&1 | head -100 || true

echo ""
echo "=== Validation and fix complete ==="

