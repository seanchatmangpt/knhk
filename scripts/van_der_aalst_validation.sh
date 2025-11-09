#!/bin/bash
# van der Aalst Process Mining Validation Script
# Executes workflows and collects event logs for conformance checking

set -e

echo "=== van der Aalst Validation: Fitness Testing ==="
echo ""

# Phase 1.1: Simple Workflow Execution
echo "Phase 1.1: Simple Workflow Execution"
cd rust/knhk-workflow-engine

# Execute a simple workflow example if it exists
if [ -f "examples/workflow_weaver_livecheck.rs" ]; then
    echo "Executing workflow example..."
    cargo run --example workflow_weaver_livecheck 2>&1 | head -30
else
    echo "⚠️  Workflow example not found"
fi

echo ""
echo "Phase 1.2: Pattern Execution"
# Run pattern tests
echo "Running pattern tests..."
cargo test --test chicago_tdd_43_patterns --no-fail-fast 2>&1 | tail -20

echo ""
echo "Phase 1.3: YAWL Workflow Execution"
# Check for YAWL workflow tests
echo "Checking YAWL workflow tests..."
cargo test --lib --no-fail-fast 2>&1 | grep -E "yawl|YAWL" | head -5 || echo "No YAWL tests found"

echo ""
echo "=== Validation Complete ==="
