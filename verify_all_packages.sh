#!/bin/bash
# Verification script for all packages

set -e

cd "$(dirname "$0")/rust"

echo "=== Checking Workspace Structure ==="
echo "Workspace members:"
grep -A 15 "members = \[" Cargo.toml | grep -E '"(knhk-|knhk-)"' || true

echo ""
echo "=== Checking Package Names ==="
for dir in knhk-* knhk-*; do
    if [ -d "$dir" ] && [ -f "$dir/Cargo.toml" ]; then
        pkg_name=$(grep "^name = " "$dir/Cargo.toml" | head -1 | sed 's/name = "\(.*\)"/\1/')
        echo "$dir: $pkg_name"
    fi
done

echo ""
echo "=== Checking Compilation ==="
echo "Running cargo check --workspace..."
cargo check --workspace 2>&1 | tail -50 || echo "Compilation check failed"

echo ""
echo "=== Checking Clippy ==="
echo "Running cargo clippy --workspace..."
cargo clippy --workspace -- -D warnings 2>&1 | tail -50 || echo "Clippy check failed"

echo ""
echo "=== Checking C Code ==="
cd ../c
echo "Running make lib..."
make clean && make lib 2>&1 | tail -50 || echo "C compilation failed"

