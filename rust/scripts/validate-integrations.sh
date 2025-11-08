#!/bin/bash
# Validate integration scenarios
# Part of KNHK Build Validation Matrix
set -e

echo "=== Integration Scenario Validation ==="
echo "Testing package integration across core, pipeline, and validation systems"
echo ""

# Core system integration
echo "## 1. Core System Integration"
echo "Building core infrastructure packages..."
cargo build -p knhk-hot -p knhk-otel -p knhk-config -p knhk-lockchain --release
echo "✅ Core system build successful"

echo "Testing core infrastructure packages..."
cargo test -p knhk-hot -p knhk-otel -p knhk-config -p knhk-lockchain --lib
echo "✅ Core system tests passed"
echo ""

# Pipeline system integration
echo "## 2. Pipeline System Integration"
echo "Building pipeline packages with all features..."
cargo build -p knhk-etl -p knhk-warm -p knhk-patterns -p knhk-unrdf --release --all-features
echo "✅ Pipeline system build successful"

echo "Testing pipeline packages..."
cargo test -p knhk-etl -p knhk-warm -p knhk-patterns -p knhk-unrdf --lib
echo "✅ Pipeline system tests passed"
echo ""

# Validation system integration
echo "## 3. Validation System Integration"
echo "Building validation packages with all features..."
cargo build -p knhk-validation -p knhk-lockchain -p knhk-connectors --release --all-features
echo "✅ Validation system build successful"

echo "Testing validation packages..."
cargo test -p knhk-validation -p knhk-lockchain -p knhk-connectors --lib
echo "✅ Validation system tests passed"
echo ""

# Full workspace integration
echo "## 4. Full Workspace Integration"
echo "Building entire workspace with all features..."
cargo build --workspace --release --all-features
echo "✅ Full workspace build successful"

echo "Testing entire workspace..."
cargo test --workspace --lib
echo "✅ Full workspace tests passed"
echo ""

echo "=== Integration Validation Summary ==="
echo "✅ Core system integration: PASSED"
echo "✅ Pipeline system integration: PASSED"
echo "✅ Validation system integration: PASSED"
echo "✅ Full workspace integration: PASSED"
echo ""
echo "✅ All integration scenarios validated successfully"
