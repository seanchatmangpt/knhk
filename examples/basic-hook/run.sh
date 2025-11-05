#!/bin/bash
# Basic Hook Execution Example
# Demonstrates creating and executing a knowledge hook

set -e

echo "=== Basic Hook Execution Example ==="
echo ""

# Create hook
echo "Creating hook: auth-check"
knhk hook create auth-check ASK_SP 0xC0FFEE 0 8

echo ""
echo "Listing hooks:"
knhk hook list

echo ""
echo "Evaluating hook:"
knhk hook eval auth-check

echo ""
echo "=== Example Complete ==="

