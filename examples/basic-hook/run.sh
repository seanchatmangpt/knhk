#!/bin/bash
# examples/basic-hook/run.sh
# Basic hook execution example

set -e

echo "=========================================="
echo "Basic Hook Example"
echo "=========================================="

# Create a test hook
echo "Creating test hook..."
knhk hook create test-ask ASK_SP 0xC0FFEE 0 8 0x1234

# Evaluate hot path hook
echo ""
echo "Evaluating hot path hook (ASK_SP)..."
knhk hook eval test-ask

# Create warm path hook
echo ""
echo "Creating warm path hook..."
knhk hook create test-construct CONSTRUCT8 0xC0FFEE 0 8 0x1234 0x5678

# Evaluate warm path hook
echo ""
echo "Evaluating warm path hook (CONSTRUCT8)..."
knhk hook eval test-construct

# List hooks
echo ""
echo "Listing hooks..."
knhk hook list

echo ""
echo "=========================================="
echo "Example complete"
echo "=========================================="
