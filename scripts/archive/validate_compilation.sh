#!/bin/bash
# Validation script to check compilation errors

cd /Users/sac/knhk/rust

echo "=== Checking knhk-etl ==="
cd knhk-etl && cargo check --lib 2>&1 | head -50
cd ..

echo ""
echo "=== Checking knhk-sidecar ==="
cd knhk-sidecar && cargo check --lib 2>&1 | head -50
cd ..

echo ""
echo "=== Checking knhk-warm ==="
cd knhk-warm && cargo check --lib 2>&1 | head -50
cd ..

echo ""
echo "=== Checking knhk-aot ==="
cd knhk-aot && cargo check --lib 2>&1 | head -50
cd ..

echo ""
echo "=== Checking knhk-lockchain ==="
cd knhk-lockchain && cargo check --lib 2>&1 | head -50
cd ..

echo ""
echo "=== Validation complete ==="

